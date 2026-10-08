//! The iced desktop application: settings, search/add, and download queue.

use crate::style::{self, compact_button};
use album::{AlbumDetail, DetailState};
use cover_art::CoverArt;
use iced::futures::{future, SinkExt};
use iced::widget::{column, container, row, scrollable, text};
use iced::{Element, Length, Task, Theme};
use iced_aw::widget::{tab_bar::TabLabel, tabs::Tabs};
use omnibox::Submit;
use paging::{Kind, Page, Section};
use qobuz_core::catalog::Reference;
use qobuz_core::config::Config;
use qobuz_core::engine::{Job, JobEvent};
use qobuz_core::quality::Quality;
use qobuz_core::rename;
use qobuz_core::tag_edit::{CoverPlan, Saved};
use qobuz_core::{auth, engine, AppCredentials, CancellationToken, QobuzClient, SigningCheck};
use status::Status;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use tag_editor::TagEditor;

mod album;
mod cover_art;
mod help;
mod notify;
mod omnibox;
mod open;
mod paging;
mod shortcut;
mod status;
mod tag_editor;
mod tasks;
mod view;

/// Fits the longer of "Light" / "Dark", so the toggle doesn't shift when its
/// label changes.
const THEME_TOGGLE_WIDTH: f32 = 64.0;

/// The app/window icon, rasterized from `assets/icon.svg`.
fn window_icon() -> Option<iced::window::Icon> {
    iced::window::icon::from_file_data(include_bytes!("../assets/icon.png"), None).ok()
}

pub fn run() -> iced::Result {
    let window = iced::window::Settings {
        size: iced::Size::new(1040.0, 1000.0),
        icon: window_icon(),
        ..Default::default()
    };
    iced::application("Qobuz-dl", App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        // iced_aw's NumberInput draws its spinner carets from this icon font.
        .font(iced_aw::iced_fonts::REQUIRED_FONT_BYTES)
        // Bundle Inter and make it the default so glyphs (dots, arrows, ×, ☀)
        // render identically on every OS instead of relying on font fallback.
        .font(include_bytes!("../assets/fonts/Inter-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/Inter-Bold.ttf").as_slice())
        .default_font(iced::Font::with_name("Inter"))
        .window(window)
        .run_with(App::new)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Settings,
    Search,
    Queue,
}

/// The queue's track and album ids, so search results can show what is
/// already queued without scanning the queue once per row.
struct Queued<'a> {
    tracks: HashSet<i64>,
    albums: HashSet<&'a str>,
}

impl Queued<'_> {
    /// Result ids are strings; one that isn't numeric can't be a queued track.
    fn track(&self, track_id: &str) -> bool {
        track_id
            .parse::<i64>()
            .is_ok_and(|id| self.tracks.contains(&id))
    }

    /// Whether any track of the album is queued.
    fn album(&self, album_id: &str) -> bool {
        self.albums.contains(album_id)
    }
}

/// A single row in the download queue.
#[derive(Debug, Clone)]
struct QueueItem {
    track_id: i64,
    /// The resolved job, retained so a failed track can be relaunched.
    job: Job,
    status: ItemStatus,
    downloaded: u64,
    total: Option<u64>,
    /// The downloaded file once done: where its group's "Open folder" looks.
    path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
enum ItemStatus {
    Queued,
    Downloading,
    Tagging,
    Done(String),
    Error(String),
}

/// Whether the queue holds anything a fresh Start would act on: tracks that
/// have never been attempted. Shared by `Message::StartDownloads` and the Queue
/// header, so the button is only offered when pressing it would do something.
/// Failed tracks are deliberately excluded — the per-item Retry and
/// "Retry failed (N)" controls own those.
fn startable(queue: &[QueueItem]) -> bool {
    queue
        .iter()
        .any(|it| matches!(it.status, ItemStatus::Queued))
}

/// Whether a track can be taken out of the queue list: anything settled. A
/// downloading or tagging track is still in the engine's hands. Removing
/// never touches files on disk.
fn removable(status: &ItemStatus) -> bool {
    matches!(
        status,
        ItemStatus::Queued | ItemStatus::Done(_) | ItemStatus::Error(_)
    )
}

/// Tracks still to process: queued, downloading, or tagging. Done and failed
/// tracks are settled, so they don't count toward the Queue tab's number.
fn remaining(queue: &[QueueItem]) -> usize {
    queue
        .iter()
        .filter(|it| {
            matches!(
                it.status,
                ItemStatus::Queued | ItemStatus::Downloading | ItemStatus::Tagging
            )
        })
        .count()
}

/// Settle a "Show more" response on its section: clear the loading flag, then
/// either append the page and return the covers it brought in, or hand back
/// the error with the existing rows left as they were.
fn more_loaded<T>(
    section: &mut Section<T>,
    result: Result<Page<T>, String>,
    id: impl Fn(&T) -> &str,
    cover: impl Fn(&T) -> Option<&String>,
) -> Result<Vec<String>, String> {
    section.loading = false;
    let page = result?;
    let covers = page
        .items
        .iter()
        .filter_map(|it| cover(it).cloned())
        .collect();
    section.append(page, id);
    Ok(covers)
}

fn open_folder(folder: PathBuf) -> Task<Message> {
    Task::perform(tasks::open_path(folder), Message::Opened)
}

fn queue_tab_label(queue: &[QueueItem]) -> String {
    match remaining(queue) {
        0 => "Queue".to_owned(),
        n => format!("Queue ({n})"),
    }
}

/// An album search result: id, title, artist, an optional cover URL, and
/// whether it is available in hi-res.
#[derive(Debug, Clone)]
struct AlbumResult {
    id: String,
    title: String,
    artist: String,
    cover: Option<String>,
    hires: bool,
}

/// A track search result: id, title, artist, and whether it is hi-res.
#[derive(Debug, Clone)]
struct TrackResult {
    id: String,
    title: String,
    artist: String,
    cover: Option<String>,
    hires: bool,
}

/// Search results reduced to display-ready entries, each type paged on its own.
/// `query` is what "Show more" asks for, independent of later edits to the field.
#[derive(Debug, Clone, Default)]
struct SearchPayload {
    query: String,
    albums: Section<AlbumResult>,
    tracks: Section<TrackResult>,
    /// Why one type's section is empty, when only that type's search failed.
    failure: Option<String>,
}

/// The Queue tab's tag editor: reading an album's files, then editing them.
#[derive(Debug, Clone)]
enum EditorSlot {
    /// The album id whose files are being read.
    Loading(String),
    Open(Box<TagEditor>),
}

impl EditorSlot {
    fn album_id(&self) -> &str {
        match self {
            EditorSlot::Loading(id) => id,
            EditorSlot::Open(editor) => &editor.album_id,
        }
    }
}

/// An album group's open Rename folder field.
#[derive(Debug, Clone)]
struct RenameState {
    album_id: String,
    name: String,
    /// Whether the rename template's suggestion is still being read from the
    /// files; typing cancels it, so a late suggestion never replaces typed text.
    suggesting: bool,
}

/// Point every queue item whose file was inside `old` at the same file under `new`.
fn rewrite_paths(queue: &mut [QueueItem], old: &Path, new: &Path) {
    for it in queue {
        let moved = it
            .path
            .as_deref()
            .and_then(|p| p.strip_prefix(old).ok())
            .map(|rest| new.join(rest));
        if moved.is_some() {
            it.path = moved;
        }
    }
}

/// How the active session's token came to be — shown in the Account card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TokenOrigin {
    /// Loaded from the OS keyring at startup.
    Restored,
    /// Pasted and validated by a Sign in during this session.
    ValidatedThisSession,
}

/// The in-memory copy of the stored auth token plus its origin.
#[derive(Debug, Clone)]
struct StoredToken {
    value: String,
    origin: TokenOrigin,
}

pub struct App {
    screen: Screen,
    config: Config,
    /// What is on disk, so unsaved edits are derived by comparison rather than
    /// tracked per message. `None` when the file could not be read: the shown
    /// defaults are not on disk, so Save must stay available to replace it.
    saved_config: Option<Config>,
    /// `None` once the user dismisses an error.
    status: Option<Status>,
    token: Option<StoredToken>,

    // Settings form fields.
    token_input: String,
    /// Whether `config.app_secret` was hand-edited since the last auto-detect.
    /// A detected secret set is trusted as a whole (only one candidate is valid),
    /// so the signing check silently adopts the working candidate; a hand-edited
    /// secret that only signs via a fallback is surfaced as a warning instead.
    secret_manually_edited: bool,

    // Search / add.
    search_query: String,
    /// The last query, when it was also a valid bare ID, offered as an explicit
    /// add action above the results.
    offered_id: Option<Reference>,
    /// Bumped on every submitted search.
    search_generation: u64,
    /// An album opened from the results, shown in their place on the Search tab.
    album: Option<AlbumDetail>,
    /// Where the results list was scrolled to, restored when an album closes.
    results_offset: scrollable::AbsoluteOffset,
    results: SearchPayload,
    /// Album cover thumbnails, keyed by cover URL, loaded lazily.
    thumbnails: HashMap<String, iced::widget::image::Handle>,
    /// Cover URLs being fetched or whose fetch failed, so neither is requested
    /// again. A failed cover is not retried during the session.
    cover_requests: HashSet<String>,

    // Queue.
    queue: Vec<QueueItem>,
    /// Album ids whose queue group is collapsed to its header.
    collapsed: HashSet<String>,
    /// An album's tag editor, shown on the Queue tab in place of the list.
    tag_editor: Option<EditorSlot>,
    rename: Option<RenameState>,
    downloading: bool,
    /// Track ids of the running (or last) batch, so its outcome counts only
    /// its own tracks, not ones finished by earlier batches.
    batch: Vec<i64>,
    /// Whether the app window has focus; notifications are skipped while it does.
    window_focused: bool,
    /// Cancels the running batch. A fresh token per batch — reusing one would
    /// start the next batch already cancelled. `None` while idle.
    cancel: Option<CancellationToken>,

    // UI preferences.
    show_template_help: bool,
    show_credentials_help: bool,
    show_account_help: bool,
    show_options_help: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Navigate(Screen),
    Shortcut(shortcut::Shortcut),
    ToggleTheme,
    DismissStatus,
    ToggleTemplateHelp,
    ToggleCredentialsHelp,
    ToggleAccountHelp,
    ToggleOptionsHelp,
    CopyTemplate(String),

    // Settings inputs.
    TokenChanged(String),
    AppIdChanged(String),
    AppSecretChanged(String),
    AutoDetectCredentials,
    CredentialsDetected(Result<AppCredentials, String>),
    CheckSigning,
    SigningChecked(Result<SigningCheck, String>),
    FolderFormatChanged(String),
    TrackFormatChanged(String),
    RenameFormatChanged(String),
    ConcurrencyChanged(usize),
    QualitySelected(Quality),
    CoverArtSelected(CoverArt),
    NotifyToggled(bool),
    PickDir,
    DirChosen(Option<PathBuf>),
    SaveSettings,
    LoginToken,
    LoggedIn(Result<String, String>),
    SignOut,

    // Search / add.
    SearchQueryChanged(String),
    SearchSubmit,
    /// Search and page responses carry the search generation they were
    /// requested under, so a response from a superseded search is dropped.
    SearchDone(u64, Result<SearchPayload, String>),
    ShowMore(Kind),
    MoreAlbums(u64, Result<Page<AlbumResult>, String>),
    MoreTracks(u64, Result<Page<TrackResult>, String>),
    ResultsScrolled(scrollable::Viewport),

