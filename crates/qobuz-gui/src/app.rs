//! The iced desktop application: settings, search/add, and download queue.

use crate::style::{self, secondary_button};
use album::{AlbumDetail, DetailState};
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
use qobuz_core::{auth, engine, AppCredentials, CancellationToken, QobuzClient, SigningCheck};
use status::Status;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

mod album;
mod help;
mod omnibox;
mod paging;
mod status;
mod tasks;
mod view;

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

/// A single row in the download queue.
#[derive(Debug, Clone)]
struct QueueItem {
    track_id: i64,
    /// The resolved job, retained so a failed track can be relaunched.
    job: Job,
    status: ItemStatus,
    downloaded: u64,
    total: Option<u64>,
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

    // Queue.
    queue: Vec<QueueItem>,
    /// Album ids whose queue group is collapsed to its header.
    collapsed: HashSet<String>,
    downloading: bool,
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
    ConcurrencyChanged(usize),
    QualitySelected(Quality),
    EmbedArtToggled(bool),
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
    RetryTrack(i64),
    DequeueTrack(i64),
    RetryFailed,
    ClearQueue,
    ToggleGroup(String),
    /// Removes the still-queued tracks of one album group.
    RemoveGroup(String),
    Download(JobEvent),
    /// Carries the app secret that actually signed during the batch, if any, so
    /// it can be promoted to the primary secret and persisted.
    DownloadsFinished(Option<String>),
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
        (Self::from_parts(config, token, status), Task::none())
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
            queue: Vec::new(),
            collapsed: HashSet::new(),
            downloading: false,
            cancel: None,
            token,
            status,
            config,
        }
    }

    /// Signed-in state, derived from the token so the two can never disagree.
    fn signed_in(&self) -> bool {
        self.token.is_some()
    }

    /// Persist the config, surfacing a failure instead of dropping it silently.
    fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
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
                self.status = Some(match self.config.save() {
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
            Message::ConcurrencyChanged(n) => {
                self.config.concurrency = n;
                Task::none()
            }
            Message::QualitySelected(q) => {
                self.config.quality = q;
                Task::none()
            }
            Message::EmbedArtToggled(b) => {
                self.config.embed_art = b;
                Task::none()
            }
            Message::PickDir => Task::perform(tasks::pick_dir(), Message::DirChosen),
            Message::DirChosen(Some(p)) => {
                self.config.download_dir = p;
                Task::none()
            }
            Message::DirChosen(None) => Task::none(),
            Message::SaveSettings => {
                match self.config.save() {
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
                self.status = Some(if n == 0 {
                    Status::info("No results.")
                } else {
                    Status::success(format!("{n} results."))
                });
                // Keep only this search's covers cached — without the eviction
                // the map grows for every cover ever viewed in the session.
                let wanted: std::collections::HashSet<String> = payload
                    .albums
                    .items
                    .iter()
                    .filter_map(|a| a.cover.clone())
                    .chain(payload.tracks.items.iter().filter_map(|t| t.cover.clone()))
                    // The queue's group headers show covers from the same
                    // cache, so a new search must not blank them.
                    .chain(self.queue_covers())
                    .collect();
                self.thumbnails.retain(|url, _| wanted.contains(url));
                self.results = payload;
                self.fetch_missing_covers(wanted)
            }
            Message::SearchDone(_, Err(e)) => {
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
            Message::RetryTrack(track_id) => {
                let job = self
                    .queue
                    .iter()
                    .find(|it| it.track_id == track_id)
                    .filter(|it| matches!(it.status, ItemStatus::Error(_)))
                    .map(|it| it.job.clone());
                match job {
                    Some(job) => self.spawn_downloads(vec![job]),
                    None => Task::none(),
                }
            }
            Message::DequeueTrack(track_id) => {
                let before = self.queue.len();
                self.queue.retain(|it| {
                    !(it.track_id == track_id && matches!(it.status, ItemStatus::Queued))
                });
                if self.queue.len() != before {
                    self.status = Some(Status::info("Removed from queue."));
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
                // Same rule as `DequeueTrack`: only queued tracks, and never
                // while a batch could be about to start them.
                if self.downloading {
                    return Task::none();
                }
                let before = self.queue.len();
                self.queue.retain(|it| {
                    !(it.job.album.id == album_id && matches!(it.status, ItemStatus::Queued))
                });
                let removed = before - self.queue.len();
                if removed > 0 {
                    self.status = Some(Status::info(format!(
                        "Removed {removed} track(s) from the queue."
                    )));
                }
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
                // be a lie — nothing was stopped.
                let was_cancelled =
                    self.cancel.take().is_some_and(|c| c.is_cancelled()) && startable(&self.queue);
                // Persist the secret that actually signed so the next session
                // starts from the known-good one instead of re-probing.
                if let Some(secret) = working_secret {
                    if secret != self.config.app_secret {
                        self.config.promote_secret(&secret);
                        self.save_config();
                    }
                }
                let errors = self
                    .queue
                    .iter()
                    .filter(|i| matches!(i.status, ItemStatus::Error(_)))
                    .count();
                self.status = Some(if was_cancelled {
                    // A track can finish between the click and the stop, so
                    // report what actually completed rather than what was
                    // showing when Cancel was pressed.
                    let done = self
                        .queue
                        .iter()
                        .filter(|i| matches!(i.status, ItemStatus::Done(_)))
                        .count();
                    let mut s = format!(
                        "Download cancelled — {done} of {} completed",
                        self.queue.len()
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
                Task::none()
            }
        }
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

    /// Lazily load cover thumbnails not already cached.
    fn fetch_missing_covers(&self, urls: impl IntoIterator<Item = String>) -> Task<Message> {
        let fetches: Vec<Task<Message>> = urls
            .into_iter()
            .filter(|url| !self.thumbnails.contains_key(url))
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
            }
        }
        let config = self.config.clone();
        self.downloading = true;
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
            JobEvent::Done { delivered, .. } => item.status = ItemStatus::Done(delivered),
            JobEvent::Failed { error, .. } => item.status = ItemStatus::Error(error),
            // Back to queued with its progress discarded, so the partial file
            // (already deleted by the engine) is re-fetched from scratch and
            // `startable` offers Start again to resume the batch.
            JobEvent::Cancelled { .. } => {
                item.status = ItemStatus::Queued;
                item.downloaded = 0;
                item.total = None;
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
            secondary_button(
                if self.config.dark_mode {
                    "Light theme"
                } else {
                    "Dark theme"
                },
                Message::ToggleTheme,
            ),
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
                TabLabel::Text("Search / Add".to_owned()),
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

/// Wraps a tab's content in a bordered pane so the active tab's area is clearly
/// delimited beneath the tab bar.
fn tab_pane<'a>(content: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
    container(content)
        .style(style::panel)
        .padding(style::SPACE_LG)
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
        };
        app.results.albums.loading = true;
        app
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

    #[test]
    fn toggle_group_flips_collapsed() {
        let mut app = app();
        let _ = app.update(Message::ToggleGroup("a".into()));
        assert!(app.collapsed.contains("a"));
        let _ = app.update(Message::ToggleGroup("a".into()));
        assert!(app.collapsed.is_empty());
    }

    #[test]
    fn remove_group_drops_only_its_queued_tracks() {
        let mut app = app();
        app.queue = vec![
            queued(1, "a", ItemStatus::Queued),
            queued(2, "a", ItemStatus::Done("FLAC".into())),
            queued(3, "b", ItemStatus::Queued),
        ];
        let _ = app.update(Message::RemoveGroup("a".into()));
        assert_eq!(queued_ids(&app), [2, 3]);
    }

    #[test]
    fn remove_group_is_ignored_while_downloading() {
        let mut app = app();
        app.queue = vec![queued(1, "a", ItemStatus::Queued)];
        app.downloading = true;
        let _ = app.update(Message::RemoveGroup("a".into()));
        assert_eq!(queued_ids(&app), [1]);
    }

    #[test]
    fn show_more_is_ignored_while_loading() {
        let mut app = app_with_results();
        let _ = app.update(Message::ShowMore(Kind::Albums));
        assert!(app.status.is_none());
        assert!(app.results.albums.loading);
    }
}
