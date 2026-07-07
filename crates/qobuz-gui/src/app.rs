//! The iced desktop application: settings, search/add, and download queue.

use crate::style::{
    self, action_button, field_input, labeled_row, secondary_button, styled_button,
};
use iced::font;
use iced::futures::{future, SinkExt};
use iced::widget::{
    checkbox, column, container, image, pick_list, progress_bar, row, scrollable, text,
};
use iced::{Element, Font, Length, Task, Theme};
use iced_aw::widget::{
    badge::Badge, card::Card, number_input::NumberInput, tab_bar::TabLabel, tabs::Tabs,
};
use qobuz_core::catalog::Reference;
use qobuz_core::config::Config;
use qobuz_core::engine::{Job, JobEvent};
use qobuz_core::quality::Quality;
use qobuz_core::{auth, engine, AppCredentials, QobuzClient};
use std::collections::HashMap;
use std::path::PathBuf;

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

/// An album search result: id, display label, and an optional cover URL.
#[derive(Debug, Clone)]
struct AlbumResult {
    id: String,
    label: String,
    cover: Option<String>,
}

/// Search results reduced to display-ready entries.
#[derive(Debug, Clone, Default)]
struct SearchPayload {
    albums: Vec<AlbumResult>,
    tracks: Vec<(String, String)>,
    artists: Vec<(String, String)>,
}

pub struct App {
    screen: Screen,
    config: Config,
    status: String,
    signed_in: bool,
    token: Option<String>,

    // Settings form fields.
    token_input: String,

    // Search / add.
    search_query: String,
    url_input: String,
    results: SearchPayload,
    /// Album cover thumbnails, keyed by cover URL, loaded lazily.
    thumbnails: HashMap<String, iced::widget::image::Handle>,

    // Queue.
    pending_jobs: Vec<Job>,
    queue: Vec<QueueItem>,
    index: HashMap<i64, usize>,
    downloading: bool,

