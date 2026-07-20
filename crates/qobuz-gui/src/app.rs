//! The iced desktop application: settings, search/add, and download queue.

use crate::style::{self, secondary_button};
use iced::futures::{future, SinkExt};
use iced::widget::{column, container, row, text};
use iced::{Element, Length, Task, Theme};
use iced_aw::widget::{tab_bar::TabLabel, tabs::Tabs};
use qobuz_core::catalog::Reference;
use qobuz_core::config::Config;
use qobuz_core::engine::{Job, JobEvent};
use qobuz_core::quality::Quality;
use qobuz_core::{auth, engine, AppCredentials, QobuzClient, SigningCheck};
use std::collections::HashMap;
use std::path::PathBuf;

mod help;
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
    title: String,
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

/// Search results reduced to display-ready entries.
#[derive(Debug, Clone, Default)]
struct SearchPayload {
    albums: Vec<AlbumResult>,
    tracks: Vec<TrackResult>,
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
    status: String,
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
    url_input: String,
    results: SearchPayload,
    /// Album cover thumbnails, keyed by cover URL, loaded lazily.
    thumbnails: HashMap<String, iced::widget::image::Handle>,

    // Queue.
    queue: Vec<QueueItem>,
    downloading: bool,

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
    SearchDone(Result<SearchPayload, String>),
    ThumbnailLoaded(String, Result<Vec<u8>, ()>),
    UrlChanged(String),
    AddUrl,
    Add(Reference),
    Resolved(Result<Vec<Job>, String>),

