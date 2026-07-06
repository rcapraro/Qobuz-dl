//! The iced desktop application: settings, search/add, and download queue.

use iced::futures::{future, SinkExt};
use iced::widget::{
    button, checkbox, column, container, pick_list, progress_bar, row, scrollable, text,
    Space,
};
use iced::{Element, Length, Task, Theme};
use qobuz_core::catalog::Reference;
use qobuz_core::config::Config;
use qobuz_core::engine::{Job, JobEvent};
use qobuz_core::quality::Quality;
use qobuz_core::{auth, engine, QobuzClient};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::style::{self, action_button, field_input, labeled_row, secondary_button, styled_button};

pub fn run() -> iced::Result {
    iced::application("Qobuz-dl", App::update, App::view)
        .theme(App::theme)
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

/// Search results reduced to (id, label) pairs for display.
#[derive(Debug, Clone, Default)]
struct SearchPayload {
    albums: Vec<(String, String)>,
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
    email: String,
    password: String,
    token_input: String,
    concurrency: String,

    // Search / add.
    search_query: String,
    url_input: String,
    results: SearchPayload,

    // Queue.
    pending_jobs: Vec<Job>,
    queue: Vec<QueueItem>,
    index: HashMap<i64, usize>,
    downloading: bool,

    // UI preferences.
    dark_mode: bool,
    show_template_help: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Navigate(Screen),
    ToggleTheme,
    ToggleTemplateHelp,
    CopyTemplate(String),

    // Settings inputs.
    EmailChanged(String),
    PasswordChanged(String),
    TokenChanged(String),
    AppIdChanged(String),
    AppSecretChanged(String),
    FolderFormatChanged(String),
    TrackFormatChanged(String),
    ConcurrencyChanged(String),
    QualitySelected(Quality),
    EmbedArtToggled(bool),
    PickDir,
    DirChosen(Option<PathBuf>),
    SaveSettings,
    LoginPassword,
    LoginToken,
    LoggedIn(Result<String, String>),
    SignOut,

    // Search / add.
    SearchQueryChanged(String),
    SearchSubmit,
    SearchDone(Result<SearchPayload, String>),
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
            concurrency: config.concurrency.to_string(),
            dark_mode: config.dark_mode,
            show_template_help: false,
            email: String::new(),
            password: String::new(),
            token_input: String::new(),
            search_query: String::new(),
            url_input: String::new(),
            results: SearchPayload::default(),
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
            .map_err(|e| e.to_string())?;
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
            Message::CopyTemplate(t) => iced::clipboard::write(t),

            // ---- Settings inputs ----
            Message::EmailChanged(v) => {
                self.email = v;
                Task::none()
            }
            Message::PasswordChanged(v) => {
                self.password = v;
                Task::none()
            }
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
            Message::ConcurrencyChanged(v) => {
                self.concurrency = v.clone();
                if let Ok(n) = v.trim().parse::<usize>() {
                    self.config.concurrency = n.clamp(1, 16);
                }
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
            Message::LoginPassword => {
                if !self.config.has_app_credentials() {
                    self.status = "Enter app_id and app_secret first.".into();
                    return Task::none();
                }
                let (id, secret) = (self.config.app_id.clone(), self.config.app_secret.clone());
                let (email, pw) = (self.email.clone(), self.password.clone());
                self.status = "Signing in…".into();
                Task::perform(login_password(id, secret, email, pw), Message::LoggedIn)
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
                self.password.clear();
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
                self.results = payload;
                Task::none()
            }
            Message::SearchDone(Err(e)) => {
                self.status = format!("Search failed: {e}");
                Task::none()
            }
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
        let Some(item) = self.index.get(&track_id).and_then(|&i| self.queue.get_mut(i)) else {
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
        let nav = row![
            nav_button("Settings", Screen::Settings, self.screen),
            nav_button("Search / Add", Screen::Search, self.screen),
            nav_button("Queue", Screen::Queue, self.screen),
            Space::with_width(Length::Fill),
            secondary_button(
                if self.dark_mode { "☀  Light" } else { "🌙  Dark" },
                Message::ToggleTheme,
            ),
            text(if self.signed_in {
                "● signed in"
            } else {
                "○ signed out"
            }),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);

        let body = match self.screen {
            Screen::Settings => self.settings_view(),
            Screen::Search => self.search_view(),
            Screen::Queue => self.queue_view(),
        };

        let content = column![
            nav,
            container(text(&self.status)).padding([style::SPACE_XS, 0]),
            container(body).height(Length::Fill),
        ]
        .spacing(style::SPACE_MD)
        .padding(style::SPACE_XL);

        content.into()
    }

    fn settings_view(&self) -> Element<'_, Message> {
        let auth_section = column![
            section("Account"),
            row![
                field_input("email", &self.email)
                    .on_input(Message::EmailChanged)
                    .width(Length::FillPortion(1)),
                field_input("password", &self.password)
                    .secure(true)
                    .on_input(Message::PasswordChanged)
                    .width(Length::FillPortion(1)),
                action_button("Sign in", Message::LoginPassword),
            ]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center),
            row![
                field_input("or paste a user_auth_token", &self.token_input)
                    .on_input(Message::TokenChanged)
                    .width(Length::Fill),
                action_button("Use token", Message::LoginToken),
                secondary_button("Sign out", Message::SignOut),
            ]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(style::SPACE_SM);

        let creds_section = column![
            section("API credentials"),
            row![
                field_input("app_id", &self.config.app_id)
                    .on_input(Message::AppIdChanged)
                    .width(Length::FillPortion(1)),
                field_input("app_secret", &self.config.app_secret)
                    .secure(true)
                    .on_input(Message::AppSecretChanged)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(style::SPACE_SM);

        let dir_row = labeled_row(
            "Download to:",
            row![
                text(self.config.download_dir.display().to_string()).width(Length::Fill),
                secondary_button("Choose…", Message::PickDir),
            ]
            .spacing(style::SPACE_SM)
            .align_y(iced::Alignment::Center),
        );

        let options_row = row![
            text("Quality:"),
            pick_list(
                Quality::ALL.to_vec(),
                Some(self.config.quality),
                Message::QualitySelected,
            ),
            checkbox("Embed cover art", self.config.embed_art).on_toggle(Message::EmbedArtToggled),
            text("Concurrency:"),
            field_input("3", &self.concurrency)
                .on_input(Message::ConcurrencyChanged)
                .width(Length::Fixed(60.0)),
        ]
        .spacing(style::SPACE_MD)
        .align_y(iced::Alignment::Center);

        let preview = self.template_preview();
        let help_toggle = secondary_button(
            if self.show_template_help {
                "Hide template help"
            } else {
                "Show template help"
            },
            Message::ToggleTemplateHelp,
        );
        let mut templates_section = column![
            section("File organization"),
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
            help_toggle,
        ]
        .spacing(style::SPACE_SM);
        if self.show_template_help {
            templates_section = templates_section.push(template_help());
        }

        scrollable(
            column![
                creds_section,
                auth_section,
                templates_section,
                options_row,
                action_button("Save settings", Message::SaveSettings),
            ]
            .spacing(style::SPACE_LG),
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
            field_input("paste a Qobuz URL or ID (album / track / playlist)", &self.url_input)
                .on_input(Message::UrlChanged)
                .on_submit(Message::AddUrl)
                .width(Length::Fill),
            action_button("Add", Message::AddUrl),
        ]
        .spacing(style::SPACE_SM)
        .align_y(iced::Alignment::Center);

        let mut results = column![].spacing(style::SPACE_SM);
        if !self.results.albums.is_empty() {
            results = results.push(section("Albums"));
            for (id, label) in &self.results.albums {
                results = results.push(result_row(label, Reference::Album(id.clone())));
            }
        }
        if !self.results.tracks.is_empty() {
            results = results.push(section("Tracks"));
            for (id, label) in &self.results.tracks {
                results = results.push(result_row(label, Reference::Track(id.clone())));
            }
        }
        if !self.results.artists.is_empty() {
            results = results.push(section("Artists"));
            for (id, label) in &self.results.artists {
                results = results.push(result_row(label, Reference::Artist(id.clone())));
            }
        }

        column![
            section("Add by search"),
            search_bar,
            section("Add by URL / ID"),
            url_bar,
            scrollable(results).height(Length::Fill),
        ]
        .spacing(style::SPACE_MD)
        .into()
    }

    fn queue_view(&self) -> Element<'_, Message> {
        let (done, total_bytes, got_bytes) = self.queue.iter().fold(
            (0usize, 0u64, 0u64),
            |(d, tb, gb), it| {
                let d = d + matches!(it.status, ItemStatus::Done(_)) as usize;
                (d, tb + it.total.unwrap_or(0), gb + it.downloaded)
            },
        );
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
            progress_bar(0.0..=1.0, overall.clamp(0.0, 1.0)).height(Length::Fixed(style::PROGRESS_HEIGHT)),
            scrollable(list).height(Length::Fill),
        ]
        .spacing(style::SPACE_MD)
        .into()
    }
}

fn nav_button(label: &str, target: Screen, current: Screen) -> Element<'_, Message> {
    let b = button(text(label)).on_press(Message::Navigate(target));
    if target == current {
        b.into()
    } else {
        b.style(button::secondary).into()
    }
}