    // UI preferences.
    dark_mode: bool,
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
    Download(JobEvent),
    DownloadsFinished,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let config = Config::load().unwrap_or_default();
        let token = auth::load_token().ok().flatten();
        let signed_in = token.is_some();
        let status = if signed_in {
            "Restored saved session.".to_string()
        } else if !config.has_app_credentials() {
            "Enter your Qobuz app_id / app_secret and sign in (Settings).".to_string()
        } else {
            "Sign in on the Settings screen.".to_string()
        };
        let app = App {
            screen: Screen::Settings,
            dark_mode: config.dark_mode,
            show_template_help: false,
            show_credentials_help: false,
            show_account_help: false,
            show_options_help: false,
            token_input: String::new(),
            search_query: String::new(),
            url_input: String::new(),
            results: SearchPayload::default(),
            thumbnails: HashMap::new(),
            pending_jobs: Vec::new(),
            queue: Vec::new(),
            index: HashMap::new(),
            downloading: false,
            token,
            signed_in,
            status,
            config,
        };
        (app, Task::none())
    }

    fn client(&self) -> Result<QobuzClient, String> {
        let mut c = QobuzClient::new(self.config.app_id.clone(), self.config.app_secret.clone())
            .map_err(|e| e.to_string())?
            .with_secret_candidates(self.config.app_secret_candidates.clone());
        if let Some(t) = &self.token {
            c = c.with_token(t.clone());
        }
        Ok(c)
    }

    fn theme(&self) -> Theme {
        style::theme(self.dark_mode)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(s) => {
                self.screen = s;
                Task::none()
            }
            Message::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
                self.config.dark_mode = self.dark_mode;
                let _ = self.config.save();
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
                // A manually entered secret supersedes any auto-detected candidates.
                self.config.app_secret_candidates.clear();
                Task::none()
            }
            Message::AutoDetectCredentials => {
                self.status = "Detecting credentials from the Qobuz web player…".into();
                Task::perform(auto_detect_credentials(), Message::CredentialsDetected)
            }
            Message::CredentialsDetected(Ok(creds)) => {
                self.config.app_id = creds.app_id;
                // Keep the first candidate as the visible secret; the rest are
                // tried automatically when signing.
                let mut secrets = creds.app_secrets.into_iter();
                self.config.app_secret = secrets.next().unwrap_or_default();
                self.config.app_secret_candidates = secrets.collect();
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
            Message::PickDir => Task::perform(pick_dir(), Message::DirChosen),
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
                Task::perform(login_token(id, secret, token), Message::LoggedIn)
            }
            Message::LoggedIn(Ok(token)) => {
                if let Err(e) = auth::store_token(&token) {
                    self.status = format!("Signed in, but token could not be stored: {e}");
                } else {
                    self.status = "Signed in.".into();
                }
                self.token = Some(token);
                self.signed_in = true;
                let _ = self.config.save();
                Task::none()
            }
            Message::LoggedIn(Err(e)) => {
                self.status = format!("Sign-in failed: {e}");
                self.signed_in = false;
                Task::none()
            }
            Message::SignOut => {
                let _ = auth::clear_token();
                self.token = None;
                self.signed_in = false;
                self.status = "Signed out.".into();
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
                Task::perform(do_search(client, q), Message::SearchDone)
            }
            Message::SearchDone(Ok(payload)) => {
                let n = payload.albums.len() + payload.tracks.len() + payload.artists.len();
                self.status = if n == 0 {
                    "No results.".into()
                } else {
                    format!("{n} results.")
                };
                // Lazily load album cover thumbnails not already cached.
                let fetches: Vec<Task<Message>> = payload
                    .albums
                    .iter()
                    .filter_map(|a| a.cover.clone())
                    .filter(|url| !self.thumbnails.contains_key(url))
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .map(|url| {
                        Task::perform(fetch_thumbnail(url.clone()), move |res| {
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
                Task::perform(resolve(client, reference), Message::Resolved)
            }
            Message::Resolved(Ok(jobs)) => {
                let added = jobs.len();
                for job in &jobs {
                    let track_id = job.track.id;
                    if self.index.contains_key(&track_id) {
                        continue;
                    }
                    self.index.insert(track_id, self.queue.len());
                    self.queue.push(QueueItem {
                        title: format!("{} — {}", job.track.artist_name(), job.track.title),
                        status: ItemStatus::Queued,
                        downloaded: 0,
                        total: None,
                    });
                }
                self.pending_jobs.extend(jobs);
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
                if self.downloading {
                    return Task::none();
                }
                if !self.signed_in {
                    self.status = "Sign in before downloading.".into();
                    return Task::none();
                }
                if self.pending_jobs.is_empty() {
                    self.status = "Nothing queued to download.".into();
                    return Task::none();
                }
                let client = match self.client() {
                    Ok(c) => c,
                    Err(e) => {
                        self.status = e;
                        return Task::none();
                    }
                };
                let jobs = std::mem::take(&mut self.pending_jobs);
                let config = self.config.clone();
                self.downloading = true;
                self.status = format!("Downloading {} track(s)…", jobs.len());

                let stream = iced::stream::channel(256, move |mut output| async move {
                    let (tx, mut rx) = tokio::sync::mpsc::channel::<JobEvent>(256);
                    let engine = engine::download_all(client, config, jobs, tx);
                    let drain = async {
                        while let Some(ev) = rx.recv().await {
                            let _ = output.send(Message::Download(ev)).await;
                        }
                    };
                    future::join(engine, drain).await;
                    let _ = output.send(Message::DownloadsFinished).await;
                });
                Task::run(stream, |m| m)
            }
            Message::Download(ev) => {
                self.apply_event(ev);
                Task::none()
            }
            Message::DownloadsFinished => {
                self.downloading = false;
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

    fn apply_event(&mut self, ev: JobEvent) {
        let track_id = match &ev {
            JobEvent::Started { track_id, .. }
            | JobEvent::Progress { track_id, .. }
            | JobEvent::Tagging { track_id }
            | JobEvent::Done { track_id, .. }
            | JobEvent::Failed { track_id, .. } => *track_id,
        };
        let Some(item) = self
            .index
            .get(&track_id)
            .and_then(|&i| self.queue.get_mut(i))
        else {
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
                .font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                })
                .color(a.mauve),
            text("dl")
                .size(style::TEXT_TITLE)
                .font(Font {
                    weight: font::Weight::Bold,
                    ..Font::DEFAULT
                })
                .color(a.blue),
        ]
        .spacing(2);
        let signed_in = self.signed_in;
        let header = row![
            wordmark.width(Length::Fill),
            secondary_button(
                if self.dark_mode {
                    "☀  Light"
                } else {
                    "🌙  Dark"
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
                tab_pane(self.settings_view()),
            )
            .push(
                Screen::Search,
                TabLabel::Text("Search / Add".to_owned()),
                tab_pane(self.search_view()),
            )
            .push(
                Screen::Queue,
                TabLabel::Text("Queue".to_owned()),
                tab_pane(self.queue_view()),
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

    fn settings_view(&self) -> Element<'_, Message> {
        let creds_fields = row![
            field_input("app_id", &self.config.app_id)
                .on_input(Message::AppIdChanged)
                .width(Length::FillPortion(1)),
            field_input("app_secret", &self.config.app_secret)
                .secure(true)
                .on_input(Message::AppSecretChanged)
                .width(Length::FillPortion(1)),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);
        let mut creds_body = column![
            creds_fields,
            row![
                action_button("Auto-detect", Message::AutoDetectCredentials),
                text("Fetch app_id and app_secret from the Qobuz web player.").size(style::TEXT_SM),
            ]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(style::SPACE_SM);
        if self.show_credentials_help {
            creds_body = creds_body.push(credentials_help());
        }

        let mut auth_body = column![row![
            field_input("paste your user_auth_token", &self.token_input)
                .secure(true)
                .on_input(Message::TokenChanged)
                .width(Length::Fill),
            action_button("Sign in", Message::LoginToken),
            secondary_button("Sign out", Message::SignOut),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center),]
        .spacing(style::SPACE_SM);
        if self.show_account_help {
            auth_body = auth_body.push(account_help());
        }

        let dir_row = labeled_row(
            "Download to:",
            row![
                text(self.config.download_dir.display().to_string()).width(Length::Fill),
                secondary_button("Choose…", Message::PickDir),
            ]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center),
        );

        let options_controls = row![
            text("Quality:"),
            pick_list(
                Quality::ALL.to_vec(),
                Some(self.config.quality),
                Message::QualitySelected,
            ),
            checkbox("Embed cover art", self.config.embed_art).on_toggle(Message::EmbedArtToggled),
            iced::widget::horizontal_space(),
            text("Concurrency:"),
            NumberInput::new(
                &self.config.concurrency,
                1..=16,
                Message::ConcurrencyChanged
            )
            .step(1)
            .width(Length::Fixed(120.0)),
        ]
        .spacing(style::SPACE_MD)
        .align_y(iced::Alignment::Center);
        let mut options_body = column![options_controls].spacing(style::SPACE_SM);
        if self.show_options_help {
            options_body = options_body.push(options_help());
        }

        let preview = self.template_preview();
        let mut org_body = column![
            dir_row,
            labeled_row(
                "Folder:",
                field_input("folder format", &self.config.folder_format)
                    .on_input(Message::FolderFormatChanged),
            ),
            labeled_row(
                "Track:",
                field_input("track format", &self.config.track_format)
                    .on_input(Message::TrackFormatChanged),
            ),
            container(text(preview).size(style::TEXT_SM)).padding([style::SPACE_XS, 0]),
        ]
        .spacing(style::SPACE_SM);
        if self.show_template_help {
            org_body = org_body.push(template_help());
        }

        scrollable(
            column![
                help_card(
                    "API credentials",
                    creds_body,
                    |a| a.mauve,
                    self.show_credentials_help,
                    Message::ToggleCredentialsHelp
                ),
                help_card(
                    "Account",
                    auth_body,
                    |a| a.green,
                    self.show_account_help,
                    Message::ToggleAccountHelp
                ),
                help_card(
                    "File organization",
                    org_body,
                    |a| a.teal,
                    self.show_template_help,
                    Message::ToggleTemplateHelp
                ),
                help_card(
                    "Options",
                    options_body,
                    |a| a.peach,
                    self.show_options_help,
                    Message::ToggleOptionsHelp
                ),
                action_button("Save settings", Message::SaveSettings),
            ]
            .spacing(style::SPACE_LG)
            .padding(iced::Padding {
                left: style::SCROLLBAR_GUTTER,
                right: style::SCROLLBAR_GUTTER,
                ..iced::Padding::ZERO
            }),
        )
        .into()
    }

    /// A representative rendered path using the current templates.
    fn template_preview(&self) -> String {
        use qobuz_core::template::{render_path, render_segment, TemplateContext};
        let mut ctx = TemplateContext::new();
        ctx.set("albumartist", "Miles Davis")
            .set("artist", "Miles Davis")
            .set("album", "Kind of Blue")
            .set("title", "So What")
            .set("year", "1959")
            .set("container", self.config.quality.extension().to_uppercase())
            .set("bit_depth", "24")
            .set("sampling_rate", "96")
            .set("explicit", "")
            .with_track_number(1);
        let folder = render_path(&self.config.folder_format, &ctx).join("/");
        let file = render_segment(&self.config.track_format, &ctx);
        format!(
            "Preview: {}/{}.{}",
            folder,
            file,
            self.config.quality.extension()
        )
    }

    fn search_view(&self) -> Element<'_, Message> {
        let search_bar = row![
            field_input("search albums, tracks, artists…", &self.search_query)
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::SearchSubmit)
                .width(Length::Fill),
            action_button("Search", Message::SearchSubmit),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);

        let url_bar = row![
            field_input(
                "paste a Qobuz URL or ID (album / track / playlist)",
                &self.url_input
            )
            .on_input(Message::UrlChanged)
            .on_submit(Message::AddUrl)
            .width(Length::Fill),
            action_button("Add", Message::AddUrl),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);

        let mut results = column![].spacing(style::SPACE_MD);
        if !self.results.albums.is_empty() {
            let mut rows = column![].spacing(style::SPACE_XS);
            for a in &self.results.albums {
                let thumb = a.cover.as_ref().and_then(|u| self.thumbnails.get(u));
                rows = rows.push(album_result_row(a, thumb));
            }
            results = results.push(card("Albums", rows, |a| a.blue));
        }
        if !self.results.tracks.is_empty() {
            let mut rows = column![].spacing(style::SPACE_XS);
            for (id, label) in &self.results.tracks {
                rows = rows.push(result_row(label, Reference::Track(id.clone())));
            }
            results = results.push(card("Tracks", rows, |a| a.green));
        }
        if !self.results.artists.is_empty() {
            let mut rows = column![].spacing(style::SPACE_XS);
            for (id, label) in &self.results.artists {
                rows = rows.push(result_row(label, Reference::Artist(id.clone())));
            }
            results = results.push(card("Artists", rows, |a| a.mauve));
        }

        column![
            section("Add by search"),
            search_bar,
            section("Add by URL / ID"),
            url_bar,
            scrollable(results.padding(iced::Padding {
                left: style::SCROLLBAR_GUTTER,
                right: style::SCROLLBAR_GUTTER,
                ..iced::Padding::ZERO
            }))
            .height(Length::Fill),
        ]
        .spacing(style::SPACE_MD)
        .into()
    }

    fn queue_view(&self) -> Element<'_, Message> {
        let (done, total_bytes, got_bytes) =
            self.queue
                .iter()
                .fold((0usize, 0u64, 0u64), |(d, tb, gb), it| {
                    let d = d + matches!(it.status, ItemStatus::Done(_)) as usize;
                    (d, tb + it.total.unwrap_or(0), gb + it.downloaded)
                });
        let overall = if total_bytes > 0 {
            got_bytes as f32 / total_bytes as f32
        } else if self.queue.is_empty() {
            0.0
        } else {
            done as f32 / self.queue.len() as f32
        };

        let header = row![
            text(format!("{done}/{} complete", self.queue.len())).width(Length::Fill),
            styled_button(if self.downloading {
                "Downloading…"
            } else {
                "Start downloads"
            })
            .on_press_maybe((!self.downloading).then_some(Message::StartDownloads)),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);

        let mut list = column![].spacing(style::SPACE_SM);
        for it in &self.queue {
            list = list.push(queue_row(it));
        }

        column![
            header,
            progress_bar(0.0..=1.0, overall.clamp(0.0, 1.0))
                .height(Length::Fixed(style::PROGRESS_HEIGHT)),
            scrollable(list.padding(iced::Padding {
                left: style::SCROLLBAR_GUTTER,
                right: style::SCROLLBAR_GUTTER,
                ..iced::Padding::ZERO
            }))
            .height(Length::Fill),
        ]
        .spacing(style::SPACE_MD)
        .into()
    }
}

fn section(title: &str) -> Element<'_, Message> {
    text(title).size(style::TEXT_SECTION).into()
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

/// A titled card grouping a section's controls. `head` picks the accent color
/// for the card's header from the active Catppuccin flavor.
fn card<'a>(
    title: &'a str,
    body: impl Into<Element<'a, Message>>,
    head: fn(&style::Accents) -> iced::Color,
) -> Element<'a, Message> {
    card_el(text(title).size(style::TEXT_SECTION), body, head)
}