    // Album detail.
    OpenAlbum(AlbumResult),
    /// Keyed by album id so a response for an album no longer open is dropped.
    AlbumLoaded(String, Result<Vec<Job>, String>),
    RetryAlbum,
    CloseAlbum,
    ToggleTrack(i64),
    SelectAllTracks,
    SelectNoTracks,
    AddSelected,
    ThumbnailLoaded(String, Result<Vec<u8>, ()>),
    Add(Reference),
    Resolved(Result<Vec<Job>, String>),

    // Downloads.
    StartDownloads,
    CancelDownloads,
    RetryFailed,
    ClearQueue,
    ToggleGroup(String),
    /// Takes all settled tracks of one album group out of the queue list.
    RemoveGroup(String),
    /// Opens the folder holding an album group's downloaded files.
    OpenAlbumFolder(String),
    OpenDownloadDir,
    Opened(Result<(), String>),
    Download(JobEvent),
    /// Carries the app secret that actually signed during the batch, if any, so
    /// it can be promoted to the primary secret and persisted.
    DownloadsFinished(Option<String>),
    WindowFocus(bool),
    /// A desktop notification was posted (or failed and was logged).
    Notified,

    // Tag editor.
    EditTags(String),
    /// Keyed by album id so a read for an editor since closed is dropped.
    TagsRead(String, Result<Vec<tasks::ReadTags>, String>),
    Editor(tag_editor::Edit),
    PickCover,
    CoverPicked(Result<Option<Vec<u8>>, String>),
    SaveTags,
    CoverPlanned(Result<CoverPlan, String>),
    TagsSaved(i64, Result<Saved, String>),
    CloseTagEditor,

    // Rename folder.
    RenameFolder(String),
    /// Keyed by album id so a suggestion for a field since closed is dropped.
    RenameSuggested(String, Result<Option<String>, String>),
    RenameNameChanged(String),
    ConfirmRename,
    CancelRename,
    /// Carries the folder as it was before the rename.
    FolderRenamed(PathBuf, Result<PathBuf, String>),
}

impl App {
    fn new() -> (Self, Task<Message>) {
        // `load` only errors on a real failure (a missing file yields defaults) —
        // don't silently discard the user's saved settings without a hint.
        let (config, config_error) = match Config::load() {
            Ok(c) => (c, None),
            Err(e) => {
                tracing::warn!("could not load config: {e}");
                (Config::default(), Some(e))
            }
        };
        let token = auth::load_token().ok().flatten().map(|value| StoredToken {
            value,
            origin: TokenOrigin::Restored,
        });
        let load_failed = config_error.is_some();
        // Missing setup is shown by the Search screen's prompt, not here.
        let status = if let Some(e) = config_error {
            Some(Status::error(format!(
                "Could not load saved settings ({e}); using defaults."
            )))
        } else {
            token
                .is_some()
                .then(|| Status::info("Restored saved session."))
        };
        let mut app = Self::from_parts(config, token, status);
        if load_failed {
            app.saved_config = None;
        }
        (app, Task::none())
    }

    /// The initial state for already-loaded settings and token, without
    /// touching the config file or the keyring.
    fn from_parts(config: Config, token: Option<StoredToken>, status: Option<Status>) -> Self {
        App {
            screen: Screen::Search,
            show_template_help: false,
            show_credentials_help: false,
            show_account_help: false,
            show_options_help: false,
            token_input: String::new(),
            secret_manually_edited: false,
            search_query: String::new(),
            offered_id: None,
            search_generation: 0,
            album: None,
            results_offset: scrollable::AbsoluteOffset::default(),
            results: SearchPayload::default(),
            thumbnails: HashMap::new(),
            cover_requests: HashSet::new(),
            queue: Vec::new(),
            collapsed: HashSet::new(),
            tag_editor: None,
            rename: None,
            downloading: false,
            batch: Vec::new(),
            window_focused: true,
            cancel: None,
            token,
            status,
            saved_config: Some(config.clone()),
            config,
        }
    }

    /// Signed-in state, derived from the token so the two can never disagree.
    fn signed_in(&self) -> bool {
        self.token.is_some()
    }

    fn settings_dirty(&self) -> bool {
        self.saved_config.as_ref() != Some(&self.config)
    }

    /// The only path that writes the config, so every save — explicit or
    /// implicit — keeps the unsaved-changes snapshot truthful.
    fn persist_config(&mut self) -> qobuz_core::Result<()> {
        self.config.save()?;
        self.saved_config = Some(self.config.clone());
        Ok(())
    }

    /// Persist the config, surfacing a failure instead of dropping it silently.
    fn save_config(&mut self) {
        if let Err(e) = self.persist_config() {
            tracing::warn!("could not save config: {e}");
            self.status = Some(Status::error(format!("Could not save settings: {e}")));
        }
    }

    /// The queue row for `track_id`, if any.
    fn item_mut(&mut self, track_id: i64) -> Option<&mut QueueItem> {
        self.queue.iter_mut().find(|it| it.track_id == track_id)
    }

    fn client(&self) -> Result<QobuzClient, String> {
        let mut c = QobuzClient::new(self.config.app_id.clone(), self.config.app_secret.clone())
            .map_err(|e| e.to_string())?
            .with_secret_candidates(self.config.app_secret_candidates.clone());
        if let Some(t) = &self.token {
            c = c.with_token(t.value.clone());
        }
        Ok(c)
    }