fn section(title: &str) -> Element<'_, Message> {
    text(title).size(style::TEXT_SECTION).into()
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
        text("• Use {placeholder} tokens; unknown tokens render as empty text.").size(style::TEXT_SM),
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

    column![
        section("Placeholders"),
        list,
        rules,
        folder_ex,
        track_ex,
    ]
    .spacing(style::SPACE_SM)
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

fn queue_row(it: &QueueItem) -> Element<'_, Message> {
    let (status_text, fraction): (String, f32) = match &it.status {
        ItemStatus::Queued => ("queued".into(), 0.0),
        ItemStatus::Downloading => {
            let f = match it.total {
                Some(t) if t > 0 => it.downloaded as f32 / t as f32,
                _ => 0.0,
            };
            (format!("downloading {:.0}%", f * 100.0), f)
        }
        ItemStatus::Tagging => ("tagging".into(), 1.0),
        ItemStatus::Done(q) => (format!("done · {q}"), 1.0),
        ItemStatus::Error(e) => (format!("error: {e}"), 0.0),
    };

    column![
        row![
            text(&it.title).width(Length::Fill),
            text(status_text).size(style::TEXT_SM),
        ]
        .spacing(style::SPACE_SM),
        progress_bar(0.0..=1.0, fraction.clamp(0.0, 1.0)).height(Length::Fixed(style::PROGRESS_HEIGHT)),
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

async fn login_password(
    app_id: String,
    app_secret: String,
    email: String,
    password: String,
) -> Result<String, String> {
    let mut c = QobuzClient::new(app_id, app_secret).map_err(|e| e.to_string())?;
    c.login(&email, &password).await.map_err(|e| e.to_string())
}

async fn login_token(
    app_id: String,
    app_secret: String,
    token: String,
) -> Result<String, String> {
    let mut c = QobuzClient::new(app_id, app_secret).map_err(|e| e.to_string())?;
    c.login_with_token(&token).await.map_err(|e| e.to_string())?;
    Ok(token)
}

async fn do_search(client: QobuzClient, query: String) -> Result<SearchPayload, String> {
    let r = client.search(&query, 25).await.map_err(|e| e.to_string())?;
    let mut payload = SearchPayload::default();
    if let Some(list) = r.albums {
        for a in list.items {
            let label = format!("{} — {}", a.artist_name(), a.title);
            payload.albums.push((a.id, label));
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

async fn resolve(client: QobuzClient, reference: Reference) -> Result<Vec<Job>, String> {
    engine::resolve(&client, &reference)
        .await
        .map_err(|e| e.to_string())
}