/// Like [`card`] but with an arbitrary header element (e.g. a title plus a help
/// toggle) instead of a plain title.
fn card_el<'a>(
    head_content: impl Into<Element<'a, Message>>,
    body: impl Into<Element<'a, Message>>,
    head: fn(&style::Accents) -> iced::Color,
) -> Element<'a, Message> {
    Card::new(head_content, body)
        .style(move |theme, _status| {
            let a = style::accents(theme);
            style::card(&a, head(&a))
        })
        .into()
}

/// A settings card whose header carries a right-aligned "?" help toggle.
fn help_card<'a>(
    title: &'a str,
    body: impl Into<Element<'a, Message>>,
    head: fn(&style::Accents) -> iced::Color,
    shown: bool,
    toggle: Message,
) -> Element<'a, Message> {
    let header = row![
        text(title).size(style::TEXT_SECTION).width(Length::Fill),
        style::help_button(shown, toggle),
    ]
    .align_y(iced::Alignment::Center);
    card_el(header, body, head)
}

/// Background/foreground accent selector for a queue item's status badge.
fn badge_palette(status: &ItemStatus) -> fn(&style::Accents) -> (iced::Color, iced::Color) {
    match status {
        ItemStatus::Queued => |a| (a.surface2, a.text),
        ItemStatus::Downloading => |a| (a.blue, a.on_accent),
        ItemStatus::Tagging => |a| (a.yellow, a.on_accent),
        ItemStatus::Done(_) => |a| (a.green, a.on_accent),
        ItemStatus::Error(_) => |a| (a.red, a.on_accent),
    }
}