    // Downloads.
    StartDownloads,
    RetryTrack(i64),
    DequeueTrack(i64),
    RetryFailed,
    ClearQueue,
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
        let status = if let Some(e) = config_error {
            format!("Could not load saved settings ({e}); using defaults.")
        } else if token.is_some() {
            "Restored saved session.".to_string()
        } else if !config.has_app_credentials() {
            "Enter your Qobuz app_id / app_secret and sign in (Settings).".to_string()
        } else {
            "Sign in on the Settings screen.".to_string()
        };
        let app = App {
            screen: Screen::Settings,
            show_template_help: false,
            show_credentials_help: false,
            show_account_help: false,
            show_options_help: false,
            token_input: String::new(),
            secret_manually_edited: false,
            search_query: String::new(),
            url_input: String::new(),
            results: SearchPayload::default(),
            thumbnails: HashMap::new(),
            queue: Vec::new(),
            downloading: false,
            token,
            status,
            config,
        };
        (app, Task::none())
    }

    /// Signed-in state, derived from the token so the two can never disagree.
    fn signed_in(&self) -> bool {
        self.token.is_some()
    }

    /// Persist the config, surfacing a failure instead of dropping it silently.
    fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
            tracing::warn!("could not save config: {e}");
            self.status = format!("Could not save settings: {e}");
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
                self.status = "Detecting credentials from the Qobuz web player…".into();
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
                self.status = match self.config.save() {
                    Ok(()) => "Credentials detected and saved. You can now sign in.".into(),
                    Err(e) => format!("Credentials detected but could not save: {e}"),
                };
                Task::none()
            }
            Message::CredentialsDetected(Err(e)) => {
                self.status = format!("Auto-detect failed: {e}. Enter credentials manually.");
                Task::none()
            }
            Message::CheckSigning => {
                if !self.signed_in() {
                    self.status = "Sign in before checking signing.".into();
                    return Task::none();
                }
                match self.client() {
                    Ok(client) => {
                        self.status = "Checking request signing…".into();
                        Task::perform(tasks::check_signing_probe(client), Message::SigningChecked)
                    }
                    Err(e) => {
                        self.status = e;
                        Task::none()
                    }
                }
            }
            Message::SigningChecked(Ok(SigningCheck::Primary)) => {
                self.status = "Signing OK — request signatures are being accepted.".into();
                Task::none()
            }
            Message::SigningChecked(Ok(SigningCheck::Fallback { working_secret })) => {
                if self.secret_manually_edited {
                    // The user typed a secret that doesn't sign; only a saved
                    // fallback does. Flag it rather than silently overriding.
                    self.status =
                        "Entered app_secret is invalid — a saved fallback works; update or \
                         re-detect it."
                            .into();
                } else {
                    // The primary came from auto-detect; adopt the candidate that
                    // actually signs so the check reads cleanly from now on.
                    self.config.promote_secret(&working_secret);
                    self.status = "Signing OK — request signatures are being accepted.".into();
                    self.save_config();
                }
                Task::none()
            }
            Message::SigningChecked(Err(e)) => {
                self.status = format!("Signing check failed: {e}");
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
                    Ok(()) => self.status = "Settings saved.".into(),
                    Err(e) => self.status = format!("Could not save settings: {e}"),
                }
                Task::none()
            }
            Message::LoginToken => {
                if !self.config.has_app_credentials() {
                    self.status = "Enter app_id and app_secret first.".into();
                    return Task::none();
                }
                let (id, secret) = (self.config.app_id.clone(), self.config.app_secret.clone());
                let token = self.token_input.trim().to_string();
                if token.is_empty() {
                    self.status = "Paste a user_auth_token first.".into();
                    return Task::none();
                }
                self.status = "Validating token…".into();
                Task::perform(tasks::login_token(id, secret, token), Message::LoggedIn)
            }
            Message::LoggedIn(Ok(token)) => {
                if let Err(e) = auth::store_token(&token) {
                    self.status = format!("Signed in, but token could not be stored: {e}");
                } else {
                    self.status = "Signed in.".into();
                }
                self.token = Some(StoredToken {
                    value: token,
                    origin: TokenOrigin::ValidatedThisSession,
                });
                self.save_config();
                Task::none()
            }
            Message::LoggedIn(Err(e)) => {
                self.status = format!("Sign-in failed: {e}");
                Task::none()
            }
            Message::SignOut => {
                // Only drop the in-memory token when the keyring copy is
                // actually gone — the displayed state must stay truthful.
                self.status = match auth::clear_token() {
                    Ok(()) => {
                        self.token = None;
                        "Signed out.".into()
                    }
                    Err(e) => {
                        format!("Sign-out failed: the stored token could not be removed: {e}")
                    }
                };
                Task::none()
            }

            // ---- Search / add ----
            Message::SearchQueryChanged(v) => {
                self.search_query = v;
                Task::none()
            }
            Message::SearchSubmit => {
                let q = self.search_query.trim().to_string();
                if q.is_empty() {
                    return Task::none();
                }
                let client = match self.client() {
                    Ok(c) => c,
                    Err(e) => {
                        self.status = e;
                        return Task::none();
                    }
                };
                self.status = format!("Searching “{q}”…");
                Task::perform(tasks::do_search(client, q), Message::SearchDone)
            }
            Message::SearchDone(Ok(payload)) => {
                let n = payload.albums.len() + payload.tracks.len();
                self.status = if n == 0 {
                    "No results.".into()
                } else {
                    format!("{n} results.")
                };
                // Keep only this search's covers cached — without the eviction
                // the map grows for every cover ever viewed in the session.
                let wanted: std::collections::HashSet<String> = payload
                    .albums
                    .iter()
                    .filter_map(|a| a.cover.clone())
                    .chain(payload.tracks.iter().filter_map(|t| t.cover.clone()))
                    .collect();
                self.thumbnails.retain(|url, _| wanted.contains(url));
                // Lazily load album cover thumbnails not already cached.
                let fetches: Vec<Task<Message>> = wanted
                    .into_iter()
                    .filter(|url| !self.thumbnails.contains_key(url))
                    .map(|url| {
                        Task::perform(tasks::fetch_thumbnail(url.clone()), move |res| {
                            Message::ThumbnailLoaded(url.clone(), res)
                        })
                    })
                    .collect();
                self.results = payload;
                Task::batch(fetches)
            }
            Message::SearchDone(Err(e)) => {
                self.status = format!("Search failed: {e}");
                Task::none()
            }
            Message::ThumbnailLoaded(url, Ok(bytes)) => {
                self.thumbnails
                    .insert(url, iced::widget::image::Handle::from_bytes(bytes));
                Task::none()
            }
            Message::ThumbnailLoaded(_, Err(())) => Task::none(),
            Message::UrlChanged(v) => {
                self.url_input = v;
                Task::none()
            }
            Message::AddUrl => match qobuz_core::catalog::parse_input(&self.url_input) {
                Ok(reference) => self.update(Message::Add(reference)),
                Err(e) => {
                    self.status = e.to_string();
                    Task::none()
                }
            },
            Message::Add(reference) => {
                let client = match self.client() {
                    Ok(c) => c,
                    Err(e) => {
                        self.status = e;
                        return Task::none();
                    }
                };
                self.status = format!("Resolving {}…", reference.kind());
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
                        title: format!("{} — {}", job.track.artist_name(), job.track.title),
                        job,
                        status: ItemStatus::Queued,
                        downloaded: 0,
                        total: None,
                    });
                    added += 1;
                }
                self.status = format!("Added {added} track(s) to the queue.");
                self.screen = Screen::Queue;
                Task::none()
            }
            Message::Resolved(Err(e)) => {
                self.status = format!("Could not resolve: {e}");
                Task::none()
            }

            // ---- Downloads ----
            Message::StartDownloads => {
                // Download everything not yet done (fresh + previously errored).
                let jobs =
                    self.jobs_with(|s| matches!(s, ItemStatus::Queued | ItemStatus::Error(_)));
                if jobs.is_empty() {
                    self.status = "Nothing queued to download.".into();
                    return Task::none();
                }
                self.spawn_downloads(jobs)
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
                    self.status = "Removed from queue.".into();
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
                self.status = "Queue cleared.".into();
                Task::none()
            }
            Message::Download(ev) => {
                self.apply_event(ev);
                Task::none()
            }
            Message::DownloadsFinished(working_secret) => {
                self.downloading = false;
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
                self.status = if errors == 0 {
                    "All downloads finished.".into()
                } else {
                    format!("Downloads finished with {errors} error(s).")
                };
                Task::none()
            }
        }
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
            self.status = "Sign in before downloading.".into();
            return Task::none();
        }
        let client = match self.client() {
            Ok(c) => c,
            Err(e) => {
                self.status = e;
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
        self.status = format!("Downloading {} track(s)…", jobs.len());

        let stream = iced::stream::channel(256, move |mut output| async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<JobEvent>(256);
            // The engine clones the client internally; a retained clone shares
            // the `working_secret` cache, so we can read which secret signed
            // once the batch completes.
            let probe = client.clone();
            let engine = engine::download_all(client, config, jobs, tx);
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
            | JobEvent::Failed { track_id, .. } => *track_id,
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let a = style::accents(&self.theme());
        let wordmark = row![
            text("Qobuz")
                .size(style::TEXT_TITLE)
                .font(view::bold())
                .color(a.mauve),
            text("dl")
                .size(style::TEXT_TITLE)
                .font(view::bold())
                .color(a.blue),
        ]
        .spacing(2);
        let signed_in = self.signed_in();
        let header = row![
            wordmark.width(Length::Fill),
            secondary_button(
                if self.config.dark_mode {
                    "☀  Light"
                } else {
                    "★  Dark"
                },
                Message::ToggleTheme,
            ),
            text(if signed_in {
                "●  signed in"
            } else {
                "○  signed out"
            })
            .size(style::TEXT_SM)
            .color(if signed_in { a.green } else { a.red }),
        ]
        .spacing(style::SPACE_MD)
        .align_y(iced::Alignment::Center);

        let tabs = Tabs::new(Message::Navigate)
            .push(
                Screen::Settings,
                TabLabel::Text("Settings".to_owned()),
                tab_pane(view::settings::settings_view(self)),
            )
            .push(
                Screen::Search,
                TabLabel::Text("Search / Add".to_owned()),
                tab_pane(view::search::search_view(self)),
            )
            .push(
                Screen::Queue,
                TabLabel::Text("Queue".to_owned()),
                tab_pane(view::queue::queue_view(self)),
            )
            .set_active_tab(&self.screen)
            .tab_bar_style(style::tab_bar)
            .tab_label_padding([style::SPACE_SM as f32, style::SPACE_LG as f32])
            .tab_label_spacing(style::SPACE_XS as f32)
            .text_size(style::TEXT_BODY as f32)
            .height(Length::Fill);

        let status_bar = container(text(&self.status).size(style::TEXT_SM))
            .style(style::status_surface)
            .padding([style::SPACE_SM, style::SPACE_MD])
            .width(Length::Fill);

        let content = column![header, status_bar, tabs]
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