    fn theme(&self) -> Theme {
        style::theme(self.config.dark_mode)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(s) => {
                self.screen = s;
                Task::none()
            }
            Message::Shortcut(s) => self.on_shortcut(s),
            Message::ToggleTheme => {
                self.config.dark_mode = !self.config.dark_mode;
                self.save_config();
                Task::none()
            }
            Message::DismissStatus => {
                self.status = None;
                Task::none()
            }
            Message::ToggleTemplateHelp => {
                self.show_template_help = !self.show_template_help;
                Task::none()
            }
            Message::ToggleCredentialsHelp => {
                self.show_credentials_help = !self.show_credentials_help;
                Task::none()
            }
            Message::ToggleAccountHelp => {
                self.show_account_help = !self.show_account_help;
                Task::none()
            }
            Message::ToggleOptionsHelp => {
                self.show_options_help = !self.show_options_help;
                Task::none()
            }
            Message::CopyTemplate(t) => iced::clipboard::write(t),

            // ---- Settings inputs ----
            Message::TokenChanged(v) => {
                self.token_input = v;
                Task::none()
            }
            Message::AppIdChanged(v) => {
                self.config.app_id = v;
                Task::none()
            }
            Message::AppSecretChanged(v) => {
                self.config.app_secret = v;
                self.secret_manually_edited = true;
                // The manual secret is tried first (it's the primary in `client()`),
                // but keep any auto-detected candidates as fallback. Clearing them
                // stranded the working secret whenever a manual edit didn't happen
                // to match the actual signer — recoverable only by re-detecting.
                Task::none()
            }
            Message::AutoDetectCredentials => {
                self.status = Some(Status::progress(
                    "Detecting credentials from the Qobuz web player…",
                ));
                Task::perform(
                    tasks::auto_detect_credentials(),
                    Message::CredentialsDetected,
                )
            }
            Message::CredentialsDetected(Ok(creds)) => {
                self.config.app_id = creds.app_id;
                // Keep the first candidate as the visible secret; the rest are
                // tried automatically when signing.
                let mut secrets = creds.app_secrets.into_iter();
                self.config.app_secret = secrets.next().unwrap_or_default();
                self.config.app_secret_candidates = secrets.collect();
                // Detected as a set — the working candidate may not be the one we
                // picked as primary; let the signing check adopt it silently.
                self.secret_manually_edited = false;
                self.status = Some(match self.persist_config() {
                    Ok(()) => {
                        Status::success("Credentials detected and saved. You can now sign in.")
                    }
                    Err(e) => {
                        Status::error(format!("Credentials detected but could not save: {e}"))
                    }
                });
                Task::none()
            }
            Message::CredentialsDetected(Err(e)) => {
                self.status = Some(Status::error(format!(
                    "Auto-detect failed: {e}. Enter credentials manually."
                )));
                Task::none()
            }
            Message::CheckSigning => {
                if !self.signed_in() {
                    self.status = Some(Status::error("Sign in before checking signing."));
                    return Task::none();
                }
                match self.client() {
                    Ok(client) => {
                        self.status = Some(Status::progress("Checking request signing…"));
                        Task::perform(tasks::check_signing_probe(client), Message::SigningChecked)
                    }
                    Err(e) => {
                        self.status = Some(Status::error(e));
                        Task::none()
                    }
                }
            }
            Message::SigningChecked(Ok(SigningCheck::Primary)) => {
                self.status = Some(Status::success(
                    "Signing OK — request signatures are being accepted.",
                ));
                Task::none()
            }
            Message::SigningChecked(Ok(SigningCheck::Fallback { working_secret })) => {
                if self.secret_manually_edited {
                    // The user typed a secret that doesn't sign; only a saved
                    // fallback does. Flag it rather than silently overriding.
                    self.status = Some(Status::error(
                        "Entered app_secret is invalid — a saved fallback works; update or \
                         re-detect it.",
                    ));
                } else {
                    // The primary came from auto-detect; adopt the candidate that
                    // actually signs so the check reads cleanly from now on.
                    self.config.promote_secret(&working_secret);
                    self.status = Some(Status::success(
                        "Signing OK — request signatures are being accepted.",
                    ));
                    self.save_config();
                }
                Task::none()
            }
            Message::SigningChecked(Err(e)) => {
                self.status = Some(Status::error(format!("Signing check failed: {e}")));
                Task::none()
            }
            Message::FolderFormatChanged(v) => {
                self.config.folder_format = v;
                Task::none()
            }
            Message::TrackFormatChanged(v) => {
                self.config.track_format = v;
                Task::none()
            }
            Message::RenameFormatChanged(v) => {
                self.config.rename_format = v;
                Task::none()
            }
            Message::ConcurrencyChanged(n) => {
                self.config.concurrency = n;
                Task::none()
            }
            Message::QualitySelected(q) => {
                self.config.quality = q;
                Task::none()
            }
            Message::CoverArtSelected(choice) => {
                match choice {
                    CoverArt::Off => self.config.embed_art = false,
                    CoverArt::Size(size) => {
                        self.config.embed_art = true;
                        self.config.cover_size = size;
                    }
                }
                Task::none()
            }
            Message::NotifyToggled(b) => {
                self.config.notify_on_finish = b;
                Task::none()
            }
            Message::PickDir => Task::perform(tasks::pick_dir(), Message::DirChosen),
            Message::DirChosen(Some(p)) => {
                self.config.download_dir = p;
                Task::none()
            }
            Message::DirChosen(None) => Task::none(),
            Message::SaveSettings => {
                match self.persist_config() {
                    Ok(()) => self.status = Some(Status::success("Settings saved.")),
                    Err(e) => {
                        self.status = Some(Status::error(format!("Could not save settings: {e}")))
                    }
                }
                Task::none()
            }
            Message::LoginToken => {
                if !self.config.has_app_credentials() {
                    self.status = Some(Status::error("Enter app_id and app_secret first."));
                    return Task::none();
                }
                let (id, secret) = (self.config.app_id.clone(), self.config.app_secret.clone());
                let token = self.token_input.trim().to_string();
                if token.is_empty() {
                    self.status = Some(Status::error("Paste a user_auth_token first."));
                    return Task::none();
                }
                self.status = Some(Status::progress("Validating token…"));
                Task::perform(tasks::login_token(id, secret, token), Message::LoggedIn)
            }
            Message::LoggedIn(Ok(token)) => {
                if let Err(e) = auth::store_token(&token) {
                    self.status = Some(Status::error(format!(
                        "Signed in, but token could not be stored: {e}"
                    )));
                } else {
                    self.status = Some(Status::success("Signed in."));
                }
                self.token = Some(StoredToken {
                    value: token,
                    origin: TokenOrigin::ValidatedThisSession,
                });
                self.save_config();
                Task::none()
            }
            Message::LoggedIn(Err(e)) => {
                self.status = Some(Status::error(format!("Sign-in failed: {e}")));
                Task::none()
            }
            Message::SignOut => {
                // Only drop the in-memory token when the keyring copy is
                // actually gone — the displayed state must stay truthful.
                self.status = Some(match auth::clear_token() {
                    Ok(()) => {
                        self.token = None;
                        Status::success("Signed out.")
                    }
                    Err(e) => Status::error(format!(
                        "Sign-out failed: the stored token could not be removed: {e}"
                    )),
                });
                Task::none()
            }

            // ---- Search / add ----
            Message::SearchQueryChanged(v) => {
                self.search_query = v;
                Task::none()
            }
            Message::SearchSubmit => {
                let (q, bare_id) = match omnibox::classify(&self.search_query) {
                    Submit::Empty => return Task::none(),
                    Submit::Add(reference) => {
                        self.offered_id = None;
                        return self.update(Message::Add(reference));
                    }
                    Submit::BadUrl(e) => {
                        self.status = Some(Status::error(e));
                        return Task::none();
                    }
                    Submit::Search { query, bare_id } => (query, bare_id),
                };
                self.offered_id = bare_id;
                self.album = None;
                self.results_offset = scrollable::AbsoluteOffset::default();
                let client = match self.client() {
                    Ok(c) => c,
                    Err(e) => {
                        self.status = Some(Status::error(e));
                        return Task::none();
                    }
                };
                self.status = Some(Status::progress(format!("Searching “{q}”…")));
                self.search_generation += 1;
                let generation = self.search_generation;
                Task::perform(tasks::do_search(client, q), move |r| {
                    Message::SearchDone(generation, r)
                })
            }
            Message::SearchDone(generation, _) if generation != self.search_generation => {
                Task::none()
            }
            Message::SearchDone(_, Ok(payload)) => {
                let n = payload.albums.items.len() + payload.tracks.items.len();
                self.status = Some(match &payload.failure {
                    Some(e) => Status::error(format!("{n} results, but the {e}")),
                    None if n == 0 => Status::info("No results."),
                    None => Status::success(format!("{n} results.")),
                });
                self.results = payload;
                // Without the eviction the cache grows for every cover ever
                // viewed in the session.
                let in_use = self.covers_in_use();
                self.thumbnails.retain(|url, _| in_use.contains(url));
                self.fetch_missing_covers(in_use)
            }
            Message::SearchDone(_, Err(e)) => {
                // The bumped generation drops any "Show more" still in flight
                // for the results left on screen, so nothing else clears it.
                self.results.albums.loading = false;
                self.results.tracks.loading = false;
                self.status = Some(Status::error(format!("Search failed: {e}")));
                Task::none()
            }
            Message::ShowMore(kind) => self.show_more(kind),
            Message::ResultsScrolled(viewport) => {
                self.results_offset = viewport.absolute_offset();
                Task::none()
            }

            // ---- Album detail ----
            Message::OpenAlbum(header) => {
                self.album = Some(AlbumDetail::loading(header));
                self.load_album()
            }
            Message::RetryAlbum => {
                if let Some(album) = &mut self.album {
                    album.state = DetailState::Loading;
                }
                self.load_album()
            }
            Message::AlbumLoaded(id, result) => {
                let Some(album) = self.album.as_mut().filter(|a| a.id() == id) else {
                    return Task::none();
                };
                match result {
                    Ok(jobs) => album.loaded(jobs),
                    Err(e) => album.state = DetailState::Failed(e),
                }
                Task::none()
            }
            Message::CloseAlbum => {
                self.album = None;
                // The results subtree is rebuilt when the detail closes, which
                // resets its scroll; put it back where the user left it.
                scrollable::scroll_to(view::search::results_id(), self.results_offset)
            }
            Message::ToggleTrack(track_id) => {
                if let Some(album) = &mut self.album {
                    album.toggle(track_id);
                }
                Task::none()
            }
            Message::SelectAllTracks => {
                if let Some(album) = &mut self.album {
                    album.select_all();
                }
                Task::none()
            }
            Message::SelectNoTracks => {
                if let Some(album) = &mut self.album {
                    album.select_none();
                }
                Task::none()
            }
            Message::AddSelected => {
                let Some(album) = &self.album else {
                    return Task::none();
                };
                let jobs = album::selected_jobs(album.jobs(), &album.selected);
                if jobs.is_empty() {
                    return Task::none();
                }
                self.update(Message::Resolved(Ok(jobs)))
            }
            Message::MoreAlbums(generation, _) | Message::MoreTracks(generation, _)
                if generation != self.search_generation =>
            {
                Task::none()
            }
            Message::MoreAlbums(_, result) => {
                let covers = more_loaded(
                    &mut self.results.albums,
                    result,
                    |a| &a.id,
                    |a| a.cover.as_ref(),
                );
                self.after_more(Kind::Albums, covers)
            }
            Message::MoreTracks(_, result) => {
                let covers = more_loaded(
                    &mut self.results.tracks,
                    result,
                    |t| &t.id,
                    |t| t.cover.as_ref(),
                );
                self.after_more(Kind::Tracks, covers)
            }
            Message::ThumbnailLoaded(url, Ok(bytes)) => {
                // Released so the cover can be fetched again if an eviction
                // later drops it from the cache.
                self.cover_requests.remove(&url);
                self.thumbnails
                    .insert(url, iced::widget::image::Handle::from_bytes(bytes));
                Task::none()
            }
            Message::ThumbnailLoaded(_, Err(())) => Task::none(),
            Message::Add(reference) => {
                let client = match self.client() {
                    Ok(c) => c,
                    Err(e) => {
                        self.status = Some(Status::error(e));
                        return Task::none();
                    }
                };
                self.status = Some(Status::progress(format!("Resolving {}…", reference.kind())));
                Task::perform(tasks::resolve(client, reference), Message::Resolved)
            }
            Message::Resolved(Ok(jobs)) => {
                let mut added = 0;
                for job in jobs {
                    let track_id = job.track.id;
                    if self.queue.iter().any(|it| it.track_id == track_id) {
                        continue;
                    }
                    self.queue.push(QueueItem {
                        track_id,
                        job,
                        status: ItemStatus::Queued,
                        downloaded: 0,
                        total: None,
                        path: None,
                    });
                    added += 1;
                }
                self.status = Some(Status::success(format!(
                    "Added {added} track(s) to the queue."
                )));
                self.screen = Screen::Queue;
                self.fetch_missing_covers(self.queue_covers())
            }
            Message::Resolved(Err(e)) => {
                self.status = Some(Status::error(format!("Could not resolve: {e}")));
                Task::none()
            }

            // ---- Downloads ----
            Message::StartDownloads => {
                // Only tracks that have never been attempted — relaunching a
                // failed one is the Retry controls' job. Matches `startable`,
                // which decides whether the button is offered at all.
                let jobs = self.jobs_with(|s| matches!(s, ItemStatus::Queued));
                if jobs.is_empty() {
                    // Unreachable from the button, which hides itself in this
                    // state, but the message can still arrive.
                    self.status = Some(Status::info("Nothing queued to download."));
                    return Task::none();
                }
                self.spawn_downloads(jobs)
            }
            Message::CancelDownloads => {
                // A request, not a state change: the batch still ends through
                // its normal completion path, so `DownloadsFinished` stays the
                // one place `downloading` is cleared and the queue is left for
                // the per-track `Cancelled` events to requeue.
                if let Some(cancel) = &self.cancel {
                    cancel.cancel();
                    self.status = Some(Status::progress("Cancelling…"));
                }
                Task::none()
            }
            Message::RetryFailed => {
                let jobs = self.jobs_with(|s| matches!(s, ItemStatus::Error(_)));
                if jobs.is_empty() {
                    return Task::none();
                }
                self.spawn_downloads(jobs)
            }
            Message::ClearQueue => {
                self.queue.clear();
                self.collapsed.clear();
                self.rename = None;
                self.status = Some(Status::info("Queue cleared."));
                Task::none()
            }
            Message::ToggleGroup(album_id) => {
                if !self.collapsed.remove(&album_id) {
                    self.collapsed.insert(album_id);
                }
                Task::none()
            }
            Message::RemoveGroup(album_id) => {
                self.remove_where(|it| it.job.album.id == album_id);
                Task::none()
            }
            Message::OpenAlbumFolder(album_id) => match self.album_folder(&album_id) {
                Some(folder) => open_folder(folder),
                None => Task::none(),
            },
            Message::OpenDownloadDir => open_folder(self.config.download_dir.clone()),
            Message::Opened(Ok(())) => Task::none(),
            Message::Opened(Err(e)) => {
                self.status = Some(Status::error(e));
                Task::none()
            }
            Message::Download(ev) => {
                self.apply_event(ev);
                Task::none()
            }
            Message::DownloadsFinished(working_secret) => {
                self.downloading = false;
                // Requeued rows are the evidence that cancelling actually cut
                // work short. The token alone isn't enough: a Cancel pressed
                // after the engine finished but before this message is handled
                // still flips it, and reporting that batch as cancelled would
                // be a lie — nothing was stopped. Only the batch's own rows
                // count: tracks added mid-batch are queued without being cut.
                let requeued = self.queue.iter().any(|it| {
                    self.batch.contains(&it.track_id) && matches!(it.status, ItemStatus::Queued)
                });
                let was_cancelled =
                    self.cancel.take().is_some_and(|c| c.is_cancelled()) && requeued;
                // Persist the secret that actually signed so the next session
                // starts from the known-good one instead of re-probing.
                if let Some(secret) = working_secret {
                    if secret != self.config.app_secret {
                        self.config.promote_secret(&secret);
                        self.save_config();
                    }
                }
                // Counted over the batch's own tracks, like the notification,
                // so failures left from earlier batches don't show up here.
                let outcome = self.batch_outcome(was_cancelled);
                let errors = outcome.failed;
                self.status = Some(if was_cancelled {
                    // A track can finish between the click and the stop, so
                    // report what actually completed rather than what was
                    // showing when Cancel was pressed.
                    let mut s = format!(
                        "Download cancelled — {} of {} completed",
                        outcome.downloaded,
                        self.batch.len()
                    );
                    // Failures that happened before the cancel are still worth
                    // surfacing; the Retry failed control is keyed off them.
                    if errors > 0 {
                        s.push_str(&format!(", {errors} error(s)"));
                    }
                    s.push('.');
                    Status::info(s)
                } else if errors == 0 {
                    Status::success("All downloads finished.")
                } else {
                    Status::error(format!("Downloads finished with {errors} error(s)."))
                });
                match self.finish_notification(was_cancelled) {
                    Some((summary, body)) => {
                        Task::perform(tasks::notify(summary, body), |()| Message::Notified)
                    }
                    None => Task::none(),
                }
            }
            Message::WindowFocus(focused) => {
                self.window_focused = focused;
                Task::none()
            }
            Message::Notified => Task::none(),
            Message::EditTags(album_id) => self.open_tag_editor(album_id),
            Message::TagsRead(album_id, result) => {
                if self.tag_editor.as_ref().map(EditorSlot::album_id) != Some(&album_id) {
                    return Task::none();
                }
                match result {
                    Ok(read) => {
                        self.tag_editor =
                            Some(EditorSlot::Open(Box::new(TagEditor::new(album_id, read))));
                    }
                    Err(e) => {
                        self.tag_editor = None;
                        self.status = Some(Status::error(format!("Could not read tags: {e}")));
                    }
                }
                Task::none()
            }
            Message::Editor(edit) => {
                if let Some(editor) = self.open_editor_mut() {
                    if editor.saving.is_none() {
                        editor.apply(edit);
                    }
                }
                Task::none()
            }
            Message::PickCover => Task::perform(tasks::pick_cover(), Message::CoverPicked),
            Message::CoverPicked(Ok(Some(image))) => {
                let size = self.config.cover_size;
                // A save in progress ends by reloading the files, which would
                // drop a cover picked meanwhile without saving it.
                if let Some(editor) = self.open_editor_mut().filter(|e| e.saving.is_none()) {
                    editor.replace_cover(image, size);
                }
                Task::none()
            }
            Message::CoverPicked(Ok(None)) => Task::none(),
            Message::CoverPicked(Err(e)) => {
                self.status = Some(Status::error(e));
                Task::none()
            }
            Message::SaveTags => {
                if !self.can_save_tags() {
                    return Task::none();
                }
                let Some(editor) = self.open_editor_mut() else {
                    return Task::none();
                };
                let cover = editor.start_saving();
                Task::perform(tasks::plan_cover(cover), Message::CoverPlanned)
            }
            Message::CoverPlanned(result) => {
                let Some(editor) = self.open_editor_mut() else {
                    return Task::none();
                };
                match result {
                    Ok(plan) => {
                        let next = editor.cover_planned(plan);
                        self.save_next(next)
                    }
                    Err(e) => {
                        editor.finish_saving();
                        self.status =
                            Some(Status::error(format!("Could not prepare the cover: {e}")));
                        Task::none()
                    }
                }
            }
            Message::TagsSaved(track_id, result) => {
                let Some(editor) = self.open_editor_mut() else {
                    return Task::none();
                };
                let next = editor.file_saved(track_id, result);
                self.save_next(next)
            }
            Message::CloseTagEditor => {
                let saving = self
                    .open_editor()
                    .is_some_and(|editor| editor.saving.is_some());
                if !saving {
                    self.tag_editor = None;
                }
                Task::none()
            }
            Message::RenameFolder(album_id) => self.open_rename(album_id),
            Message::RenameSuggested(album_id, result) => {
                let Some(rename) = self
                    .rename
                    .as_mut()
                    .filter(|r| r.album_id == album_id && r.suggesting)
                else {
                    return Task::none();
                };
                rename.suggesting = false;
                match result {
                    Ok(Some(name)) => rename.name = name,
                    Ok(None) => {}
                    Err(e) => {
                        self.status = Some(Status::error(format!("Could not suggest a name: {e}")))
                    }
                }
                Task::none()
            }
            Message::RenameNameChanged(name) => {
                if let Some(rename) = &mut self.rename {
                    rename.name = name;
                    rename.suggesting = false;
                }
                Task::none()
            }
            Message::ConfirmRename => self.confirm_rename(),
            Message::CancelRename => {
                self.rename = None;
                Task::none()
            }
            Message::FolderRenamed(old, Ok(new)) => {
                rewrite_paths(&mut self.queue, &old, &new);
                let name = new.file_name().unwrap_or_default().to_string_lossy();
                self.status = Some(Status::success(format!(
                    "Renamed the folder to \"{name}\"."
                )));
                Task::none()
            }
            Message::FolderRenamed(_, Err(e)) => {
                self.status = Some(Status::error(format!("Could not rename the folder: {e}")));
                Task::none()
            }
        }
    }

    /// Open an album group's Rename folder field on the folder's current name,
    /// and read the rename template's suggestion from its first track.
    fn open_rename(&mut self, album_id: String) -> Task<Message> {
        if !self.renamable(&album_id) {
            return Task::none();
        }
        let (Some(folder), Some(file)) = (
            self.rename_target(&album_id),
            self.first_done_file(&album_id),
        ) else {
            return Task::none();
        };
        self.rename = Some(RenameState {
            album_id: album_id.clone(),
            name: folder
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned(),
            suggesting: true,
        });
        let template = self.config.rename_format.clone();
        Task::perform(tasks::suggest_folder_name(template, file), move |r| {
            Message::RenameSuggested(album_id.clone(), r)
        })
    }

    /// Rename the open field's folder, unless the name is empty, the album
    /// can't be renamed now, or another album's files share the folder.
    fn confirm_rename(&mut self) -> Task<Message> {
        let Some(rename) = &self.rename else {
            return Task::none();
        };
        if rename::folder_name(&rename.name).is_none() || !self.renamable(&rename.album_id) {
            return Task::none();
        }
        let Some(folder) = self.rename_target(&rename.album_id) else {
            return Task::none();
        };
        if self.shares_folder(&rename.album_id, &folder) {
            self.status = Some(Status::error(
                "Could not rename the folder: another album in the queue has files in it.",
            ));
            return Task::none();
        }
        let files: Vec<PathBuf> = self
            .done_items(&rename.album_id)
            .filter_map(|it| it.path.clone())
            .collect();
        let name = self.rename.take().map(|r| r.name).unwrap_or_default();
        self.status = Some(Status::progress("Renaming the folder…"));
        let download_dir = self.config.download_dir.clone();
        let old = folder.clone();
        Task::perform(
            tasks::rename_folder(download_dir, folder, files, name),
            move |result| Message::FolderRenamed(old.clone(), result),
        )
    }

    /// The folder Rename folder acts on: the album's shared folder, or the one
    /// above its `Disc N` folder when only one disc of a multi-disc album is done.
    fn rename_target(&self, album_id: &str) -> Option<PathBuf> {
        let folder = self.album_folder(album_id)?;
        let mut done = self.done_items(album_id).peekable();
        let multi_disc = done.peek().is_some_and(|it| it.job.multi_disc);
        let one_disc = done.all(|it| it.path.as_deref().and_then(Path::parent) == Some(&folder));
        if multi_disc && one_disc {
            return folder.parent().map(Path::to_path_buf);
        }
        Some(folder)
    }

    /// The album's done file that comes first in disc and track order.
    fn first_done_file(&self, album_id: &str) -> Option<PathBuf> {
        self.done_items(album_id)
            .filter(|it| it.path.is_some())
            .min_by_key(|it| (it.job.track.disc_number(), it.job.track.track_number))
            .and_then(|it| it.path.clone())
    }

    /// Whether another album's done files lie inside `folder`, so renaming it
    /// would move them too.
    fn shares_folder(&self, album_id: &str, folder: &Path) -> bool {
        self.queue.iter().any(|it| {
            it.job.album.id != album_id
                && matches!(it.status, ItemStatus::Done(_))
                && it.path.as_deref().is_some_and(|p| p.starts_with(folder))
        })
    }

    /// Whether an album's folder can be renamed now: as for
    /// [`App::tags_editable`], but with no download running anywhere, since
    /// another album may be writing into the same folder, and no failed track,
    /// whose retry would land under the old folder name.
    fn renamable(&self, album_id: &str) -> bool {
        let items: Vec<&QueueItem> = self
            .queue
            .iter()
            .filter(|it| it.job.album.id == album_id)
            .collect();
        self.group_renamable(&items)
    }

    /// [`App::renamable`] over one album's own tracks, for the Queue view.
    fn group_renamable(&self, items: &[&QueueItem]) -> bool {
        !self.downloading
            && !items
                .iter()
                .any(|it| matches!(it.status, ItemStatus::Error(_)))
            && self.group_editable(items.iter().copied())
    }

    /// Close the Rename folder field once its album can no longer be renamed,
    /// so it doesn't linger with a Rename control that does nothing.
    fn drop_stale_rename(&mut self) {
        if self
            .rename
            .as_ref()
            .is_some_and(|r| !self.renamable(&r.album_id))
        {
            self.rename = None;
        }
    }

    fn done_items<'a>(&'a self, album_id: &'a str) -> impl Iterator<Item = &'a QueueItem> + 'a {
        self.queue
            .iter()
            .filter(move |it| it.job.album.id == album_id)
            .filter(|it| matches!(it.status, ItemStatus::Done(_)))
    }

    /// Whether an album's tags can be edited now: it has a done track, and no
    /// track still queued, downloading or tagging, or in the running batch.
    fn tags_editable(&self, album_id: &str) -> bool {
        self.group_editable(self.queue.iter().filter(|it| it.job.album.id == album_id))
    }

    /// [`App::tags_editable`] over one album's own tracks, so the Queue view
    /// can decide per group without scanning the whole queue again.
    fn group_editable<'a>(&self, items: impl IntoIterator<Item = &'a QueueItem>) -> bool {
        let mut any_done = false;
        for it in items {
            let in_batch = self.downloading && self.batch.contains(&it.track_id);
            match it.status {
                ItemStatus::Done(_) if !in_batch => any_done = true,
                ItemStatus::Error(_) if !in_batch => {}
                _ => return false,
            }
        }
        any_done
    }

    fn open_tag_editor(&mut self, album_id: String) -> Task<Message> {
        if !self.tags_editable(&album_id) {
            return Task::none();
        }
        let files: Vec<(Job, PathBuf)> = self
            .queue
            .iter()
            .filter(|it| it.job.album.id == album_id && matches!(it.status, ItemStatus::Done(_)))
            .filter_map(|it| Some((it.job.clone(), it.path.clone()?)))
            .collect();
        self.tag_editor = Some(EditorSlot::Loading(album_id.clone()));
        self.rename = None;
        Task::perform(tasks::read_tags(files), move |read| {
            Message::TagsRead(album_id.clone(), read)
        })
    }

    fn open_editor(&self) -> Option<&TagEditor> {
        match &self.tag_editor {
            Some(EditorSlot::Open(editor)) => Some(&**editor),
            _ => None,
        }
    }

    fn open_editor_mut(&mut self) -> Option<&mut TagEditor> {
        match &mut self.tag_editor {
            Some(EditorSlot::Open(editor)) => Some(&mut **editor),
            _ => None,
        }
    }

    /// Whether Save can run: there are valid edits, no save is under way, and
    /// the album hasn't gone back into a download since the editor opened.
    fn can_save_tags(&self) -> bool {
        self.open_editor()
            .is_some_and(|editor| editor.has_edits() && self.save_ready(editor))
    }

    /// Everything [`App::can_save_tags`] checks but whether there are edits,
    /// for a view that has already worked that out.
    fn save_ready(&self, editor: &TagEditor) -> bool {
        editor.saving.is_none() && !editor.has_invalid() && self.tags_editable(&editor.album_id)
    }

    /// Save the next file, or report the finished save and read the files
    /// back, so the editor shows what they now hold.
    fn save_next(&mut self, next: Option<tag_editor::NextSave>) -> Task<Message> {
        if let Some((track_id, path, edits, plan)) = next {
            return Task::perform(tasks::save_tags(path, edits, plan), move |result| {
                Message::TagsSaved(track_id, result)
            });
        }
        let Some(report) = self.open_editor_mut().and_then(TagEditor::finish_saving) else {
            return Task::none();
        };
        let saved = match report.written {
            1 => "Saved 1 file".to_owned(),
            n => format!("Saved {n} files"),
        };
        if !report.failures.is_empty() {
            // The edits stay as typed, so Save retries them once the problem
            // is fixed; files already saved then count as unchanged.
            self.status = Some(Status::error(format!(
                "{saved}; {} failed — {}. Save again to retry.",
                report.failures.len(),
                report.failures.join("; ")
            )));
            return Task::none();
        }
        let album_id = self
            .tag_editor
            .as_ref()
            .map(|slot| slot.album_id().to_owned())
            .unwrap_or_default();
        if !self.tags_editable(&album_id) {
            // Re-queued meanwhile: its files may be rewritten by a download,
            // so the editor can't show what they hold.
            self.tag_editor = None;
            self.status = Some(Status::info(format!(
                "{saved}. The album is back in the queue, so the editor closed."
            )));
            return Task::none();
        }
        self.status = Some(if report.written == 0 {
            Status::info("Nothing to save: the files already hold these values.")
        } else {
            Status::success(format!("{saved}."))
        });
        self.open_tag_editor(album_id)
    }

    /// The folder an album group's downloaded files share, once any is done.
    fn album_folder(&self, album_id: &str) -> Option<PathBuf> {
        open::shared_folder(
            self.done_items(album_id)
                .filter_map(|it| it.path.as_deref()),
        )
    }

    /// Act on a keyboard shortcut. The album-detail keys only apply where that
    /// detail is on screen, so Escape on another tab does nothing.
    fn on_shortcut(&mut self, s: shortcut::Shortcut) -> Task<Message> {
        use shortcut::Shortcut;
        // The setup prompt replaces the album on screen while setup is missing.
        let album_on_screen = self.screen == Screen::Search
            && self.album.is_some()
            && view::search::setup_gap(&self.config, self.signed_in()).is_none();
        match s {
            // Not from Settings: its number field leaves a rejected `/`
            // unclaimed, so the press can't be told from one in no field.
            Shortcut::FocusSearch if self.screen != Screen::Settings => {
                self.screen = Screen::Search;
                iced::widget::text_input::focus(view::search::search_input_id())
            }
            Shortcut::FocusSearch => Task::none(),
            Shortcut::NextTab => self.update(Message::Navigate(shortcut::next_tab(self.screen))),
            Shortcut::PreviousTab => {
                self.update(Message::Navigate(shortcut::previous_tab(self.screen)))
            }
            Shortcut::Escape if album_on_screen => self.update(Message::CloseAlbum),
            Shortcut::AddSelected if album_on_screen => self.update(Message::AddSelected),
            Shortcut::Escape | Shortcut::AddSelected => Task::none(),
        }
    }

    /// What the queue holds, indexed once per render for the result rows.
    fn queued(&self) -> Queued<'_> {
        Queued {
            tracks: self.queue.iter().map(|it| it.track_id).collect(),
            albums: self
                .queue
                .iter()
                .map(|it| it.job.album.id.as_str())
                .collect(),
        }
    }

    /// Whether a track can be taken out of the queue list now: it is settled
    /// and not part of a running batch, where a queued track may be about to
    /// start and the end-of-batch status counts the batch's rows. Tracks added
    /// after the batch started stay removable.
    fn can_remove(&self, it: &QueueItem) -> bool {
        removable(&it.status) && !(self.downloading && self.batch.contains(&it.track_id))
    }

    /// Take the matching removable tracks out of the queue list; files on disk
    /// are never touched. An album left with no rows also leaves `collapsed`,
    /// so adding it again shows it expanded.
    fn remove_where(&mut self, matches: impl Fn(&QueueItem) -> bool) {
        let before = self.queue.len();
        let doomed: Vec<i64> = self
            .queue
            .iter()
            .filter(|it| matches(it) && self.can_remove(it))
            .map(|it| it.track_id)
            .collect();
        self.queue.retain(|it| !doomed.contains(&it.track_id));
        self.drop_stale_rename();
        let queue = &self.queue;
        self.collapsed
            .retain(|id| queue.iter().any(|it| &it.job.album.id == id));
        let removed = before - self.queue.len();
        if removed > 0 {
            self.status = Some(Status::info(format!(
                "Removed {removed} track(s) from the queue."
            )));
        }
    }

    /// How the running (or last) batch's own tracks ended.
    fn batch_outcome(&self, cancelled: bool) -> notify::BatchOutcome {
        let in_batch = |pred: fn(&ItemStatus) -> bool| {
            self.queue
                .iter()
                .filter(|it| self.batch.contains(&it.track_id) && pred(&it.status))
                .count()
        };
        notify::BatchOutcome {
            downloaded: in_batch(|s| matches!(s, ItemStatus::Done(_))),
            failed: in_batch(|s| matches!(s, ItemStatus::Error(_))),
            cancelled,
        }
    }

    /// The desktop notification for the batch that just ended, if one is due.
    fn finish_notification(&self, cancelled: bool) -> Option<(String, String)> {
        notify::notification(
            self.batch_outcome(cancelled),
            self.config.notify_on_finish,
            self.window_focused,
        )
    }

    /// Window focus, which decides whether a finished batch notifies, and key
    /// presses for shortcuts. Raw events rather than `keyboard::on_key_press`,
    /// which drops presses a focused text field captured.
    fn subscription(&self) -> iced::Subscription<Message> {
        iced::event::listen_with(|event, status, _window| match event {
            iced::Event::Window(iced::window::Event::Focused) => Some(Message::WindowFocus(true)),
            iced::Event::Window(iced::window::Event::Unfocused) => {
                Some(Message::WindowFocus(false))
            }
            iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                shortcut::shortcut(&key, modifiers, status).map(Message::Shortcut)
            }
            _ => None,
        })
    }

    /// Fetch the open album's tracks. Built on `resolve`, so the jobs it
    /// returns can be enqueued as they are, without fetching the album again.
    fn load_album(&mut self) -> Task<Message> {
        let Some(id) = self.album.as_ref().map(|a| a.id().to_owned()) else {
            return Task::none();
        };
        let client = match self.client() {
            Ok(c) => c,
            Err(e) => {
                if let Some(album) = &mut self.album {
                    album.state = DetailState::Failed(e);
                }
                return Task::none();
            }
        };
        let reference = Reference::Album(id.clone());
        Task::perform(tasks::resolve(client, reference), move |r| {
            Message::AlbumLoaded(id.clone(), r)
        })
    }

    /// Request the next page of one results section, unless one is already in
    /// flight or none remains.
    fn show_more(&mut self, kind: Kind) -> Task<Message> {
        let (loading, has_more, offset) = match kind {
            Kind::Albums => {
                let s = &self.results.albums;
                (s.loading, s.has_more(), s.next_offset())
            }
            Kind::Tracks => {
                let s = &self.results.tracks;
                (s.loading, s.has_more(), s.next_offset())
            }
        };
        if loading || !has_more {
            return Task::none();
        }
        let client = match self.client() {
            Ok(c) => c,
            Err(e) => {
                self.status = Some(Status::error(e));
                return Task::none();
            }
        };
        let query = self.results.query.clone();
        let generation = self.search_generation;
        match kind {
            Kind::Albums => {
                self.results.albums.loading = true;
                Task::perform(tasks::more_albums(client, query, offset), move |r| {
                    Message::MoreAlbums(generation, r)
                })
            }
            Kind::Tracks => {
                self.results.tracks.loading = true;
                Task::perform(tasks::more_tracks(client, query, offset), move |r| {
                    Message::MoreTracks(generation, r)
                })
            }
        }
    }

    /// Report a failed page, or fetch the covers an appended page brought in.
    fn after_more(&mut self, kind: Kind, covers: Result<Vec<String>, String>) -> Task<Message> {
        match covers {
            Ok(urls) => self.fetch_missing_covers(urls),
            Err(e) => {
                let what = match kind {
                    Kind::Albums => "albums",
                    Kind::Tracks => "tracks",
                };
                self.status = Some(Status::error(format!("Could not load more {what}: {e}")));
                Task::none()
            }
        }
    }

    /// The cover of every album with a group in the queue.
    fn queue_covers(&self) -> Vec<String> {
        let mut covers: Vec<String> = Vec::new();
        for url in self
            .queue
            .iter()
            .filter_map(|it| tasks::thumbnail(it.job.album.image.as_ref()))
        {
            if !covers.contains(&url) {
                covers.push(url);
            }
        }
        covers
    }

    /// Every cover a view can show right now: search results, the open album,
    /// and the queue's group headers. Evicting anything outside this set never
    /// blanks a visible cover, whichever view reads the cache.
    fn covers_in_use(&self) -> HashSet<String> {
        let results = &self.results;
        results
            .albums
            .items
            .iter()
            .filter_map(|a| a.cover.clone())
            .chain(results.tracks.items.iter().filter_map(|t| t.cover.clone()))
            .chain(self.album.as_ref().and_then(|a| a.header.cover.clone()))
            .chain(self.queue_covers())
            .collect()
    }

    /// Lazily load cover thumbnails not already cached or requested.
    fn fetch_missing_covers(&mut self, urls: impl IntoIterator<Item = String>) -> Task<Message> {
        let fetches: Vec<Task<Message>> = urls
            .into_iter()
            .filter(|url| !self.thumbnails.contains_key(url))
            .filter(|url| self.cover_requests.insert(url.clone()))
            .map(|url| {
                Task::perform(tasks::fetch_thumbnail(url.clone()), move |res| {
                    Message::ThumbnailLoaded(url.clone(), res)
                })
            })
            .collect();
        Task::batch(fetches)
    }

    /// Whether a cancel has been requested and the batch is still winding down.
    /// The Queue header uses it to show "Cancelling…" and stop a second press.
    fn cancelling(&self) -> bool {
        self.cancel.as_ref().is_some_and(|c| c.is_cancelled())
    }

    /// Clone the jobs of every queue row whose status matches `pred`.
    fn jobs_with(&self, pred: impl Fn(&ItemStatus) -> bool) -> Vec<Job> {
        self.queue
            .iter()
            .filter(|it| pred(&it.status))
            .map(|it| it.job.clone())
            .collect()
    }

    /// Launch a download batch for `jobs`, bridging core `JobEvent`s into
    /// `Message::Download`. Resets the targeted rows to queued once the batch
    /// actually starts, so a relaunched track's error badge clears.
    fn spawn_downloads(&mut self, jobs: Vec<Job>) -> Task<Message> {
        if jobs.is_empty() || self.downloading {
            return Task::none();
        }
        if !self.signed_in() {
            self.status = Some(Status::error("Sign in before downloading."));
            return Task::none();
        }
        let client = match self.client() {
            Ok(c) => c,
            Err(e) => {
                self.status = Some(Status::error(e));
                return Task::none();
            }
        };
        for job in &jobs {
            if let Some(it) = self.item_mut(job.track.id) {
                it.status = ItemStatus::Queued;
                it.downloaded = 0;
                it.total = None;
                it.path = None;
            }
        }
        let config = self.config.clone();
        self.downloading = true;
        self.drop_stale_rename();
        self.batch = jobs.iter().map(|job| job.track.id).collect();
        self.status = Some(Status::progress(format!(
            "Downloading {} track(s)…",
            jobs.len()
        )));
        // Fresh per batch — a reused token would already be cancelled.
        let cancel = CancellationToken::new();
        self.cancel = Some(cancel.clone());

        let stream = iced::stream::channel(256, move |mut output| async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<JobEvent>(256);
            // The engine clones the client internally; a retained clone shares
            // the `working_secret` cache, so we can read which secret signed
            // once the batch completes.
            let probe = client.clone();
            let engine = engine::download_all(client, config, jobs, tx, cancel);
            let drain = async {
                while let Some(ev) = rx.recv().await {
                    let _ = output.send(Message::Download(ev)).await;
                }
            };
            future::join(engine, drain).await;
            let _ = output
                .send(Message::DownloadsFinished(probe.working_secret()))
                .await;
        });
        Task::run(stream, |m| m)
    }

    fn apply_event(&mut self, ev: JobEvent) {
        let track_id = match &ev {
            JobEvent::Started { track_id, .. }
            | JobEvent::Progress { track_id, .. }
            | JobEvent::Tagging { track_id }
            | JobEvent::Done { track_id, .. }
            | JobEvent::Failed { track_id, .. }
            | JobEvent::Cancelled { track_id } => *track_id,
        };
        let Some(item) = self.item_mut(track_id) else {
            return;
        };
        match ev {
            JobEvent::Started { .. } => item.status = ItemStatus::Downloading,
            JobEvent::Progress {
                downloaded, total, ..
            } => {
                item.status = ItemStatus::Downloading;
                item.downloaded = downloaded;
                item.total = total;
            }
            JobEvent::Tagging { .. } => item.status = ItemStatus::Tagging,
            JobEvent::Done {
                delivered, path, ..
            } => {
                item.status = ItemStatus::Done(delivered);
                item.path = Some(path);
            }
            JobEvent::Failed { error, .. } => item.status = ItemStatus::Error(error),
            // Back to queued with its progress discarded, so the partial file
            // (already deleted by the engine) is re-fetched from scratch and
            // `startable` offers Start again to resume the batch.
            JobEvent::Cancelled { .. } => {
                item.status = ItemStatus::Queued;
                item.downloaded = 0;
                item.total = None;
                item.path = None;
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let a = style::accents(&self.theme());
        let wordmark = row![
            text("Qobuz")
                .size(style::TEXT_TITLE)
                .font(view::bold())
                .color(a.brand()),
            text("dl")
                .size(style::TEXT_TITLE)
                .font(view::bold())
                .color(a.text),
        ]
        .spacing(2);
        let signed_in = self.signed_in();
        let header = row![
            wordmark.width(Length::Fill),
            // No ☀/☾ glyphs: the bundled Inter has no ☾, so the pair can't match.
            compact_button(if self.config.dark_mode {
                "Light"
            } else {
                "Dark"
            })
            .width(Length::Fixed(THEME_TOGGLE_WIDTH))
            .on_press(Message::ToggleTheme),
            text(if signed_in {
                "●  signed in"
            } else {
                "○  signed out"
            })
            .size(style::TEXT_SM)
            .color(if signed_in { a.success() } else { a.error() }),
        ]
        .spacing(style::SPACE_MD)
        .align_y(iced::Alignment::Center);

        let tabs = Tabs::new(Message::Navigate)
            .push(
                Screen::Search,
                TabLabel::Text("Search".to_owned()),
                tab_pane(view::search::search_view(self)),
            )
            .push(
                Screen::Queue,
                TabLabel::Text(queue_tab_label(&self.queue)),
                tab_pane(view::queue::queue_view(self)),
            )
            .push(
                Screen::Settings,
                TabLabel::Text("Settings".to_owned()),
                tab_pane(view::settings::settings_view(self)),
            )
            .set_active_tab(&self.screen)
            .tab_bar_style(style::tab_bar)
            .tab_label_padding([style::SPACE_SM as f32, style::SPACE_LG as f32])
            .tab_label_spacing(style::SPACE_XS as f32)
            .text_size(style::TEXT_BODY as f32)
            .height(Length::Fill);

        let content = column![header, view::status_bar(self.status.as_ref()), tabs]
            .spacing(style::SPACE_LG)
            .padding(style::SPACE_XL);

        content.into()
    }
}

/// Places a tab's content below the tab bar. iced_aw's `Tabs` has no content
/// spacing, so the gap is a top padding here.
fn tab_pane<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .padding(iced::Padding {
            top: style::SPACE_LG as f32,
            ..iced::Padding::ZERO
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::from_parts(Config::default(), None, None)
    }

    #[test]
    fn dismiss_clears_an_error() {
        let mut app = app();
        app.status = Some(Status::error("boom"));
        let _ = app.update(Message::DismissStatus);
        assert_eq!(app.status, None);
    }

    #[test]
    fn loaded_settings_are_not_dirty() {
        assert!(!app().settings_dirty());
    }

    #[test]
    fn unreadable_config_stays_saveable() {
        let mut app = app();
        app.saved_config = None;
        assert!(app.settings_dirty());
    }

    #[test]
    fn editing_a_setting_makes_it_dirty() {
        let mut app = app();
        let _ = app.update(Message::QualitySelected(Quality::Mp3));
        assert!(app.settings_dirty());
    }

    #[test]
    fn cover_art_off_keeps_the_size() {
        use qobuz_core::CoverSize;

        let mut app = app();
        let _ = app.update(Message::CoverArtSelected(CoverArt::Size(CoverSize::Px400)));
        assert!(app.config.embed_art);
        assert_eq!(app.config.cover_size, CoverSize::Px400);
        assert!(app.settings_dirty());

        let _ = app.update(Message::CoverArtSelected(CoverArt::Off));
        assert!(!app.config.embed_art);
        assert_eq!(app.config.cover_size, CoverSize::Px400);
        assert_eq!(CoverArt::of(&app.config), CoverArt::Off);
    }

    #[test]
    fn reverting_an_edit_clears_dirty() {
        let mut app = app();
        let original = app.config.track_format.clone();
        let _ = app.update(Message::TrackFormatChanged("{title}".into()));
        let _ = app.update(Message::TrackFormatChanged(original));
        assert!(!app.settings_dirty());
    }

    #[test]
    fn bare_id_query_is_offered_not_enqueued() {
        let mut app = app();
        app.search_query = "1989".into();
        let _ = app.update(Message::SearchSubmit);
        assert_eq!(app.offered_id, Some(Reference::Album("1989".into())));
        assert!(app.queue.is_empty());
    }

    #[test]
    fn foreign_url_reports_an_error() {
        let mut app = app();
        app.search_query = "https://example.com/album/123".into();
        let _ = app.update(Message::SearchSubmit);
        assert_eq!(app.status.map(|s| s.kind), Some(status::StatusKind::Error));
        assert_eq!(app.offered_id, None);
    }

    #[test]
    fn app_opens_on_search() {
        assert_eq!(app().screen, Screen::Search);
    }

    fn album(id: &str) -> AlbumResult {
        AlbumResult {
            id: id.into(),
            title: id.into(),
            artist: "a".into(),
            cover: None,
            hires: false,
        }
    }

    fn track(id: &str) -> TrackResult {
        TrackResult {
            id: id.into(),
            title: id.into(),
            artist: "a".into(),
            cover: None,
            hires: false,
        }
    }

    fn albums(ids: &[&str], total: Option<u32>) -> Page<AlbumResult> {
        Page {
            items: ids.iter().map(|id| album(id)).collect(),
            total,
        }
    }

    /// An app showing results for search generation 1, with albums loading.
    fn app_with_results() -> App {
        let mut app = app();
        app.search_generation = 1;
        app.results = SearchPayload {
            query: "q".into(),
            albums: Section::first(albums(&["a1", "a2"], Some(10))),
            tracks: Section::first(Page {
                items: vec![track("t1")],
                total: Some(1),
            }),
            failure: None,
        };
        app.results.albums.loading = true;
        app
    }

    #[test]
    fn failed_search_releases_show_more_of_the_old_results() {
        let mut app = app_with_results();
        app.search_generation = 2;
        let _ = app.update(Message::MoreAlbums(1, Ok(albums(&["a3"], Some(10)))));
        let _ = app.update(Message::SearchDone(2, Err("offline".into())));
        assert!(!app.results.albums.loading);
        assert_eq!(album_ids(&app), ["a1", "a2"]);
    }

    #[test]
    fn partly_failed_search_keeps_the_other_type() {
        let mut app = app();
        app.search_generation = 1;
        let payload = SearchPayload {
            query: "q".into(),
            albums: Section::first(albums(&["a1"], Some(1))),
            tracks: Section::default(),
            failure: Some("track search failed: 429".into()),
        };
        let _ = app.update(Message::SearchDone(1, Ok(payload)));
        assert_eq!(album_ids(&app), ["a1"]);
        let status = app.status.unwrap();
        assert_eq!(status.kind, status::StatusKind::Error);
        assert!(status.text.contains("track search failed"));
    }

    fn album_ids(app: &App) -> Vec<&str> {
        app.results
            .albums
            .items
            .iter()
            .map(|a| a.id.as_str())
            .collect()
    }

    #[test]
    fn stale_search_results_are_dropped() {
        let mut app = app_with_results();
        app.search_generation = 2;
        let old = SearchPayload {
            query: "old".into(),
            ..SearchPayload::default()
        };
        let _ = app.update(Message::SearchDone(1, Ok(old)));
        assert_eq!(app.results.query, "q");
    }

    #[test]
    fn more_albums_appends_to_albums_only() {
        let mut app = app_with_results();
        let _ = app.update(Message::MoreAlbums(1, Ok(albums(&["a2", "a3"], Some(10)))));
        assert_eq!(album_ids(&app), ["a1", "a2", "a3"]);
        assert!(!app.results.albums.loading);
        assert_eq!(app.results.tracks.items.len(), 1);
    }

    #[test]
    fn stale_page_is_discarded() {
        let mut app = app_with_results();
        app.search_generation = 2;
        let _ = app.update(Message::MoreAlbums(1, Ok(albums(&["a3"], Some(10)))));
        assert_eq!(album_ids(&app), ["a1", "a2"]);
    }

    #[test]
    fn failed_page_keeps_rows_and_allows_retry() {
        let mut app = app_with_results();
        let _ = app.update(Message::MoreAlbums(1, Err("boom".into())));
        assert_eq!(album_ids(&app), ["a1", "a2"]);
        assert!(!app.results.albums.loading);
        assert!(app.results.albums.has_more());
        assert_eq!(app.status.map(|s| s.kind), Some(status::StatusKind::Error));
    }

    /// Dummy credentials let `client()` build; the returned task never runs.
    fn open_album(app: &mut App, id: &str) {
        app.config.app_id = "1".into();
        app.config.app_secret = "s".into();
        let _ = app.update(Message::OpenAlbum(album(id)));
    }

    #[test]
    fn missing_credentials_fail_the_album_in_place() {
        let mut app = app();
        let _ = app.update(Message::OpenAlbum(album("a1")));
        assert!(matches!(app.album.unwrap().state, DetailState::Failed(_)));
    }

    fn queued_ids(app: &App) -> Vec<i64> {
        app.queue.iter().map(|it| it.track_id).collect()
    }

    #[test]
    fn opening_an_album_leaves_results_alone() {
        let mut app = app_with_results();
        open_album(&mut app, "a1");
        let detail = app.album.as_ref().unwrap();
        assert_eq!(detail.id(), "a1");
        assert!(matches!(detail.state, DetailState::Loading));
        assert_eq!(album_ids(&app), ["a1", "a2"]);
        assert!(app.queue.is_empty());
    }

    #[test]
    fn loaded_album_selects_every_track() {
        let mut app = app();
        open_album(&mut app, "a1");
        let jobs = vec![album::tests::job(1, 1, None), album::tests::job(2, 1, None)];
        let _ = app.update(Message::AlbumLoaded("a1".into(), Ok(jobs)));
        assert_eq!(app.album.unwrap().selected.len(), 2);
    }

    #[test]
    fn response_for_a_closed_album_is_dropped() {
        let mut app = app();
        open_album(&mut app, "a1");
        let _ = app.update(Message::CloseAlbum);
        let _ = app.update(Message::AlbumLoaded("a1".into(), Ok(vec![])));
        assert!(app.album.is_none());
    }

    #[test]
    fn response_for_another_album_is_dropped() {
        let mut app = app();
        open_album(&mut app, "a2");
        let jobs = vec![album::tests::job(1, 1, None)];
        let _ = app.update(Message::AlbumLoaded("a1".into(), Ok(jobs)));
        assert!(matches!(app.album.unwrap().state, DetailState::Loading));
    }

    fn app_with_loaded_album() -> App {
        let mut app = app();
        open_album(&mut app, "a1");
        let jobs = (1..=3).map(|id| album::tests::job(id, 1, None)).collect();
        let _ = app.update(Message::AlbumLoaded("a1".into(), Ok(jobs)));
        app
    }

    #[test]
    fn add_selected_enqueues_subset_in_album_order() {
        let mut app = app_with_loaded_album();
        let _ = app.update(Message::ToggleTrack(2));
        let _ = app.update(Message::AddSelected);
        assert_eq!(queued_ids(&app), [1, 3]);
        assert!(app.album.is_some());
    }

    fn press(app: &mut App, s: shortcut::Shortcut) {
        let _ = app.update(Message::Shortcut(s));
    }

    /// An open album with setup done, so Search shows the album rather than
    /// the setup prompt.
    fn set_up_with_loaded_album() -> App {
        let mut app = app_with_loaded_album();
        app.config.app_id = "id".into();
        app.config.app_secret = "secret".into();
        app.token = Some(StoredToken {
            value: "token".into(),
            origin: TokenOrigin::Restored,
        });
        app
    }

    #[test]
    fn escape_closes_the_open_album() {
        let mut app = set_up_with_loaded_album();
        press(&mut app, shortcut::Shortcut::Escape);
        assert!(app.album.is_none());
    }

    #[test]
    fn album_shortcuts_ignore_an_album_behind_the_setup_prompt() {
        let mut app = set_up_with_loaded_album();
        app.token = None;
        press(&mut app, shortcut::Shortcut::AddSelected);
        press(&mut app, shortcut::Shortcut::Escape);
        assert!(app.queue.is_empty());
        assert!(app.album.is_some());
    }

    #[test]
    fn slash_from_settings_stays_on_settings() {
        let mut app = app();
        app.screen = Screen::Settings;
        press(&mut app, shortcut::Shortcut::FocusSearch);
        assert_eq!(app.screen, Screen::Settings);
    }

    #[test]
    fn escape_on_another_tab_keeps_the_album() {
        let mut app = app_with_loaded_album();
        app.screen = Screen::Settings;
        press(&mut app, shortcut::Shortcut::Escape);
        assert!(app.album.is_some());
    }

    #[test]
    fn escape_without_an_album_changes_nothing() {
        let mut app = app();
        press(&mut app, shortcut::Shortcut::Escape);
        assert!(app.album.is_none());
        assert_eq!(app.screen, Screen::Search);
    }

    #[test]
    fn command_enter_adds_the_selection_on_search() {
        let mut app = set_up_with_loaded_album();
        press(&mut app, shortcut::Shortcut::AddSelected);
        assert_eq!(queued_ids(&app), [1, 2, 3]);
    }

    #[test]
    fn command_enter_off_the_search_tab_adds_nothing() {
        let mut app = app_with_loaded_album();
        app.screen = Screen::Queue;
        press(&mut app, shortcut::Shortcut::AddSelected);
        assert!(app.queue.is_empty());
    }

    #[test]
    fn tab_shortcuts_navigate() {
        let mut app = app();
        press(&mut app, shortcut::Shortcut::PreviousTab);
        assert_eq!(app.screen, Screen::Settings);
        press(&mut app, shortcut::Shortcut::NextTab);
        assert_eq!(app.screen, Screen::Search);
        app.screen = Screen::Queue;
        press(&mut app, shortcut::Shortcut::FocusSearch);
        assert_eq!(app.screen, Screen::Search);
    }

    #[test]
    fn add_selected_skips_tracks_already_queued() {
        let mut app = app_with_loaded_album();
        let _ = app.update(Message::AddSelected);
        let _ = app.update(Message::AddSelected);
        assert_eq!(queued_ids(&app), [1, 2, 3]);
    }

    #[test]
    fn empty_selection_adds_nothing() {
        let mut app = app_with_loaded_album();
        let _ = app.update(Message::SelectNoTracks);
        let _ = app.update(Message::AddSelected);
        assert!(app.queue.is_empty());
    }

    #[test]
    fn new_search_closes_the_album() {
        let mut app = app_with_loaded_album();
        app.search_query = "kind of blue".into();
        let _ = app.update(Message::SearchSubmit);
        assert!(app.album.is_none());
    }

    fn queued(track_id: i64, album_id: &str, status: ItemStatus) -> QueueItem {
        let mut job = album::tests::job(track_id, 1, None);
        job.album.id = album_id.into();
        QueueItem {
            track_id,
            job,
            status,
            downloaded: 0,
            total: None,
            path: None,
        }
    }

    #[test]
    fn search_keeps_queued_album_covers() {
        let mut app = app();
        let mut item = queued(1, "a", ItemStatus::Queued);
        item.job.album.image = Some(qobuz_core::models::Image {
            large: None,
            small: Some("cover-a".into()),
            thumbnail: None,
        });
        app.queue = vec![item];
        app.thumbnails.insert(
            "cover-a".into(),
            iced::widget::image::Handle::from_bytes(Vec::new()),
        );
        app.search_generation = 1;
        let _ = app.update(Message::SearchDone(1, Ok(SearchPayload::default())));
        assert!(app.thumbnails.contains_key("cover-a"));
    }

    /// A batch of tracks 2 and 3 that just ended, beside track 1 finished by
    /// an earlier batch, with the window in the background.
    fn app_after_batch() -> App {
        let mut app = app();
        app.queue = vec![
            queued(1, "a", ItemStatus::Done("FLAC".into())),
            queued(2, "a", ItemStatus::Done("FLAC".into())),
            queued(3, "a", ItemStatus::Error("x".into())),
        ];
        app.batch = vec![2, 3];
        app.window_focused = false;
        app
    }

    #[test]
    fn finish_notification_counts_only_the_batch() {
        assert_eq!(
            app_after_batch().finish_notification(false),
            Some((
                "Downloads finished with errors".into(),
                "1 downloaded, 1 failed".into()
            ))
        );
    }

    #[test]
    fn focused_window_gets_no_notification() {
        let mut app = app_after_batch();
        let _ = app.update(Message::WindowFocus(true));
        assert_eq!(app.finish_notification(false), None);
    }

    #[test]
    fn cancelled_batch_gets_no_notification() {
        assert_eq!(app_after_batch().finish_notification(true), None);
    }

    #[test]
    fn notify_toggle_updates_config() {
        let mut app = app();
        assert!(app.config.notify_on_finish);
        let _ = app.update(Message::NotifyToggled(false));
        assert!(!app.config.notify_on_finish);
    }

    fn done_at(track_id: i64, album_id: &str, file: &str) -> QueueItem {
        QueueItem {
            path: Some(PathBuf::from(file)),
            ..queued(track_id, album_id, ItemStatus::Done("FLAC".into()))
        }
    }

    #[test]
    fn queued_track_matches_by_id() {
        let mut app = app();
        app.queue = vec![queued(7, "a", ItemStatus::Queued)];
        assert!(app.queued().track("7"));
        assert!(!app.queued().track("8"));
        assert!(!app.queued().track("not-a-number"));
    }

    #[test]
    fn queued_album_matches_any_of_its_tracks() {
        let mut app = app();
        app.queue = vec![queued(7, "a", ItemStatus::Done("FLAC 24/96".into()))];
        assert!(app.queued().album("a"));
        assert!(!app.queued().album("b"));
    }

    #[test]
    fn queued_results_clear_with_the_queue() {
        let mut app = app();
        app.queue = vec![queued(7, "a", ItemStatus::Queued)];
        let _ = app.update(Message::ClearQueue);
        assert!(!app.queued().track("7"));
        assert!(!app.queued().album("a"));
    }

    #[test]
    fn album_folder_comes_from_done_tracks_only() {
        let mut app = app();
        app.queue = vec![
            done_at(1, "a", "/music/Album/01.flac"),
            queued(2, "a", ItemStatus::Queued),
            done_at(3, "b", "/music/Other/01.flac"),
        ];
        assert_eq!(app.album_folder("a"), Some(PathBuf::from("/music/Album")));
    }

    #[test]
    fn album_without_a_done_track_has_no_folder() {
        let mut app = app();
        app.queue = vec![queued(1, "a", ItemStatus::Queued)];
        assert_eq!(app.album_folder("a"), None);
    }

    #[test]
    fn renamed_folder_moves_only_the_paths_inside_it() {
        let mut app = app();
        app.queue = vec![
            done_at(1, "a", "/music/Album/Disc 1/01.flac"),
            done_at(2, "a", "/music/Album/Disc 2/01.flac"),
            done_at(3, "b", "/music/Album B/01.flac"),
        ];
        let _ = app.update(Message::FolderRenamed(
            PathBuf::from("/music/Album"),
            Ok(PathBuf::from("/music/Renamed")),
        ));
        let paths: Vec<_> = app
            .queue
            .iter()
            .map(|it| it.path.clone().unwrap())
            .collect();
        assert_eq!(
            paths,
            [
                "/music/Renamed/Disc 1/01.flac",
                "/music/Renamed/Disc 2/01.flac",
                "/music/Album B/01.flac",
            ]
            .map(PathBuf::from)
        );
        assert_eq!(app.album_folder("a"), Some(PathBuf::from("/music/Renamed")));
    }

    #[test]
    fn folder_shared_with_another_album_is_not_renamed() {
        let mut app = app();
        app.queue = vec![
            done_at(1, "a", "/music/Shared/01.flac"),
            done_at(2, "b", "/music/Shared/02.flac"),
        ];
        app.rename = Some(RenameState {
            album_id: "a".into(),
            name: "New".into(),
            suggesting: false,
        });
        let _ = app.update(Message::ConfirmRename);
        assert!(app.rename.is_some(), "the field stays open");
        let status = app.status.unwrap();
        assert_eq!(status.kind, status::StatusKind::Error);
        assert!(status.text.contains("another album"));
    }

    #[test]
    fn empty_name_is_not_renamed() {
        let mut app = app();
        app.queue = vec![done_at(1, "a", "/music/A/01.flac")];
        app.rename = Some(RenameState {
            album_id: "a".into(),
            name: " / ".into(),
            suggesting: false,
        });
        let _ = app.update(Message::ConfirmRename);
        assert!(app.rename.is_some());
        assert_eq!(app.status, None);
    }

    #[test]
    fn one_done_disc_renames_the_album_folder() {
        let mut app = app();
        let mut item = done_at(1, "a", "/music/Album/Disc 1/01.flac");
        item.job.multi_disc = true;
        app.queue = vec![item];
        assert_eq!(app.rename_target("a"), Some(PathBuf::from("/music/Album")));
    }

    #[test]
    fn rename_is_not_offered_while_downloading() {
        let mut app = app();
        app.queue = vec![
            done_at(1, "a", "/music/A/01.flac"),
            queued(2, "a", ItemStatus::Downloading),
        ];
        let _ = app.update(Message::RenameFolder("a".into()));
        assert!(app.rename.is_none());
    }

    #[test]
    fn typing_drops_a_late_suggestion() {
        let mut app = app();
        app.queue = vec![done_at(1, "a", "/music/Old/01.flac")];
        let _ = app.update(Message::RenameFolder("a".into()));
        assert_eq!(app.rename.as_ref().map(|r| r.name.as_str()), Some("Old"));
        let _ = app.update(Message::RenameNameChanged("Mine".into()));
        let _ = app.update(Message::RenameSuggested(
            "a".into(),
            Ok(Some("Suggested".into())),
        ));
        assert_eq!(app.rename.as_ref().map(|r| r.name.as_str()), Some("Mine"));
    }

    #[test]
    fn empty_suggestion_keeps_the_current_name() {
        let mut app = app();
        app.queue = vec![done_at(1, "a", "/music/Old/01.flac")];
        let _ = app.update(Message::RenameFolder("a".into()));
        let _ = app.update(Message::RenameSuggested("a".into(), Ok(None)));
        let rename = app.rename.unwrap();
        assert_eq!(rename.name, "Old");
        assert!(!rename.suggesting);
    }

    #[test]
    fn rename_is_not_offered_while_another_album_downloads() {
        let mut app = app();
        app.queue = vec![
            done_at(1, "a", "/music/A/01.flac"),
            queued(2, "b", ItemStatus::Downloading),
        ];
        app.downloading = true;
        app.batch = vec![2];
        assert!(!app.renamable("a"));
        assert!(app.tags_editable("a"), "tags stay editable");
    }

    #[test]
    fn rename_is_not_offered_with_a_failed_track() {
        let mut app = app();
        app.queue = vec![
            done_at(1, "a", "/music/A/01.flac"),
            queued(2, "a", ItemStatus::Error("boom".into())),
        ];
        assert!(!app.renamable("a"));
        assert!(app.tags_editable("a"), "tags stay editable");
    }

    #[test]
    fn removing_the_album_closes_its_rename_field() {
        let mut app = app();
        app.queue = vec![done_at(1, "a", "/music/A/01.flac")];
        let _ = app.update(Message::RenameFolder("a".into()));
        assert!(app.rename.is_some());
        let _ = app.update(Message::RemoveGroup("a".into()));
        assert!(app.rename.is_none());

        app.queue = vec![done_at(1, "a", "/music/A/01.flac")];
        let _ = app.update(Message::RenameFolder("a".into()));
        let _ = app.update(Message::ClearQueue);
        assert!(app.rename.is_none());
    }

    #[test]
    fn failed_open_is_reported() {
        let mut app = app();
        let _ = app.update(Message::Opened(Err("Could not open: gone".into())));
        let status = app.status.unwrap();
        assert_eq!(status.kind, status::StatusKind::Error);
        assert!(status.text.contains("gone"));
    }

    #[test]
    fn only_settled_tracks_are_removable() {
        assert!(removable(&ItemStatus::Queued));
        assert!(removable(&ItemStatus::Done("FLAC".into())));
        assert!(removable(&ItemStatus::Error("x".into())));
        assert!(!removable(&ItemStatus::Downloading));
        assert!(!removable(&ItemStatus::Tagging));
    }

    #[test]
    fn done_event_keeps_the_file_path_and_cancel_clears_it() {
        let mut app = app();
        app.queue = vec![queued(1, "a", ItemStatus::Downloading)];
        let file = PathBuf::from("/music/a/01.flac");
        let _ = app.update(Message::Download(JobEvent::Done {
            track_id: 1,
            path: file.clone(),
            delivered: "FLAC".into(),
        }));
        assert_eq!(app.queue[0].path, Some(file));
        let _ = app.update(Message::Download(JobEvent::Cancelled { track_id: 1 }));
        assert_eq!(app.queue[0].path, None);
    }

    #[test]
    fn toggle_group_flips_collapsed() {
        let mut app = app();
        let _ = app.update(Message::ToggleGroup("a".into()));
        assert!(app.collapsed.contains("a"));
        let _ = app.update(Message::ToggleGroup("a".into()));
        assert!(app.collapsed.is_empty());
    }

    #[test]
    fn remove_group_drops_all_its_settled_tracks() {
        let mut app = app();
        app.queue = vec![
            queued(1, "a", ItemStatus::Queued),
            queued(2, "a", ItemStatus::Done("FLAC".into())),
            queued(3, "a", ItemStatus::Error("x".into())),
            queued(4, "a", ItemStatus::Tagging),
            queued(5, "b", ItemStatus::Done("FLAC".into())),
        ];
        let _ = app.update(Message::RemoveGroup("a".into()));
        assert_eq!(queued_ids(&app), [4, 5]);
    }

    #[test]
    fn removing_a_group_drops_its_failures_from_retry() {
        let mut app = app();
        app.queue = vec![
            queued(1, "a", ItemStatus::Error("x".into())),
            queued(2, "a", ItemStatus::Done("FLAC".into())),
            queued(3, "b", ItemStatus::Error("y".into())),
        ];
        let _ = app.update(Message::RemoveGroup("a".into()));
        let failed: Vec<i64> = app
            .jobs_with(|s| matches!(s, ItemStatus::Error(_)))
            .iter()
            .map(|job| job.track.id)
            .collect();
        assert_eq!(failed, [3]);
    }

    #[test]
    fn remove_group_keeps_rows_of_the_running_batch() {
        let mut app = app();
        app.queue = vec![
            queued(1, "a", ItemStatus::Queued),
            queued(2, "a", ItemStatus::Queued),
        ];
        app.batch = vec![1];
        app.downloading = true;
        let _ = app.update(Message::RemoveGroup("a".into()));
        assert_eq!(queued_ids(&app), [1]);
    }

    #[test]
    fn removed_album_forgets_it_was_collapsed() {
        let mut app = app();
        app.queue = vec![
            queued(1, "a", ItemStatus::Done("FLAC".into())),
            queued(2, "b", ItemStatus::Done("FLAC".into())),
        ];
        app.collapsed = HashSet::from(["a".to_owned(), "b".to_owned()]);
        let _ = app.update(Message::RemoveGroup("a".into()));
        assert_eq!(app.collapsed, HashSet::from(["b".to_owned()]));
    }

    #[test]
    fn finished_status_counts_only_the_batch() {
        let mut app = app();
        app.queue = vec![
            queued(1, "a", ItemStatus::Error("x".into())),
            queued(2, "a", ItemStatus::Done("FLAC".into())),
        ];
        app.batch = vec![2];
        app.downloading = true;
        let _ = app.update(Message::DownloadsFinished(None));
        assert_eq!(
            app.status.map(|s| s.kind),
            Some(status::StatusKind::Success)
        );
    }

    #[test]
    fn late_cancel_with_tracks_added_mid_batch_is_not_cancelled() {
        let mut app = app();
        app.queue = vec![
            queued(1, "a", ItemStatus::Done("FLAC".into())),
            queued(2, "b", ItemStatus::Queued),
        ];
        app.batch = vec![1];
        app.downloading = true;
        let cancel = CancellationToken::new();
        cancel.cancel();
        app.cancel = Some(cancel);
        let _ = app.update(Message::DownloadsFinished(None));
        assert_eq!(
            app.status.map(|s| s.kind),
            Some(status::StatusKind::Success)
        );
    }

    #[test]
    fn a_cover_is_requested_once_until_it_loads() {
        let mut app = app();
        let url = || vec!["cover".to_owned()];
        let _ = app.fetch_missing_covers(url());
        assert!(app.cover_requests.contains("cover"));
        let _ = app.fetch_missing_covers(url());
        assert_eq!(app.cover_requests.len(), 1);
        let _ = app.update(Message::ThumbnailLoaded("cover".into(), Ok(Vec::new())));
        assert!(app.cover_requests.is_empty());
        assert!(app.thumbnails.contains_key("cover"));
    }

    #[test]
    fn show_more_is_ignored_while_loading() {
        let mut app = app_with_results();
        let _ = app.update(Message::ShowMore(Kind::Albums));
        assert!(app.status.is_none());
        assert!(app.results.albums.loading);
    }

    #[test]
    fn tags_are_editable_once_an_album_is_settled() {
        let mut app = app();
        app.queue = vec![
            done_at(1, "a", "/m/a/1.flac"),
            queued(2, "a", ItemStatus::Error("x".into())),
        ];
        assert!(app.tags_editable("a"));

        app.queue.push(queued(3, "a", ItemStatus::Queued));
        assert!(!app.tags_editable("a"), "a queued track blocks editing");

        app.queue = vec![queued(1, "b", ItemStatus::Error("x".into()))];
        assert!(!app.tags_editable("b"), "nothing done to edit");
    }

    #[test]
    fn tags_are_not_editable_while_in_the_running_batch() {
        let mut app = app();
        app.queue = vec![done_at(1, "a", "/m/a/1.flac")];
        app.downloading = true;
        app.batch = vec![1];
        assert!(!app.tags_editable("a"));
    }

    /// An app whose editor is open on album "a", with one track read.
    fn editing() -> App {
        let mut app = app();
        app.queue = vec![done_at(1, "a", "/m/a/1.flac")];
        let _ = app.update(Message::EditTags("a".into()));
        let job = app.queue[0].job.clone();
        let read = vec![(job, PathBuf::from("/m/a/1.flac"), Ok(Default::default()))];
        let _ = app.update(Message::TagsRead("a".into(), Ok(read)));
        app
    }

    #[test]
    fn edit_tags_opens_the_editor_after_reading() {
        let mut app = app();
        app.queue = vec![done_at(1, "a", "/m/a/1.flac")];
        let _ = app.update(Message::EditTags("a".into()));
        assert!(matches!(app.tag_editor, Some(EditorSlot::Loading(ref id)) if id == "a"));

        let app = editing();
        assert_eq!(app.open_editor().map(|e| e.tracks.len()), Some(1));
    }

    #[test]
    fn a_read_for_another_album_is_dropped() {
        let mut app = editing();
        let _ = app.update(Message::TagsRead("other".into(), Ok(Vec::new())));
        assert_eq!(app.open_editor().map(|e| e.album_id.as_str()), Some("a"));
    }

    #[test]
    fn picked_cover_takes_the_download_size() {
        let mut app = editing();
        app.config.cover_size = qobuz_core::CoverSize::Px500;
        let _ = app.update(Message::CoverPicked(Ok(Some(vec![1, 2, 3]))));
        let editor = app.open_editor().unwrap();
        assert_eq!(editor.resize, Some(qobuz_core::CoverSize::Px500));
    }

    /// `editing()` with track 1's title changed and its save under way, the
    /// cover plan ready and the file handed to the save task.
    fn saving_title() -> App {
        let mut app = editing();
        let _ = app.update(Message::Editor(tag_editor::Edit::TrackText(
            1,
            qobuz_core::tag_edit::Field::Title,
            "New".into(),
        )));
        let _ = app.update(Message::SaveTags);
        let plan = qobuz_core::tag_edit::CoverPlan::new(Default::default());
        let _ = app.update(Message::CoverPlanned(Ok(plan)));
        app
    }

    #[test]
    fn failed_save_keeps_the_edits_to_retry() {
        let mut app = saving_title();
        let _ = app.update(Message::TagsSaved(1, Err("read-only".into())));
        let editor = app.open_editor().expect("editor stays open");
        assert!(editor.saving.is_none());
        assert!(editor.has_edits(), "the edit is kept for a retry");
        assert!(app.can_save_tags());
        assert_eq!(
            app.status.as_ref().map(|s| s.kind),
            Some(status::StatusKind::Error)
        );
    }

    #[test]
    fn save_closes_the_editor_when_the_album_was_requeued() {
        let mut app = saving_title();
        app.queue[0].status = ItemStatus::Queued;
        let _ = app.update(Message::TagsSaved(1, Ok(Saved::Written)));
        assert!(app.tag_editor.is_none());
    }

    #[test]
    fn cover_picked_during_a_save_is_ignored() {
        let mut app = saving_title();
        let _ = app.update(Message::CoverPicked(Ok(Some(vec![1, 2, 3]))));
        let editor = app.open_editor().unwrap();
        assert_eq!(editor.cover, qobuz_core::tag_edit::CoverAction::Keep);
    }

    #[test]
    fn save_needs_edits_and_close_waits_for_it() {
        let mut app = editing();
        assert!(!app.can_save_tags(), "nothing to save yet");
        let _ = app.update(Message::Editor(tag_editor::Edit::TrackText(
            1,
            qobuz_core::tag_edit::Field::Title,
            "New".into(),
        )));
        assert!(app.can_save_tags());

        let _ = app.update(Message::SaveTags);
        assert!(app.open_editor().is_some_and(|e| e.saving.is_some()));
        let _ = app.update(Message::CloseTagEditor);
        assert!(app.tag_editor.is_some(), "can't close mid-save");
    }

    #[test]
    fn edited_album_keeps_its_cover_loaded() {
        let mut app = editing();
        app.queue[0].job.album.image = Some(qobuz_core::models::Image {
            thumbnail: Some("http://x/a.jpg".into()),
            small: Some("http://x/a.jpg".into()),
            large: Some("http://x/a.jpg".into()),
        });
        let url = tasks::thumbnail(app.queue[0].job.album.image.as_ref()).unwrap();
        assert!(app.covers_in_use().contains(&url));
    }
}