/// One `mono token — description` row for a help panel.
fn help_term(token: &'static str, desc: &'static str) -> Element<'static, Message> {
    row![
        style::mono(token).width(Length::Fixed(style::LABEL_WIDTH)),
        text(desc).size(style::TEXT_SM),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

/// Help for the API credentials card: what the fields are and how to obtain them.
fn credentials_help() -> Element<'static, Message> {
    column![
        help_term("app_id", "Public client id, sent as the x-app-id request header."),
        help_term(
            "app_secret",
            "Secret used to sign track file-URL requests; never sent as-is.",
        ),
        text("Normally you don't need to do anything here — press \"Auto-detect\" and the app extracts both values from the Qobuz web player for you.")
            .size(style::TEXT_SM),
        text("Manual fallback (if auto-detect fails)").size(style::TEXT_BODY),
        text("1. Open play.qobuz.com in a browser and sign in.").size(style::TEXT_SM),
        text("2. Open the browser developer tools (F12 / ⌥⌘I) → Network tab.").size(style::TEXT_SM),
        text("3. Reload; in any request to the Qobuz API, read the request header \"x-app-id\" — that is your app_id.")
            .size(style::TEXT_SM),
        text("4. In the Sources/Debugger tab, open the player's main JavaScript bundle and search for the app secret (a long hex string used for signing); copy it as app_secret.")
            .size(style::TEXT_SM),
        text("If signing later fails, the web player may have rotated these — press Auto-detect again or re-extract them.")
            .size(style::TEXT_SM),
    ]
    .spacing(style::SPACE_XS)
    .into()
}

/// Help for the Account card: how to obtain the user_auth_token and sign in.
fn account_help() -> Element<'static, Message> {
    column![
        text("Signing in with a token").size(style::TEXT_BODY),
        text("Qobuz sign-in uses your account's user_auth_token (email/password login is not supported — it does not work for partner/bundled accounts such as Qobuz via a telco).")
            .size(style::TEXT_SM),
        text("How to get your token from the Qobuz web player:").size(style::TEXT_SM),
        text("1. Open play.qobuz.com in a browser and sign in normally.").size(style::TEXT_SM),
        text("2. Open the browser developer tools (F12 / ⌥⌘I) → Network tab.").size(style::TEXT_SM),
        text("3. Reload the page, then click any request to the Qobuz API and read the request header \"x-user-auth-token\".")
            .size(style::TEXT_SM),
        text("4. Copy that value, paste it above, and press Sign in.").size(style::TEXT_SM),
        text("• The token is stored in your operating system keyring, not in the config file.")
            .size(style::TEXT_SM),
        text("• The header shows \"● signed in\" once a session is active; Sign out clears the stored token.")
            .size(style::TEXT_SM),
    ]
    .spacing(style::SPACE_XS)
    .into()
}

/// Help for the Options card: quality tiers, concurrency, and cover-art embedding.
fn options_help() -> Element<'static, Message> {
    column![
        text("Options").size(style::TEXT_BODY),
        text("• Quality: MP3 320 · FLAC 16/44.1 (CD) · FLAC 24/≤96 · FLAC 24/≤192 (Hi-Res). The service may deliver a lower tier than requested; the actual quality is read from the response.")
            .size(style::TEXT_SM),
        text("• Concurrency: how many tracks download at once (1–16).").size(style::TEXT_SM),
        text("• Embed cover art: writes the album cover into each downloaded file's tags.")
            .size(style::TEXT_SM),
    ]
    .spacing(style::SPACE_XS)
    .into()
}

/// Example folder templates offered in the help panel (using real placeholders).
const FOLDER_EXAMPLES: &[&str] = &[
    "{albumartist}/{album} ({year})",
    "{albumartist} - {album} [{container}]",
    "{albumartist}/{album} ({year}) [{bit_depth}B-{sampling_rate}kHz]",
];

/// Example track templates offered in the help panel (using real placeholders).
const TRACK_EXAMPLES: &[&str] = &[
    "{tracknumber:02} - {title}",
    "{tracknumber:02}. {artist} - {title}",
    "{artist} - {title}{explicit}",
];

/// One example row: the template string plus Copy and Apply actions.
fn example_row<'a>(template: &'a str, apply: Message) -> Element<'a, Message> {
    row![
        style::mono(template).width(Length::Fill),
        secondary_button("Copy", Message::CopyTemplate(template.to_string())),
        secondary_button("Apply", apply),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

/// The template-syntax help panel: placeholders, rules, and copyable examples.
fn template_help() -> Element<'static, Message> {
    let placeholders = [
        ("{albumartist}", "Album's primary artist"),
        ("{artist}", "Track artist"),
        ("{album}", "Album title"),
        ("{title}", "Track title"),
        ("{year}", "Release year"),
        ("{container}", "Format/extension, e.g. FLAC"),
        ("{bit_depth}", "Bit depth, e.g. 24"),
        ("{sampling_rate}", "Sample rate in kHz, e.g. 96"),
        ("{explicit}", "\" [E]\" for explicit tracks, else empty"),
        ("{composer}", "Composer, when available"),
        ("{tracknumber}", "Track number; pad with {tracknumber:02}"),
    ];
    let mut list = column![].spacing(style::SPACE_XS);
    for (token, desc) in placeholders {
        list = list.push(
            row![
                style::mono(token).width(Length::Fixed(style::LABEL_WIDTH + 40.0)),
                text(desc).size(style::TEXT_SM),
            ]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center),
        );
    }

    let rules = column![
        text("Syntax").size(style::TEXT_BODY),
        text("• Use {placeholder} tokens; unknown tokens render as empty text.")
            .size(style::TEXT_SM),
        text("• Zero-pad numbers with {name:0N}, e.g. {tracknumber:02} → 01.").size(style::TEXT_SM),
        text("• In the folder format, \"/\" creates nested subfolders.").size(style::TEXT_SM),
        text("• Illegal filename characters are replaced automatically.").size(style::TEXT_SM),
    ]
    .spacing(style::SPACE_XS);

    let mut folder_ex = column![section("Folder examples")].spacing(style::SPACE_XS);
    for &t in FOLDER_EXAMPLES {
        folder_ex = folder_ex.push(example_row(t, Message::FolderFormatChanged(t.to_string())));
    }
    let mut track_ex = column![section("Track examples")].spacing(style::SPACE_XS);
    for &t in TRACK_EXAMPLES {
        track_ex = track_ex.push(example_row(t, Message::TrackFormatChanged(t.to_string())));
    }

    let body = column![section("Placeholders"), list, rules, folder_ex, track_ex,]
        .spacing(style::SPACE_SM);
    Card::new(text("Template help").size(style::TEXT_SECTION), body)
        .style(|theme, _status| {
            let a = style::accents(theme);
            style::card(&a, a.sky)
        })
        .into()
}

fn result_row<'a>(label: &'a str, reference: Reference) -> Element<'a, Message> {
    row![
        text(label).width(Length::Fill),
        secondary_button("Add", Message::Add(reference)),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

/// An album result row with its cover thumbnail (or a placeholder while loading).
fn album_result_row<'a>(
    album: &'a AlbumResult,
    thumb: Option<&image::Handle>,
) -> Element<'a, Message> {
    const SIZE: f32 = 52.0;
    let cover: Element<'a, Message> = match thumb {
        Some(handle) => image(handle.clone())
            .width(Length::Fixed(SIZE))
            .height(Length::Fixed(SIZE))
            .into(),
        None => container(text(""))
            .width(Length::Fixed(SIZE))
            .height(Length::Fixed(SIZE))
            .style(style::thumb_placeholder)
            .into(),
    };
    row![
        cover,
        text(&album.label).width(Length::Fill),
        secondary_button("Add", Message::Add(Reference::Album(album.id.clone()))),
    ]
    .spacing(style::SPACE_SM)
    .align_y(iced::Alignment::Center)
    .into()
}

fn queue_row(it: &QueueItem) -> Element<'_, Message> {
    let (status_text, fraction): (String, f32) = match &it.status {
        ItemStatus::Queued => ("queued".into(), 0.0),
        ItemStatus::Downloading => {
            let f = match it.total {
                Some(t) if t > 0 => it.downloaded as f32 / t as f32,
                _ => 0.0,
            };
            // Pad to a constant width so the badge doesn't shift as digits change.
            (format!("downloading {:>3.0}%", f * 100.0), f)
        }
        ItemStatus::Tagging => ("tagging".into(), 1.0),
        ItemStatus::Done(q) => (format!("done · {q}"), 1.0),
        ItemStatus::Error(e) => (format!("error: {e}"), 0.0),
    };

    column![
        row![text(&it.title).width(Length::Fill), {
            let pick = badge_palette(&it.status);
            // Monospace so the padded percentage keeps a constant width (the
            // default font's digits vary in width and shift the badge).
            Badge::new(text(status_text).size(style::TEXT_SM).font(Font::MONOSPACE)).style(
                move |theme, _status| {
                    let a = style::accents(theme);
                    let (bg, fg) = pick(&a);
                    style::badge(bg, fg)
                },
            )
        },]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center),
        progress_bar(0.0..=1.0, fraction.clamp(0.0, 1.0))
            .height(Length::Fixed(style::PROGRESS_HEIGHT)),
    ]
    .spacing(style::SPACE_XS)
    .into()
}

// ---- Async helpers ------------------------------------------------------

async fn pick_dir() -> Option<PathBuf> {
    rfd::AsyncFileDialog::new()
        .pick_folder()
        .await
        .map(|h| h.path().to_path_buf())
}

async fn auto_detect_credentials() -> Result<AppCredentials, String> {
    qobuz_core::discover_app_credentials()
        .await
        .map_err(|e| e.to_string())
}

/// Validate a pasted `user_auth_token` and return it on success.
async fn login_token(app_id: String, app_secret: String, token: String) -> Result<String, String> {
    let mut c = QobuzClient::new(app_id, app_secret).map_err(|e| e.to_string())?;
    c.login_with_token(&token)
        .await
        .map_err(|e| e.to_string())?;
    Ok(token)
}

async fn do_search(client: QobuzClient, query: String) -> Result<SearchPayload, String> {
    let r = client.search(&query, 25).await.map_err(|e| e.to_string())?;
    let mut payload = SearchPayload::default();
    if let Some(list) = r.albums {
        for a in list.items {
            let label = format!("{} — {}", a.artist_name(), a.title);
            // Prefer a small image for the thumbnail to keep downloads cheap.
            let cover = a.image.as_ref().and_then(|i| {
                i.small
                    .clone()
                    .or_else(|| i.thumbnail.clone())
                    .or_else(|| i.large.clone())
            });
            payload.albums.push(AlbumResult {
                id: a.id,
                label,
                cover,
            });
        }
    }
    if let Some(list) = r.tracks {
        for t in list.items {
            let label = format!("{} — {}", t.artist_name(), t.title);
            payload.tracks.push((t.id.to_string(), label));
        }
    }
    if let Some(list) = r.artists {
        for a in list.items {
            payload.artists.push((a.id.to_string(), a.name));
        }
    }
    Ok(payload)
}

/// Download the bytes of an album cover thumbnail via the core client.
async fn fetch_thumbnail(url: String) -> Result<Vec<u8>, ()> {
    qobuz_core::fetch_bytes(&url).await.map_err(|_| ())
}

async fn resolve(client: QobuzClient, reference: Reference) -> Result<Vec<Job>, String> {
    engine::resolve(&client, &reference)
        .await
        .map_err(|e| e.to_string())
}
