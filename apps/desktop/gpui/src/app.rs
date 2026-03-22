use std::collections::HashSet;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::Duration;

use aurelia_core::media_controls::MediaControlsState;
use aurelia_core::models::{
    Album, Artist, AuthRequest, BackendProvider, Credentials, ParsedLyrics, Playlist,
    PlaylistCreateData, PlaylistUpdateData, Song,
};
use gpui::{
    actions, div, img, linear_color_stop, linear_gradient, prelude::*, px, relative, rgb, rgba,
    size, uniform_list, AnyElement, App, AsyncApp, Application, Bounds, Context, Entity,
    Focusable, FocusHandle, Image as GpuiImage, ImageFormat as GpuiImageFormat, MouseButton,
    ObjectFit, Point, ScrollHandle, ScrollStrategy, SharedString, StatefulInteractiveElement,
    UniformListScrollHandle, WeakEntity, Window, WindowBackgroundAppearance, WindowBounds,
    WindowOptions,
};
use gpui::http_client::{AsyncBody, HttpClient, Request, Response, Url};
use gpui_component::avatar::Avatar;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::input::{Input, InputEvent, InputState};
use gpui_component::scroll::ScrollableElement;
use gpui_component::slider::{Slider, SliderEvent, SliderState};
use gpui_component::switch::Switch;
use gpui_component::{
    TitleBar, h_flex, v_flex, Icon, IconName, IconNamed, Root, Sizable,
};
use raw_window_handle::RawWindowHandle;
use tokio::runtime::Runtime;
use zed_reqwest as reqwest;

use crate::assets::Assets;
use image::imageops::FilterType;
use rand::seq::SliceRandom;

// Custom icon names for our SVGs in assets/icons/
#[derive(Clone, Copy)]
enum AppIcon {
    Play,
    Pause,
    SkipForward,
    SkipBack,
    Stop,
    Shuffle,
    Repeat,
    Volume2,
    Volume1,
    VolumeX,
    Music,
    Disc3,
    ListMusic,
    MicVocal,
    House,
    LogOut,
    RefreshCw,
    Users,
    HeartCustom,
    Download,
}

impl IconNamed for AppIcon {
    fn path(self) -> SharedString {
        match self {
            Self::Play => "icons/play.svg".into(),
            Self::Pause => "icons/pause.svg".into(),
            Self::SkipForward => "icons/skip-forward.svg".into(),
            Self::SkipBack => "icons/skip-back.svg".into(),
            Self::Stop => "icons/square.svg".into(),
            Self::Shuffle => "icons/shuffle.svg".into(),
            Self::Repeat => "icons/repeat.svg".into(),
            Self::Volume2 => "icons/volume-2.svg".into(),
            Self::Volume1 => "icons/volume-1.svg".into(),
            Self::VolumeX => "icons/volume-x.svg".into(),
            Self::Music => "icons/music.svg".into(),
            Self::Disc3 => "icons/disc-3.svg".into(),
            Self::ListMusic => "icons/list-music.svg".into(),
            Self::MicVocal => "icons/mic-vocal.svg".into(),
            Self::House => "icons/house.svg".into(),
            Self::LogOut => "icons/log-out.svg".into(),
            Self::RefreshCw => "icons/refresh-cw.svg".into(),
            Self::Users => "icons/users.svg".into(),
            Self::HeartCustom => "icons/heart.svg".into(),
            Self::Download => "icons/download.svg".into(),
        }
    }
}

actions!(aurelia, [Quit]);

const APP_NAME: &str = "Aurelia GPUI";

const APP_DIR_NAME: &str = "aurelia-gpui";

// -- Design System (matches web default-dark / zinc palette) --

mod theme {
    pub const BACKGROUND: u32 = 0x09090b;
    pub const BACKGROUND_DARK: u32 = 0x030304;
    pub const FOREGROUND: u32 = 0xfafafa;
    pub const CARD: u32 = 0x121215;
    pub const BORDER: u32 = 0x27272a;
    pub const MUTED_FG: u32 = 0x9f9fa9;
    pub const SIDEBAR_BG: u32 = 0x000000;
    pub const SIDEBAR_ITEM_ACTIVE: u32 = 0x27272a;
    pub const SIDEBAR_ITEM_HOVER: u32 = 0x18181b;
    pub const PLAYER_BG: u32 = 0x030304;
}

fn format_duration(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let m = total / 60;
    let s = total % 60;
    format!("{m}:{s:02}")
}

fn format_duration_long(secs: f64) -> String {
    let total_minutes = (secs.max(0.0) / 60.0).round() as u64;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    match (hours, minutes) {
        (0, m) => format!("{m} min"),
        (h, 0) => format!("{h} hr"),
        (h, m) => format!("{h} hr {m} min"),
    }
}

fn format_song_file_info(song: &Song) -> String {
    let mut parts = Vec::new();

    if let Some(container) = song.container.as_ref().filter(|value| !value.trim().is_empty()) {
        parts.push(container.to_lowercase());
    }

    if let Some(sample_rate) = song.sample_rate.filter(|value| *value > 0) {
        let khz = sample_rate as f64 / 1000.0;
        let formatted = if (khz.fract() - 0.0).abs() < f64::EPSILON {
            format!("{khz:.0}khz")
        } else {
            format!("{khz:.1}khz")
        };
        parts.push(formatted);
    }

    if let Some(bit_rate) = song.bit_rate.filter(|value| *value > 0) {
        parts.push(format!("{}kbps", bit_rate / 1000));
    }

    if parts.is_empty() {
        "Unknown format".to_string()
    } else {
        parts.join(" / ")
    }
}

pub fn run() {
    Application::new()
        .with_assets(Assets)
        .run(|cx: &mut App| {
            if let Ok(http_client) = ReqwestHttpClient::new("aurelia-gpui") {
                cx.set_http_client(Arc::new(http_client));
            }
            gpui_component::init(cx);

            cx.bind_keys([gpui::KeyBinding::new("cmd-q", Quit, None)]);

            let bounds = Bounds::centered(None, size(px(1400.0), px(900.0)), cx);
            let window = cx
                .open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        window_background: WindowBackgroundAppearance::Opaque,
                        titlebar: Some(gpui::TitlebarOptions {
                            title: Some(APP_NAME.into()),
                            appears_transparent: true,
                            traffic_light_position: Some(gpui::point(px(9.0), px(9.0))),
                        }),
                        ..Default::default()
                    },
                    |window, cx| {
                        let view = cx.new(|cx| DesktopApp::new(window, cx));
                        cx.new(|cx| Root::new(view, window, cx))
                    },
                )
                .expect("failed to open GPUI window");

            window
                .update(cx, |_root, _window, _cx| {
                    // Root handles focus management
                })
                .ok();

            cx.activate(true);
            cx.on_action(|_: &Quit, cx| cx.quit());
        });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewTab {
    Home,
    Songs,
    Albums,
    Artists,
    Playlists,
    Search,
    Settings,
}

impl ViewTab {
    const ALL: [Self; 7] = [
        Self::Home,
        Self::Songs,
        Self::Albums,
        Self::Artists,
        Self::Playlists,
        Self::Search,
        Self::Settings,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Songs => "Songs",
            Self::Albums => "Albums",
            Self::Artists => "Artists",
            Self::Playlists => "Playlists",
            Self::Search => "Search",
            Self::Settings => "Settings",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Self::Home => Icon::new(AppIcon::House),
            Self::Songs => Icon::new(AppIcon::Music),
            Self::Albums => Icon::new(AppIcon::Disc3),
            Self::Artists => Icon::new(AppIcon::Users),
            Self::Playlists => Icon::new(AppIcon::ListMusic),
            Self::Search => Icon::new(IconName::Search),
            Self::Settings => Icon::new(IconName::Settings),
        }
    }
}

#[derive(Clone, Debug)]
struct SessionData {
    credentials: Credentials,
}

#[derive(Clone, Debug, Default)]
struct LibrarySnapshot {
    songs: Vec<Song>,
    albums: Vec<Album>,
    artists: Vec<Artist>,
    playlists: Vec<Playlist>,
    recent: Vec<Song>,
}

#[derive(Clone, Debug, Default)]
struct PlayerState {
    original_queue: Vec<Song>,
    queue: Vec<Song>,
    current_index: Option<usize>,
    is_playing: bool,
    position_secs: f64,
    duration_secs: f64,
    volume: f64,
    is_shuffled: bool,
    repeat_mode: RepeatMode,
}

impl PlayerState {
    fn current_song(&self) -> Option<&Song> {
        self.current_index.and_then(|index| self.queue.get(index))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum RightPanel {
    #[default]
    None,
    Queue,
    Lyrics,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum RepeatMode {
    #[default]
    None,
    All,
    One,
}

#[derive(Clone, Debug)]
struct DesktopAppState {
    app_data_dir: Arc<str>,
    session: Option<SessionData>,
    library: LibrarySnapshot,
    selected_tab: ViewTab,
    right_panel: RightPanel,
    selected_song_id: Option<String>,
    selected_album_id: Option<String>,
    selected_artist_id: Option<String>,
    selected_playlist_id: Option<String>,
    search_query: String,
    status: String,
    sync_status: String,
    playback_status: String,
    lyrics_status: String,
    lyrics: Option<ParsedLyrics>,
    player: PlayerState,
    favorite_ids: Vec<String>,
    minimize_to_tray: bool,
    close_to_tray: bool,
    lyrics_server_url: String,
    // Pending clears for InputState fields (workaround: async closures lack &mut Window)
    pending_clear_password: bool,
    pending_clear_playlist_name: bool,
}

impl DesktopAppState {
    fn new(app_data_dir: Arc<str>) -> Self {
        Self {
            app_data_dir,
            session: None,
            library: LibrarySnapshot::default(),
            selected_tab: ViewTab::Songs,
            right_panel: RightPanel::None,
            selected_song_id: None,
            selected_album_id: None,
            selected_artist_id: None,
            selected_playlist_id: None,
            search_query: String::new(),
            status: "Starting up...".to_string(),
            sync_status: "Idle".to_string(),
            playback_status: "Stopped".to_string(),
            lyrics_status: "No lyrics loaded".to_string(),
            lyrics: None,
            player: PlayerState {
                volume: 1.0,
                ..Default::default()
            },
            favorite_ids: Vec::new(),
            minimize_to_tray: false,
            close_to_tray: false,
            lyrics_server_url: String::new(),
            pending_clear_password: false,
            pending_clear_playlist_name: false,
        }
    }

    fn selected_song(&self) -> Option<&Song> {
        self.selected_song_id
            .as_ref()
            .and_then(|id| self.library.songs.iter().find(|song| &song.id == id))
    }

    fn selected_album(&self) -> Option<&Album> {
        self.selected_album_id
            .as_ref()
            .and_then(|id| self.library.albums.iter().find(|album| album.id.as_deref() == Some(id.as_str())))
    }

    fn selected_artist(&self) -> Option<&Artist> {
        self.selected_artist_id
            .as_ref()
            .and_then(|id| self.library.artists.iter().find(|artist| artist.id == *id))
    }

    fn songs_for_album(&self, album_id: &str) -> Vec<Song> {
        let mut songs = self
            .library
            .songs
            .iter()
            .filter(|song| song.album_id.as_deref() == Some(album_id))
            .cloned()
            .collect::<Vec<_>>();
        songs.sort_by_key(|song| (song.disc_number.unwrap_or(0), song.track_number.unwrap_or(0)));
        songs
    }

    fn songs_for_artist_id(&self, artist_id: &str) -> Vec<Song> {
        let artist_name = self
            .library
            .artists
            .iter()
            .find(|artist| artist.id == artist_id)
            .map(|artist| artist.name.to_lowercase());

        let mut songs = self
            .library
            .songs
            .iter()
            .filter(|song| {
                song.artist_ids
                    .as_ref()
                    .is_some_and(|ids| ids.iter().any(|id| id == artist_id))
                    || artist_name.as_ref().is_some_and(|name| {
                        song.artists
                            .as_ref()
                            .is_some_and(|artists| artists.iter().any(|artist| artist.to_lowercase() == *name))
                    })
            })
            .cloned()
            .collect::<Vec<_>>();
        songs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        songs
    }

    fn albums_for_artist_id(&self, artist_id: &str) -> Vec<Album> {
        let mut album_ids = HashSet::new();
        let mut albums = Vec::new();

        for song in self.songs_for_artist_id(artist_id) {
            let Some(album_id) = song.album_id.as_ref() else {
                continue;
            };
            if !album_ids.insert(album_id.clone()) {
                continue;
            }
            if let Some(album) = self
                .library
                .albums
                .iter()
                .find(|album| album.id.as_deref() == Some(album_id.as_str()))
            {
                albums.push(album.clone());
            }
        }

        albums.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        albums
    }

    fn recently_added_albums(&self) -> Vec<Album> {
        let mut albums = self.library.albums.clone();
        albums.sort_by(|a, b| {
            b.date_created
                .as_ref()
                .or(b.date_modified.as_ref())
                .cmp(&a.date_created.as_ref().or(a.date_modified.as_ref()))
        });
        albums
    }

    fn featured_album(&self) -> Option<Album> {
        self.player
            .current_song()
            .and_then(|song| song.album_id.as_ref())
            .and_then(|album_id| {
                self.library
                    .albums
                    .iter()
                    .find(|album| album.id.as_deref() == Some(album_id))
            })
            .cloned()
            .or_else(|| {
                self.library.recent.iter().find_map(|song| {
                    song.album_id.as_ref().and_then(|album_id| {
                        self.library
                            .albums
                            .iter()
                            .find(|album| album.id.as_deref() == Some(album_id))
                            .cloned()
                    })
                })
            })
            .or_else(|| self.recently_added_albums().into_iter().next())
    }

    fn home_highlight_albums(&self) -> Vec<Album> {
        let mut seen = HashSet::new();
        let mut albums = Vec::new();

        for song in &self.library.recent {
            let Some(album_id) = song.album_id.as_ref() else {
                continue;
            };
            if !seen.insert(album_id.clone()) {
                continue;
            }
            if let Some(album) = self
                .library
                .albums
                .iter()
                .find(|album| album.id.as_deref() == Some(album_id.as_str()))
            {
                albums.push(album.clone());
            }
        }

        for album in self.recently_added_albums() {
            if let Some(id) = album.id.as_ref() {
                if seen.insert(id.clone()) {
                    albums.push(album);
                }
            }
            if albums.len() >= 12 {
                break;
            }
        }

        albums
    }

    fn filtered_songs(&self) -> Vec<Song> {
        let query = self.search_query.trim().to_lowercase();
        let mut songs = self.library.songs.clone();
        songs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        if query.is_empty() {
            return songs;
        }

        songs
            .into_iter()
            .filter(|song| {
                song.name.to_lowercase().contains(&query)
                    || song
                        .album
                        .as_ref()
                        .is_some_and(|album| album.to_lowercase().contains(&query))
                    || song.artists.as_ref().is_some_and(|artists| {
                        artists
                            .iter()
                            .any(|artist| artist.to_lowercase().contains(&query))
                    })
            })
            .collect()
    }
}

struct DesktopApp {
    login_server: Entity<InputState>,
    login_username: Entity<InputState>,
    login_password: Entity<InputState>,
    playlist_name: Entity<InputState>,
    search_input: Entity<InputState>,
    lyrics_server_input: Entity<InputState>,
    seek_slider: Entity<SliderState>,
    volume_slider: Entity<SliderState>,
    songs_scroll_handle: UniformListScrollHandle,
    recent_songs_scroll_handle: ScrollHandle,
    recent_albums_scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
    state: DesktopAppState,
    featured_background_source_url: Option<String>,
    featured_background_image: Option<Arc<GpuiImage>>,
    featured_background_loading: bool,
    runtime: Arc<Runtime>,
    media_controls: Arc<MediaControlsState>,
}

impl DesktopApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let app_data_dir = Arc::<str>::from(resolve_app_data_dir().to_string_lossy().to_string());
        let runtime = Arc::new(
            Runtime::new().expect("failed to create tokio runtime for GPUI desktop app"),
        );
        let media_controls = Arc::new(MediaControlsState::new());
        let hwnd = raw_hwnd(window);
        let _ = media_controls.init(hwnd);

        let login_server = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Server URL")
        });
        let login_username = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Username")
        });
        let login_password = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Password")
                .masked(true)
        });
        let playlist_name = cx.new(|cx| {
            InputState::new(window, cx).placeholder("New playlist name")
        });
        let search_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Search...")
        });
        let lyrics_server_input = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Lyrics server URL")
        });

        // Subscribe to search input changes
        cx.subscribe(&search_input, |this: &mut Self, _entity, event: &InputEvent, cx| {
            match event {
                InputEvent::Change | InputEvent::PressEnter { .. } => {
                    this.use_search_text(cx);
                }
                _ => {}
            }
        })
        .detach();

        // Subscribe to login inputs for Enter key
        cx.subscribe(&login_server, |this: &mut Self, _entity, event: &InputEvent, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) && this.state.session.is_none() {
                this.login(cx);
            }
        })
        .detach();
        cx.subscribe(&login_username, |this: &mut Self, _entity, event: &InputEvent, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) && this.state.session.is_none() {
                this.login(cx);
            }
        })
        .detach();
        cx.subscribe(&login_password, |this: &mut Self, _entity, event: &InputEvent, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) && this.state.session.is_none() {
                this.login(cx);
            }
        })
        .detach();
        cx.subscribe(&lyrics_server_input, |this: &mut Self, _entity, event: &InputEvent, cx| {
            if matches!(event, InputEvent::PressEnter { .. }) {
                this.save_lyrics_server_url(cx);
            }
        })
        .detach();

        let seek_slider = cx.new(|_| SliderState::new().min(0.0).max(1000.0).step(1.0).default_value(0.0));

        cx.subscribe(&seek_slider, |this: &mut Self, _entity, event: &SliderEvent, cx| {
            let SliderEvent::Change(value) = event;
            let progress = value.end() / 1000.0;
            this.seek_to(progress as f64, cx);
        })
        .detach();

        // Volume slider
        let volume_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(1.0)
                .default_value(100.0)
        });
        let songs_scroll_handle = UniformListScrollHandle::new();
        let recent_songs_scroll_handle = ScrollHandle::new();
        let recent_albums_scroll_handle = ScrollHandle::new();

        cx.subscribe(&volume_slider, |this: &mut Self, _entity, event: &SliderEvent, cx| {
            let SliderEvent::Change(value) = event;
            let vol = value.end() / 100.0;
            this.set_volume(vol as f64, cx);
        })
        .detach();

        let mut app = Self {
            login_server,
            login_username,
            login_password,
            playlist_name,
            search_input,
            lyrics_server_input,
            seek_slider,
            volume_slider,
            songs_scroll_handle,
            recent_songs_scroll_handle,
            recent_albums_scroll_handle,
            focus_handle: cx.focus_handle(),
            state: DesktopAppState::new(app_data_dir),
            featured_background_source_url: None,
            featured_background_image: None,
            featured_background_loading: false,
            runtime,
            media_controls,
        };
        app.restore_settings();
        let initial_lyrics_server = app.state.lyrics_server_url.clone();
        app.lyrics_server_input
            .update(cx, |input, cx| input.set_value(&initial_lyrics_server, window, cx));
        let initial_volume = (app.state.player.volume * 100.0).round() as f32;
        app.volume_slider
            .update(cx, |slider, cx| slider.set_value(initial_volume, window, cx));
        app.bootstrap_session(cx);
        app.start_pollers(cx);
        app
    }

    // ---------------------------------------------------------------
    // Business logic (preserved from original, adapted for InputState)
    // ---------------------------------------------------------------

    fn bootstrap_session(&mut self, cx: &mut Context<Self>) {
        let app_data_dir = self.state.app_data_dir.to_string();
        let runtime = Arc::clone(&self.runtime);
        self.state.status = "Checking saved session...".to_string();
        cx.notify();

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime.spawn_blocking(move || aurelia_core::load_credentials(app_data_dir)).await;

            match result {
                Ok(Ok(Some(credentials))) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.session = Some(SessionData { credentials: credentials.clone() });
                        this.state.status = format!("Restored {}", credentials.server_url);
                        this.load_initial_data(cx);
                        cx.notify();
                    });
                }
                Ok(Ok(None)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = "Sign in to start using the desktop app".to_string();
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Failed to restore session: {error}");
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Failed to restore session task: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn sync_featured_background(&mut self, album: Option<&Album>, cx: &mut Context<Self>) {
        let Some(album) = album else {
            self.featured_background_source_url = None;
            self.featured_background_image = None;
            self.featured_background_loading = false;
            return;
        };

        let Some(art_url) = self.album_image_url(album, 1200.0) else {
            self.featured_background_source_url = None;
            self.featured_background_image = None;
            self.featured_background_loading = false;
            return;
        };

        if self.featured_background_source_url.as_deref() == Some(art_url.as_str()) {
            return;
        }

        self.featured_background_source_url = Some(art_url.clone());
        self.featured_background_image = None;
        self.featured_background_loading = true;

        let runtime = Arc::clone(&self.runtime);
        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
                let result = load_blurred_featured_background(runtime, art_url.clone()).await;

                let _ = view.update(&mut async_cx, |this, cx| {
                    if this.featured_background_source_url.as_deref() != Some(art_url.as_str()) {
                        return;
                    }

                    this.featured_background_loading = false;

                    match result {
                        Ok(image) => {
                            this.featured_background_image = Some(image);
                        }
                        Err(error) => {
                            tracing::warn!("failed to load blurred featured background: {error}");
                            this.featured_background_image = None;
                        }
                    }

                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn restore_settings(&mut self) {
        let app_data_dir = self.state.app_data_dir.to_string();
        let runtime = Arc::clone(&self.runtime);
        if let Ok(volume) = runtime.block_on(async {
            runtime
                .spawn(async { aurelia_core::audio_get_volume_player().await })
                .await
        }) && let Ok(volume) = volume {
            self.state.player.volume = volume.clamp(0.0, 1.0);
        }

        if let Ok(Some(value)) =
            aurelia_core::load_setting(self.state.app_data_dir.to_string(), "volume".to_string())
            && let Ok(volume) = value.parse::<f64>()
        {
            self.state.player.volume = volume.clamp(0.0, 1.0);
        }

        if let Ok(Some(value)) = aurelia_core::load_setting(app_data_dir.clone(), "minimizeToTray".to_string()) {
            self.state.minimize_to_tray = value == "true";
        }
        if let Ok(Some(value)) = aurelia_core::load_setting(app_data_dir.clone(), "closeToTray".to_string()) {
            self.state.close_to_tray = value == "true";
        }
        if let Ok(Some(value)) = aurelia_core::load_setting(app_data_dir, "lyricsServerUrl".to_string()) {
            self.state.lyrics_server_url = value;
        }
    }

    fn persist_setting(&self, key: &str, value: String) {
        let _ = aurelia_core::save_setting(self.state.app_data_dir.to_string(), key.to_string(), value);
    }

    fn credentials(&self) -> Option<&Credentials> {
        self.state.session.as_ref().map(|session| &session.credentials)
    }

    fn image_url_for_item(&self, item_id: &str, size_px: f32) -> Option<String> {
        let credentials = self.credentials()?;
        let size = image_request_size(size_px);
        aurelia_core::build_image_url(
            credentials.server_url.clone(),
            credentials.token.clone(),
            item_id.to_string(),
            "Primary".to_string(),
            Some(size),
            Some(90),
        )
        .ok()
        .flatten()
    }

    fn album_image_url(&self, album: &Album, size_px: f32) -> Option<String> {
        album
            .id
            .as_ref()
            .and_then(|id| self.image_url_for_item(id, size_px))
            .or_else(|| album.album_art_url.clone())
    }

    fn artist_image_url(&self, artist: &Artist, size_px: f32) -> Option<String> {
        self.image_url_for_item(&artist.id, size_px)
            .or_else(|| artist.image_url.clone())
    }

    fn song_image_url(&self, song: &Song, size_px: f32) -> Option<String> {
        song.album_id
            .as_ref()
            .and_then(|id| self.image_url_for_item(id, size_px))
            .or_else(|| song.album_art_url.clone())
    }

    fn load_initial_data(&mut self, cx: &mut Context<Self>) {
        self.load_cached_library();
        self.refresh_playlists(cx);
        self.refresh_recent(cx);
        self.refresh_favorites(cx);
        self.sync_library(cx);
    }

    fn load_cached_library(&mut self) {
        let app_data_dir = self.state.app_data_dir.to_string();
        match aurelia_core::load_cached_songs(app_data_dir) {
            Ok(songs) => {
                self.state.library.songs = songs.clone();
                self.state.library.albums = derive_albums(&songs, self.credentials());
                self.state.library.artists = derive_artists(&songs, self.credentials());
                if self.state.selected_song_id.is_none() {
                    self.state.selected_song_id = songs.first().map(|song| song.id.clone());
                }
                self.state.status = format!("Loaded {} cached songs", self.state.library.songs.len());
            }
            Err(error) => {
                self.state.status = format!("Failed to load cached library: {error}");
            }
        }
    }

    fn login(&mut self, cx: &mut Context<Self>) {
        let server = self.login_server.read(cx).value().to_string();
        let username = self.login_username.read(cx).value().to_string();
        let password = self.login_password.read(cx).value().to_string();

        if server.trim().is_empty() || username.trim().is_empty() || password.is_empty() {
            self.state.status = "Server, username, and password are required".to_string();
            cx.notify();
            return;
        }

        let normalized_server = normalize_server_url(&server);
        let app_data_dir = self.state.app_data_dir.to_string();
        let runtime = Arc::clone(&self.runtime);

        self.state.status = format!("Signing in to {normalized_server}...");
        cx.notify();

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let login_result = runtime
                .spawn(async move {
                    let provider = aurelia_core::detect_provider(normalized_server.clone()).await.unwrap_or(BackendProvider::Jellyfin);
                    let request = AuthRequest {
                        provider,
                        server_url: normalized_server.clone(),
                        username: username.clone(),
                        password: password.clone(),
                        device_id: format!("aurelia-gpui-{}", std::process::id()),
                    };

                    let response = aurelia_core::authenticate(request).await?;
                    let credentials = Credentials {
                        provider,
                        server_url: normalized_server.clone(),
                        username: username.clone(),
                        token: response.token,
                        user_id: response.user_id,
                    };
                    aurelia_core::save_credentials(app_data_dir.clone(), credentials.clone())?;
                    Ok::<Credentials, anyhow::Error>(credentials)
                })
                .await;

            match login_result {
                Ok(Ok(credentials)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.session = Some(SessionData { credentials: credentials.clone() });
                        this.state.status = format!("Signed in to {}", credentials.server_url);
                        // Mark pending clear -- will be applied in next render when we have window
                        this.state.pending_clear_password = true;
                        this.load_initial_data(cx);
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Login failed: {error}");
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Login task failed: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn logout(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let _ = aurelia_core::clear_credentials(self.state.app_data_dir.to_string());
        let _ = self.runtime.block_on(async { aurelia_core::audio_stop_player().await });
        let _ = self.media_controls.clear_now_playing();

        self.state = DesktopAppState::new(Arc::clone(&self.state.app_data_dir));
        self.restore_settings();
        self.state.status = "Signed out".to_string();

        // Clear all input fields
        self.login_server.update(cx, |s, cx| s.set_value("", window, cx));
        self.login_username.update(cx, |s, cx| s.set_value("", window, cx));
        self.login_password.update(cx, |s, cx| s.set_value("", window, cx));
        self.search_input.update(cx, |s, cx| s.set_value("", window, cx));

        cx.notify();
    }

    fn sync_library(&mut self, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let app_data_dir = self.state.app_data_dir.to_string();
        let runtime = Arc::clone(&self.runtime);

        self.state.sync_status = "Syncing library...".to_string();
        self.state.status = format!("Syncing {}", credentials.server_url);
        cx.notify();

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let sync_result = runtime
                .spawn(async move {
                    aurelia_core::sync_library_smart(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        credentials.user_id.clone(),
                        app_data_dir.clone(),
                    )
                    .await
                })
                .await;

            match sync_result {
                Ok(Ok(report)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.load_cached_library();
                        this.state.sync_status = format!(
                            "Sync complete: {} songs, {} artists, {} albums",
                            report.songs_updated, report.artists_updated, report.albums_updated
                        );
                        this.state.status = this.state.sync_status.clone();
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.sync_status = format!("Sync failed: {error}");
                        this.state.status = this.state.sync_status.clone();
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.sync_status = format!("Sync task failed: {error}");
                        this.state.status = this.state.sync_status.clone();
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn refresh_playlists(&mut self, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let runtime = Arc::clone(&self.runtime);

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    aurelia_core::get_playlists(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        credentials.user_id.clone(),
                    )
                    .await
                })
                .await;

            if let Ok(Ok(playlists)) = result {
                let _ = view.update(&mut async_cx, |this, cx| {
                    this.state.library.playlists = playlists;
                    cx.notify();
                });
            }
        }})
        .detach();
    }

    fn refresh_recent(&mut self, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let runtime = Arc::clone(&self.runtime);
        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    aurelia_core::get_recently_played(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        credentials.user_id.clone(),
                    )
                    .await
                })
                .await;

            if let Ok(Ok(recent)) = result {
                let _ = view.update(&mut async_cx, |this, cx| {
                    this.state.library.recent = recent;
                    cx.notify();
                });
            }
        }})
        .detach();
    }

    fn refresh_favorites(&mut self, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let runtime = Arc::clone(&self.runtime);
        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    aurelia_core::get_favorite_ids(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        credentials.user_id.clone(),
                    )
                    .await
                })
                .await;

            if let Ok(Ok(favorite_ids)) = result {
                let _ = view.update(&mut async_cx, |this, cx| {
                    this.state.favorite_ids = favorite_ids;
                    cx.notify();
                });
            }
        }})
        .detach();
    }

    fn play_song_by_id(&mut self, song_id: String, cx: &mut Context<Self>) {
        let songs = self.state.filtered_songs();
        if let Some(index) = songs.iter().position(|song| song.id == song_id) {
            self.play_queue(songs, index, cx);
        }
    }

    fn show_album(&mut self, album_id: String, cx: &mut Context<Self>) {
        self.state.selected_tab = ViewTab::Albums;
        self.state.selected_album_id = Some(album_id.clone());
        if let Some(song) = self
            .state
            .library
            .songs
            .iter()
            .find(|song| song.album_id.as_deref() == Some(album_id.as_str()))
        {
            self.state.selected_song_id = Some(song.id.clone());
        }
        cx.notify();
    }

    fn show_artist(&mut self, artist_id: String, cx: &mut Context<Self>) {
        self.state.selected_tab = ViewTab::Artists;
        self.state.selected_artist_id = Some(artist_id.clone());
        if let Some(song) = self
            .state
            .library
            .songs
            .iter()
            .find(|song| {
                song.artist_ids
                    .as_ref()
                    .is_some_and(|ids| ids.iter().any(|id| id == &artist_id))
            })
        {
            self.state.selected_song_id = Some(song.id.clone());
        }
        cx.notify();
    }

    fn play_album_by_id(&mut self, album_id: String, cx: &mut Context<Self>) {
        let songs = self.state.songs_for_album(&album_id);
        if !songs.is_empty() {
            self.play_queue(songs, 0, cx);
            self.state.selected_album_id = Some(album_id);
        }
    }

    fn shuffle_album_by_id(&mut self, album_id: String, cx: &mut Context<Self>) {
        let mut songs = self.state.songs_for_album(&album_id);
        if songs.is_empty() {
            return;
        }

        let mut rng = rand::thread_rng();
        songs.shuffle(&mut rng);
        self.play_queue(songs, 0, cx);
        self.state.selected_album_id = Some(album_id);
    }

    fn play_playlist(&mut self, playlist_id: String, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let runtime = Arc::clone(&self.runtime);
        self.state.playback_status = "Loading playlist...".to_string();
        cx.notify();

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    aurelia_core::get_playlist_items(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        playlist_id.clone(),
                    )
                    .await
                })
                .await;

            match result {
                Ok(Ok(items)) if !items.is_empty() => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.play_queue(items, 0, cx);
                        cx.notify();
                    });
                }
                Ok(Ok(_)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.playback_status = "Playlist is empty".to_string();
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.playback_status = format!("Failed to load playlist: {error}");
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.playback_status = format!("Playlist task failed: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn play_queue(&mut self, songs: Vec<Song>, index: usize, cx: &mut Context<Self>) {
        if songs.is_empty() || index >= songs.len() {
            return;
        }

        self.state.player.original_queue = songs.clone();
        self.state.player.queue = songs.clone();
        self.state.player.is_shuffled = false;
        self.state.player.current_index = Some(index);
        self.state.selected_song_id = Some(songs[index].id.clone());
        self.state.player.duration_secs = songs[index].duration.unwrap_or_default();
        self.state.player.position_secs = 0.0;
        self.state.playback_status = format!("Loading {}...", songs[index].name);
        cx.notify();

        self.start_current_song(cx);
    }

    fn start_current_song(&mut self, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let Some(song) = self.state.player.current_song().cloned() else {
            return;
        };
        let song_for_ui = song.clone();
        let playback_volume = self.state.player.volume;
        let runtime = Arc::clone(&self.runtime);
        let media_controls = Arc::clone(&self.media_controls);

        self.state.playback_status = format!("Starting {}", song.name);
        cx.notify();

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let play_result = runtime
                .spawn(async move {
                    aurelia_core::audio_init_player().await?;
                    let _ = aurelia_core::audio_set_volume_player(playback_volume).await;
                    let stream_url = aurelia_core::build_stream_url(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        song.id.clone(),
                        song.container.clone(),
                    );
                    if stream_url.is_empty() {
                        anyhow::bail!("stream URL was empty");
                    }

                    aurelia_core::audio_play_url(stream_url, credentials.token.clone(), None).await?;
                    aurelia_core::report_playback_start_event(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        credentials.user_id.clone(),
                        song.id.clone(),
                        Some(0),
                    )
                    .await
                    .ok();

                    media_controls
                        .update_now_playing(aurelia_core::models::NowPlayingPayload {
                            title: song.name.clone(),
                            artist: song.artists.as_ref().and_then(|artists| artists.first().cloned()),
                            album: song.album.clone(),
                            duration: song.duration,
                            cover_url: song.album_id.as_ref().and_then(|album_id| {
                                aurelia_core::build_image_url(
                                    credentials.server_url.clone(),
                                    credentials.token.clone(),
                                    album_id.clone(),
                                    "Primary".to_string(),
                                    Some(400),
                                    Some(90),
                                )
                                .ok()
                                .flatten()
                            }),
                        })
                        .ok();
                    media_controls.set_playback_status(true, Some(0.0)).ok();

                    Ok::<(), anyhow::Error>(())
                })
                .await;

            match play_result {
                Ok(Ok(())) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.player.is_playing = true;
                        this.state.playback_status = format!("Playing {}", song_for_ui.name);
                        this.fetch_lyrics_for(song_for_ui.clone(), cx);
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.player.is_playing = false;
                        this.state.playback_status = format!("Playback failed: {error}");
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.player.is_playing = false;
                        this.state.playback_status = format!("Playback task failed: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn fetch_lyrics_for(&mut self, song: Song, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let runtime = Arc::clone(&self.runtime);
        let lyrics_server_url = if self.state.lyrics_server_url.trim().is_empty() {
            None
        } else {
            Some(self.state.lyrics_server_url.clone())
        };

        self.state.lyrics_status = format!("Loading lyrics for {}", song.name);
        self.state.lyrics = None;
        cx.notify();

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    aurelia_core::get_parsed_lyrics(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        song.id.clone(),
                        song.artists
                            .as_ref()
                            .and_then(|artists| artists.first().cloned())
                            .unwrap_or_default(),
                        song.name.clone(),
                        song.path.clone(),
                        lyrics_server_url,
                    )
                    .await
                })
                .await;

            match result {
                Ok(lyrics) if !lyrics.is_empty() => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.lyrics = Some(lyrics);
                        this.state.lyrics_status = "Lyrics loaded".to_string();
                        cx.notify();
                    });
                }
                Ok(_) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.lyrics_status = "No lyrics available".to_string();
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.lyrics_status = format!("Lyrics failed: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn toggle_play_pause(&mut self, cx: &mut Context<Self>) {
        self.set_playback_state(!self.state.player.is_playing, cx);
    }

    fn set_playback_state(&mut self, should_play: bool, cx: &mut Context<Self>) {
        let runtime = Arc::clone(&self.runtime);
        let media_controls = Arc::clone(&self.media_controls);
        self.state.playback_status = if should_play { "Resuming..." } else { "Pausing..." }.to_string();
        cx.notify();

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    if should_play {
                        aurelia_core::audio_resume_player().await?;
                        media_controls.set_playback_status(true, None).ok();
                    } else {
                        aurelia_core::audio_pause_player().await?;
                        media_controls.set_playback_status(false, None).ok();
                    }
                    Ok::<bool, anyhow::Error>(should_play)
                })
                .await;

            match result {
                Ok(Ok(next_state)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.player.is_playing = next_state;
                        this.state.playback_status = if next_state { "Playing" } else { "Paused" }.to_string();
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.playback_status = format!("Playback change failed: {error}");
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.playback_status = format!("Playback change task failed: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn next_track_action(&mut self, cx: &mut Context<Self>) {
        if let Some(index) = self.state.player.current_index {
            if index + 1 < self.state.player.queue.len() {
                self.state.player.current_index = Some(index + 1);
                self.state.selected_song_id = self.state.player.queue.get(index + 1).map(|song| song.id.clone());
                self.start_current_song(cx);
            } else if self.state.player.repeat_mode == RepeatMode::All && !self.state.player.queue.is_empty() {
                self.state.player.current_index = Some(0);
                self.state.selected_song_id = self.state.player.queue.first().map(|song| song.id.clone());
                self.start_current_song(cx);
            }
        }
    }

    fn previous_track_action(&mut self, cx: &mut Context<Self>) {
        if let Some(index) = self.state.player.current_index
            && index > 0
        {
            self.state.player.current_index = Some(index - 1);
            self.state.selected_song_id = self.state.player.queue.get(index - 1).map(|song| song.id.clone());
            self.start_current_song(cx);
        }
    }

    fn stop_playback_action(&mut self, cx: &mut Context<Self>) {
        let runtime = Arc::clone(&self.runtime);
        let media_controls = Arc::clone(&self.media_controls);
        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let _ = runtime.spawn(async move { aurelia_core::audio_stop_player().await }).await;
            media_controls.clear_now_playing().ok();
            let _ = view.update(&mut async_cx, |this, cx| {
                this.state.player.is_playing = false;
                this.state.player.position_secs = 0.0;
                this.state.playback_status = "Stopped".to_string();
                cx.notify();
            });
        }})
        .detach();
    }

    fn toggle_queue_panel(&mut self, cx: &mut Context<Self>) {
        self.state.right_panel = if self.state.right_panel == RightPanel::Queue {
            RightPanel::None
        } else {
            RightPanel::Queue
        };
        cx.notify();
    }

    fn toggle_lyrics_panel(&mut self, cx: &mut Context<Self>) {
        self.state.right_panel = if self.state.right_panel == RightPanel::Lyrics {
            RightPanel::None
        } else {
            RightPanel::Lyrics
        };
        cx.notify();
    }

    fn close_right_panel(&mut self, cx: &mut Context<Self>) {
        self.state.right_panel = RightPanel::None;
        cx.notify();
    }

    fn toggle_favorite_current(&mut self, cx: &mut Context<Self>) {
        let Some(current_song_id) = self.state.player.current_song().map(|song| song.id.clone()) else {
            return;
        };

        let previous_selection = self.state.selected_song_id.clone();
        self.state.selected_song_id = Some(current_song_id);
        self.toggle_favorite_selected(cx);
        self.state.selected_song_id = previous_selection;
    }

    fn toggle_shuffle(&mut self, cx: &mut Context<Self>) {
        let current_song_id = self.state.player.current_song().map(|song| song.id.clone());
        if self.state.player.original_queue.is_empty() {
            self.state.player.original_queue = self.state.player.queue.clone();
        }
        if self.state.player.original_queue.is_empty() {
            return;
        }

        if self.state.player.is_shuffled {
            self.state.player.queue = self.state.player.original_queue.clone();
            self.state.player.is_shuffled = false;
        } else {
            let mut shuffled = self.state.player.original_queue.clone();
            let mut rng = rand::thread_rng();
            shuffled.shuffle(&mut rng);

            if let Some(current_song_id) = current_song_id.clone()
                && let Some(current_index) = shuffled.iter().position(|song| song.id == current_song_id)
            {
                let current_song = shuffled.remove(current_index);
                shuffled.insert(0, current_song);
            }

            self.state.player.queue = shuffled;
            self.state.player.is_shuffled = true;
        }

        self.state.player.current_index = current_song_id
            .as_ref()
            .and_then(|id| self.state.player.queue.iter().position(|song| &song.id == id));
        self.state.playback_status = if self.state.player.is_shuffled {
            "Shuffle on".to_string()
        } else {
            "Shuffle off".to_string()
        };
        cx.notify();
    }

    fn cycle_repeat_mode(&mut self, cx: &mut Context<Self>) {
        self.state.player.repeat_mode = match self.state.player.repeat_mode {
            RepeatMode::None => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::None,
        };
        self.state.playback_status = match self.state.player.repeat_mode {
            RepeatMode::None => "Repeat off".to_string(),
            RepeatMode::All => "Repeat all".to_string(),
            RepeatMode::One => "Repeat one".to_string(),
        };
        cx.notify();
    }

    fn seek_to(&mut self, fraction: f64, cx: &mut Context<Self>) {
        let Some(song) = self.state.player.current_song() else {
            return;
        };
        let duration = song.duration.unwrap_or_default().max(0.0);
        let position = (duration * fraction.clamp(0.0, 1.0)).max(0.0);
        let runtime = Arc::clone(&self.runtime);
        self.state.player.position_secs = position;
        cx.notify();

        drop(runtime.spawn(async move {
            let _ = aurelia_core::audio_seek_player(position).await;
        }));
    }

    fn set_volume(&mut self, volume: f64, cx: &mut Context<Self>) {
        self.state.player.volume = volume.clamp(0.0, 1.0);
        let runtime = Arc::clone(&self.runtime);
        let vol = self.state.player.volume;
        self.persist_setting("volume", vol.to_string());
        cx.notify();
        drop(runtime.spawn(async move {
            let _ = aurelia_core::audio_set_volume_player(vol).await;
        }));
    }

    fn scroll_carousel_by(&mut self, handle: &ScrollHandle, delta_px: f32, cx: &mut Context<Self>) {
        let offset = handle.offset();
        let max_offset = handle.max_offset();
        let next_x = (offset.x.to_f64() - delta_px as f64).clamp(-max_offset.width.to_f64(), 0.0);
        handle.set_offset(Point {
            x: px(next_x as f32),
            y: offset.y,
        });
        cx.notify();
    }

    fn create_playlist(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let name = self.playlist_name.read(cx).value().to_string();
        if name.trim().is_empty() {
            self.state.status = "Playlist name cannot be empty".to_string();
            cx.notify();
            return;
        }

        let selected_ids = self
            .state
            .selected_song()
            .map(|song| vec![song.id.clone()])
            .filter(|ids| !ids.is_empty());
        let runtime = Arc::clone(&self.runtime);
        let playlist_name = name.clone();

        // Clear the input immediately (we have window here)
        self.playlist_name.update(cx, |s, cx| s.set_value("", window, cx));

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    aurelia_core::create_playlist(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        PlaylistCreateData {
                            name: name.clone(),
                            ids: selected_ids,
                            user_id: credentials.user_id.clone(),
                            is_public: Some(false),
                        },
                    )
                    .await
                })
                .await;

            match result {
                Ok(Ok(_playlist)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Created playlist {playlist_name}");
                        this.refresh_playlists(cx);
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Failed to create playlist: {error}");
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Create playlist task failed: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn rename_selected_playlist(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let Some(playlist_id) = self.state.selected_playlist_id.clone() else {
            self.state.status = "Select a playlist first".to_string();
            cx.notify();
            return;
        };
        let name = self.playlist_name.read(cx).value().to_string();
        if name.trim().is_empty() {
            self.state.status = "Enter a playlist name first".to_string();
            cx.notify();
            return;
        }
        let runtime = Arc::clone(&self.runtime);
        let playlist_name = name.clone();

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    aurelia_core::update_playlist(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        playlist_id.clone(),
                        PlaylistUpdateData {
                            name: Some(name.clone()),
                            ids: None,
                            user_id: Some(credentials.user_id.clone()),
                            is_public: Some(false),
                            songs: None,
                            is_favorite: None,
                        },
                    )
                    .await
                })
                .await;

            match result {
                Ok(Ok(_)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Renamed playlist to {playlist_name}");
                        this.refresh_playlists(cx);
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Failed to rename playlist: {error}");
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Rename playlist task failed: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn delete_selected_playlist(&mut self, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let Some(playlist_id) = self.state.selected_playlist_id.clone() else {
            self.state.status = "Select a playlist first".to_string();
            cx.notify();
            return;
        };
        let runtime = Arc::clone(&self.runtime);

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    aurelia_core::delete_playlist(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        playlist_id.clone(),
                    )
                    .await
                })
                .await;

            match result {
                Ok(Ok(())) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.selected_playlist_id = None;
                        this.state.status = "Playlist deleted".to_string();
                        this.refresh_playlists(cx);
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Failed to delete playlist: {error}");
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Delete playlist task failed: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn toggle_favorite_selected(&mut self, cx: &mut Context<Self>) {
        let Some(credentials) = self.credentials().cloned() else {
            return;
        };
        let Some(song) = self.state.selected_song().cloned() else {
            return;
        };
        let should_be_favorite = !self.state.favorite_ids.iter().any(|id| id == &song.id);
        let runtime = Arc::clone(&self.runtime);

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            let result = runtime
                .spawn(async move {
                    aurelia_core::toggle_favorite(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        credentials.user_id.clone(),
                        song.id.clone(),
                        should_be_favorite,
                    )
                    .await
                })
                .await;

            match result {
                Ok(Ok(_)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = if should_be_favorite {
                            format!("Favorited {}", song.name)
                        } else {
                            format!("Unfavorited {}", song.name)
                        };
                        this.refresh_favorites(cx);
                        cx.notify();
                    });
                }
                Ok(Err(error)) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Favorite update failed: {error}");
                        cx.notify();
                    });
                }
                Err(error) => {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        this.state.status = format!("Favorite task failed: {error}");
                        cx.notify();
                    });
                }
            }
        }})
        .detach();
    }

    fn select_tab(&mut self, tab: ViewTab, cx: &mut Context<Self>) {
        self.state.selected_tab = tab;
        cx.notify();
    }

    fn set_seek_slider_value(&mut self, value: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.seek_slider.update(cx, |slider, cx| slider.set_value(value, window, cx));
    }

    fn use_search_text(&mut self, cx: &mut Context<Self>) {
        self.state.search_query = self.search_input.read(cx).value().to_string();
        cx.notify();
    }

    fn save_lyrics_server_url(&mut self, cx: &mut Context<Self>) {
        let value = self.lyrics_server_input.read(cx).value().trim().to_string();
        self.state.lyrics_server_url = value.clone();
        self.persist_setting("lyricsServerUrl", value);
        self.state.status = if self.state.lyrics_server_url.is_empty() {
            "Lyrics server cleared".to_string()
        } else {
            "Lyrics server URL saved".to_string()
        };
        cx.notify();
    }

    fn poll_audio_and_media(&mut self, cx: &mut Context<Self>) {
        let runtime = Arc::clone(&self.runtime);
        let media_controls = Arc::clone(&self.media_controls);

        cx.spawn(move |view: WeakEntity<DesktopApp>, async_cx: &mut AsyncApp| {
            let mut async_cx = async_cx.clone();
            async move {
            loop {
                async_cx.background_executor().timer(Duration::from_millis(400)).await;

                let position_result = runtime
                    .spawn(async {
                        let is_playing = aurelia_core::audio_is_playing_player().await.unwrap_or(false);
                        let is_finished = aurelia_core::audio_is_finished_player().await.unwrap_or(false);
                        let position = aurelia_core::audio_get_position_secs().await.unwrap_or(0.0);
                        (is_playing, is_finished, position)
                    })
                    .await;

                if let Ok((is_playing, is_finished, position)) = position_result {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        let previous_is_playing = this.state.player.is_playing;
                        let previous_display_second = this.state.player.position_secs.floor() as i64;
                        let next_display_second = position.floor() as i64;
                        let mut should_notify = false;

                        if previous_is_playing != is_playing {
                            this.state.player.is_playing = is_playing;
                            should_notify = true;
                        }

                        if previous_display_second != next_display_second {
                            this.state.player.position_secs = position;
                            should_notify = true;
                        } else {
                            this.state.player.position_secs = position;
                        }

                        if is_finished {
                            if this.state.player.repeat_mode == RepeatMode::One {
                                this.start_current_song(cx);
                            } else if let Some(index) = this.state.player.current_index {
                                if index + 1 < this.state.player.queue.len() {
                                    this.state.player.current_index = Some(index + 1);
                                    this.state.selected_song_id = this
                                        .state
                                        .player
                                        .queue
                                        .get(index + 1)
                                        .map(|song| song.id.clone());
                                    this.start_current_song(cx);
                                } else if this.state.player.repeat_mode == RepeatMode::All
                                    && !this.state.player.queue.is_empty()
                                {
                                    this.state.player.current_index = Some(0);
                                    this.state.selected_song_id = this.state.player.queue.first().map(|song| song.id.clone());
                                    this.start_current_song(cx);
                                } else {
                                    this.state.player.is_playing = false;
                                    this.state.player.position_secs = this.state.player.duration_secs;
                                    should_notify = true;
                                }
                            }
                        }

                        if should_notify {
                            cx.notify();
                        }
                    });
                }

                while let Some(event) = media_controls.pop_event() {
                    let _ = view.update(&mut async_cx, |this, cx| {
                        match event {
                            aurelia_core::media_controls::MediaEvent::Play
                            | aurelia_core::media_controls::MediaEvent::Toggle => {
                                if !this.state.player.is_playing {
                                    this.set_playback_state(true, cx);
                                }
                            }
                            aurelia_core::media_controls::MediaEvent::Pause => {
                                if this.state.player.is_playing {
                                    this.set_playback_state(false, cx);
                                }
                            }
                            aurelia_core::media_controls::MediaEvent::Next => {
                                this.next_track_action(cx);
                            }
                            aurelia_core::media_controls::MediaEvent::Previous => {
                                this.previous_track_action(cx);
                            }
                            aurelia_core::media_controls::MediaEvent::SeekDelta(delta) => {
                                let duration = this.state.player.duration_secs.max(1.0);
                                let position = (this.state.player.position_secs + delta).clamp(0.0, duration);
                                this.seek_to(position / duration, cx);
                            }
                            aurelia_core::media_controls::MediaEvent::SetPosition(position) => {
                                let duration = this.state.player.duration_secs.max(1.0);
                                this.seek_to(position / duration, cx);
                            }
                            aurelia_core::media_controls::MediaEvent::Stop => {
                                this.stop_playback_action(cx);
                            }
                        }
                    });
                }
            }
        }})
        .detach();
    }

    fn start_pollers(&mut self, cx: &mut Context<Self>) {
        self.poll_audio_and_media(cx);
    }

    // ---------------------------------------------------------------
    // Render methods (fully migrated to gpui-component)
    // ---------------------------------------------------------------

    /// Apply any pending input clears that couldn't happen in async closures.
    fn flush_pending_clears(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.state.pending_clear_password {
            self.state.pending_clear_password = false;
            self.login_password.update(cx, |s, cx| s.set_value("", window, cx));
        }
        if self.state.pending_clear_playlist_name {
            self.state.pending_clear_playlist_name = false;
            self.playlist_name.update(cx, |s, cx| s.set_value("", window, cx));
        }
    }

    fn render_login(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let login_server = self.login_server.clone();
        let login_username = self.login_username.clone();
        let login_password = self.login_password.clone();

        div()
            .size_full()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::FOREGROUND))
            .flex()
            .items_center()
            .justify_center()
            .child(
                v_flex()
                    .max_w(px(420.0))
                    .w(relative(1.0))
                    .gap(px(24.0))
                    // Branding
                    .child(
                        v_flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_size(px(32.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(theme::FOREGROUND))
                                    .child("Aurelia"),
                            )
                            .child(
                                div()
                                    .text_size(px(14.0))
                                    .text_color(rgb(theme::MUTED_FG))
                                    .child("Sign in to your music server"),
                            ),
                    )
                    // Card
                    .child(
                        v_flex()
                            .gap(px(16.0))
                            .p(px(24.0))
                            .rounded(px(12.0))
                            .bg(rgb(theme::CARD))
                            .border_1()
                            .border_color(rgb(theme::BORDER))
                            .child(
                                v_flex()
                                    .gap(px(6.0))
                                    .child(field_label("Server URL"))
                                    .child(Input::new(&login_server)),
                            )
                            .child(
                                v_flex()
                                    .gap(px(6.0))
                                    .child(field_label("Username"))
                                    .child(Input::new(&login_username)),
                            )
                            .child(
                                v_flex()
                                    .gap(px(6.0))
                                    .child(field_label("Password"))
                                    .child(Input::new(&login_password).mask_toggle()),
                            )
                            .child(
                                Button::new("login-btn")
                                    .label("Sign in")
                                    .primary()
                                    .on_click(cx.listener(|this, _, _window, cx| this.login(cx))),
                            ),
                    )
                    .when(!self.state.status.is_empty(), |this| {
                        this.child(
                            div()
                                .text_size(px(12.0))
                                .text_color(rgb(theme::MUTED_FG))
                                .text_center()
                                .child(self.state.status.clone()),
                        )
                    }),
            )
    }

    fn render_shell(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let player = &self.state.player;
        let progress = if player.duration_secs > 0.0 {
            player.position_secs / player.duration_secs
        } else {
            0.0
        };
        let right_panel_open = self.state.right_panel != RightPanel::None;
        let has_player = self.state.player.current_song().is_some();

        v_flex()
            .relative()
            .size_full()
            .bg(rgb(theme::BACKGROUND))
            .text_color(rgb(theme::FOREGROUND))
            // Top: header bar
            .child(self.render_header(window, cx))
            // Middle: sidebar + content + inspector
            .child(
                h_flex()
                    .flex_1()
                    .overflow_hidden()
                    .items_start()
                    .child(self.render_sidebar(cx))
                    .child(self.render_content(right_panel_open, cx))
                    .when(right_panel_open, |this| this.child(self.render_right_panel(cx))),
            )
            .when(has_player, |this| {
                this.child(self.render_player_bar(progress, right_panel_open, cx))
            })
    }

    fn render_header(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        TitleBar::new()
            .bg(rgb(theme::BACKGROUND))
            .border_color(rgb(theme::BORDER))
            .child(
                h_flex()
                    .items_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child("Aurelia"),
                    )
                    .child(
                        Button::new("sync-btn")
                            .icon(Icon::new(AppIcon::RefreshCw))
                            .ghost()
                            .small()
                            .tooltip("Sync library")
                            .on_click(cx.listener(|this, _, _, cx| this.sync_library(cx))),
                    ),
            )
    }

    fn render_sidebar(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let search_input = self.search_input.clone();
        let username = self
            .credentials()
            .map(|c| c.username.clone())
            .unwrap_or_default();
        let server = self
            .credentials()
            .map(|c| {
                c.server_url
                    .trim_start_matches("https://")
                    .trim_start_matches("http://")
                    .to_string()
            })
            .unwrap_or_default();

        v_flex()
            .w(px(200.0))
            .flex_shrink_0()
            .h_full()
            .border_r_1()
            .border_color(rgb(theme::BORDER))
            .bg(rgb(theme::SIDEBAR_BG))
            .justify_between()
            .child(
                v_flex()
                    .p(px(12.0))
                    .gap(px(10.0))
                    .child(
                        div().child(
                            Input::new(&search_input)
                                .prefix(Icon::new(IconName::Search).small())
                                .cleanable(true),
                        ),
                    )
                    // Nav items
                    .children(ViewTab::ALL.into_iter().map(|tab| {
                        let selected = self.state.selected_tab == tab;
                        div()
                            .id(SharedString::from(format!("nav-{}", tab.label())))
                            .px(px(12.0))
                            .py(px(8.0))
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .text_size(px(13.0))
                            .bg(if selected {
                                rgb(theme::SIDEBAR_ITEM_ACTIVE)
                            } else {
                                rgba(0x00000000)
                            })
                            .text_color(if selected {
                                rgb(theme::FOREGROUND)
                            } else {
                                rgb(theme::MUTED_FG)
                            })
                            .hover(|style| {
                                if selected {
                                    style
                                } else {
                                    style.bg(rgb(theme::SIDEBAR_ITEM_HOVER))
                                }
                            })
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| this.select_tab(tab, cx)),
                            )
                            .child(
                                h_flex()
                                    .gap(px(10.0))
                                    .child(
                                        tab.icon()
                                            .small()
                                            .text_color(rgb(theme::MUTED_FG)),
                                    )
                                    .child(tab.label().to_string()),
                            )
                    })),
            )
            // Bottom profile section
            .child(
                v_flex()
                    .p(px(12.0))
                    .border_t_1()
                    .border_color(rgb(theme::BORDER))
                    .gap(px(8.0))
                    .child(
                        v_flex()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .text_color(rgb(theme::FOREGROUND))
                                    .truncate()
                                    .child(username),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(rgb(theme::MUTED_FG))
                                    .truncate()
                                    .child(server),
                            ),
                    )
                    .child(
                        Button::new("logout-btn")
                            .label("Sign out")
                            .icon(Icon::new(AppIcon::LogOut))
                            .ghost()
                            .small()
                            .on_click(cx.listener(|this, _, window, cx| this.logout(window, cx))),
                    ),
            )
    }

    fn render_content(&mut self, right_panel_open: bool, cx: &mut Context<Self>) -> impl IntoElement {
        match self.state.selected_tab {
            ViewTab::Home => div()
                .id("content-scroll")
                .flex_1()
                .min_w(px(0.0))
                .h_full()
                .overflow_y_scroll()
                .overflow_x_hidden()
                .pb(if self.state.player.current_song().is_some() { px(112.0) } else { px(24.0) })
                .child(self.render_home(cx))
                .into_any_element(),
            _ => div()
                .id("content-scroll")
                .flex_1()
                .min_w(px(0.0))
                .h_full()
                .overflow_y_scroll()
                .overflow_x_hidden()
                .p(px(24.0))
                .pr(if right_panel_open { px(20.0) } else { px(24.0) })
                .pb(if self.state.player.current_song().is_some() { px(112.0) } else { px(24.0) })
                .child(match self.state.selected_tab {
                    ViewTab::Songs => self.render_songs(cx).into_any_element(),
                    ViewTab::Albums => self.render_albums(cx).into_any_element(),
                    ViewTab::Artists => self.render_artists(cx).into_any_element(),
                    ViewTab::Playlists => self.render_playlists(cx).into_any_element(),
                    ViewTab::Search => self.render_search(cx).into_any_element(),
                    ViewTab::Settings => self.render_settings(cx).into_any_element(),
                    ViewTab::Home => unreachable!(),
                })
                .into_any_element(),
        }
    }

    fn render_home(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let featured_album = self.state.featured_album();
        self.sync_featured_background(featured_album.as_ref(), cx);
        let recently_added = self
            .state
            .recently_added_albums()
            .into_iter()
            .take(8)
            .collect::<Vec<_>>();
        let highlights = self
            .state
            .home_highlight_albums()
            .into_iter()
            .take(10)
            .collect::<Vec<_>>();
        let recent_songs = self.state.library.recent.iter().take(8).cloned().collect::<Vec<_>>();
        let recent_song_card_width = 192.0;
        let recent_album_card_width = 192.0;

        v_flex()
            .w_full()
            .gap(px(24.0))
            .when_some(featured_album.as_ref(), |this, album| {
                let play_album_id = album.id.clone();
                let shuffle_album_id = album.id.clone();
                let open_album_id_for_title = album.id.clone();
                let open_album_id_for_art = album.id.clone();
                let album_name = album.name.clone();
                let artist_name = album.artist.clone();
                let artist_id = album.artist_id.clone();
                let artist_id_for_name = artist_id.clone();
                let art_url = self.album_image_url(album, 220.0);
                let track_count = album.song_count;
                let blurred_background = self.featured_background_image.clone();

                this.child(
                    div()
                        .relative()
                        .h(px(320.0))
                        .overflow_hidden()
                        .rounded(px(0.0))
                        .bg(rgb(theme::BACKGROUND_DARK))
                        .child(
                            div()
                                .absolute()
                                .top_0()
                                .left_0()
                                .size_full()
                                .when_some(blurred_background.clone(), |this, image| {
                                    this.child(
                                        img(image)
                                            .size_full()
                                            .object_fit(ObjectFit::Cover)
                                            .opacity(0.22)
                                            .with_fallback(|| div().size_full().bg(rgb(theme::CARD)).into_any_element())
                                            .with_loading(|| div().size_full().bg(rgb(theme::CARD)).into_any_element()),
                                    )
                                })
                                .child(
                                    div()
                                        .absolute()
                                        .top_0()
                                        .left_0()
                                        .size_full()
                                        .bg(rgba(0x0000005e)),
                                )
                                .child(
                                    div()
                                        .absolute()
                                        .bottom_0()
                                        .left_0()
                                        .right_0()
                                        .h(px(144.0))
                                        .bg(linear_gradient(
                                            180.0,
                                            linear_color_stop(rgba(0x09090b00), 0.0),
                                            linear_color_stop(rgba(0x09090bff), 1.0),
                                        )),
                                ),
                        )
                        .child(
                            div()
                                .relative()
                                .w_full()
                                .h_full()
                                .px(px(24.0))
                                .py(px(20.0))
                                .child(
                                    h_flex()
                                        .justify_between()
                                        .gap(px(28.0))
                                        .items_center()
                                        .w_full()
                                        .h_full()
                                        .child(
                                            v_flex()
                                                .gap(px(10.0))
                                                .flex_1()
                                                .child(
                                                    div()
                                                        .text_size(px(34.0))
                                                        .font_weight(gpui::FontWeight::BOLD)
                                                        .text_color(rgb(theme::FOREGROUND))
                                                        .cursor_pointer()
                                                        .hover(|style| style.text_color(rgba(0xffffffd8)))
                                                        .on_mouse_up(
                                                            MouseButton::Left,
                                                            cx.listener(move |this, _, _, cx| {
                                                                if let Some(album_id) = open_album_id_for_title.clone() {
                                                                    this.show_album(album_id, cx);
                                                                }
                                                            }),
                                                        )
                                                        .child(album_name.clone()),
                                                )
                                                .child(
                                                    h_flex()
                                                        .gap(px(8.0))
                                                        .mb(px(10.0))
                                                        .items_center()
                                                        .child(
                                                            div()
                                                                .text_size(px(14.0))
                                                                .text_color(rgba(0xffffffc7))
                                                                .when_some(artist_id_for_name.clone(), |this, artist_id| {
                                                                    this.cursor_pointer()
                                                                        .hover(|style| style.text_color(rgba(0xffffffff)))
                                                                        .on_mouse_up(
                                                                            MouseButton::Left,
                                                                            cx.listener(move |this, _, _, cx| {
                                                                                this.show_artist(artist_id.clone(), cx);
                                                                            }),
                                                                        )
                                                                })
                                                                .child(artist_name),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_size(px(14.0))
                                                                .text_color(rgba(0xffffff8a))
                                                                .child(format!(
                                                                    "·  {} track{}",
                                                                    track_count,
                                                                    if track_count == 1 { "" } else { "s" }
                                                                )),
                                                        ),
                                                )
                                                .child(
                                                    h_flex()
                                                        .gap(px(10.0))
                                                        .child(
                                                            Button::new("home-play-featured")
                                                                .label("Play")
                                                                .icon(Icon::new(AppIcon::Play))
                                                                .primary()
                                                                .small()
                                                                .rounded(px(999.0))
                                                                .px(px(22.0))
                                                                .h(px(40.0))
                                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                                    if let Some(album_id) = play_album_id.clone() {
                                                                        this.play_album_by_id(album_id, cx);
                                                                    }
                                                                })),
                                                        )
                                                        .child(
                                                            Button::new("home-open-featured")
                                                                .label("Shuffle")
                                                                .icon(Icon::new(AppIcon::Shuffle))
                                                                .small()
                                                                .rounded(px(999.0))
                                                                .px(px(22.0))
                                                                .h(px(40.0))
                                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                                    if let Some(album_id) = shuffle_album_id.clone() {
                                                                        this.shuffle_album_by_id(album_id, cx);
                                                                    }
                                                                })),
                                                        )
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex_shrink_0()
                                                .cursor_pointer()
                                                .hover(|style| style.opacity(0.94))
                                                .on_mouse_up(
                                                    MouseButton::Left,
                                                    cx.listener(move |this, _, _, cx| {
                                                        if let Some(album_id) = open_album_id_for_art.clone() {
                                                            this.show_album(album_id, cx);
                                                        }
                                                    }),
                                                )
                                                .child(cover_art(
                                                    art_url.as_deref(),
                                                    220.0,
                                                    18.0,
                                                    false,
                                                    AppIcon::Music,
                                                )),
                                        ),
                                ),
                        ),
                )
            })
            .child(
                div().px(px(24.0)).child(
                    h_flex()
                        .gap(px(12.0))
                        .items_start()
                        .w_full()
                        .flex_wrap()
                        .child(stat_card("Songs", self.state.library.songs.len()))
                        .child(stat_card("Albums", self.state.library.albums.len()))
                        .child(stat_card("Artists", self.state.library.artists.len()))
                        .child(stat_card("Playlists", self.state.library.playlists.len())),
                ),
            )
            .when(!recent_songs.is_empty(), |this| {
                this.child(
                    v_flex()
                        .px(px(24.0))
                        .child(carousel_header(
                            "Recently Played",
                            "home-recent-songs-prev",
                            "home-recent-songs-next",
                            cx,
                            move |this, cx| {
                                let handle = this.recent_songs_scroll_handle.clone();
                                this.scroll_carousel_by(&handle, recent_song_card_width, cx);
                            },
                            move |this, cx| {
                                let handle = this.recent_songs_scroll_handle.clone();
                                this.scroll_carousel_by(&handle, -recent_song_card_width, cx);
                            },
                        ))
                        .child(
                            div()
                                .id("home-recent-songs-scroll")
                                .w_full()
                                .track_scroll(&self.recent_songs_scroll_handle)
                                .overflow_x_scroll()
                                .on_scroll_wheel(cx.listener(|_, _, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                }))
                                .child(
                                    h_flex().gap(px(12.0)).children(recent_songs.into_iter().map(|song| {
                                        let art_url = self.song_image_url(&song, 156.0);
                                        let song_id = song.id.clone();
                                        let album_id = song.album_id.clone();
                                        div()
                                            .w(px(180.0))
                                            .flex_shrink_0()
                                            .rounded(px(12.0))
                                            .border_1()
                                            .border_color(rgb(theme::BORDER))
                                            .bg(rgb(theme::CARD))
                                            .p(px(12.0))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(theme::SIDEBAR_ITEM_HOVER)))
                                            .on_mouse_up(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    this.state.selected_song_id = Some(song_id.clone());
                                                    if let Some(album_id) = album_id.clone() {
                                                        this.state.selected_album_id = Some(album_id);
                                                    }
                                                    cx.notify();
                                                }),
                                            )
                                            .child(cover_art(art_url.as_deref(), 156.0, 12.0, false, AppIcon::Music))
                                            .child(
                                                v_flex()
                                                    .gap(px(4.0))
                                                    .mt(px(10.0))
                                                    .min_w(px(0.0))
                                                    .child(
                                                        div()
                                                            .w_full()
                                                            .text_size(px(13.0))
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .truncate()
                                                            .child(song.name.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .w_full()
                                                            .text_size(px(11.0))
                                                            .text_color(rgb(theme::MUTED_FG))
                                                            .truncate()
                                                            .child(
                                                                song.artists
                                                                    .as_ref()
                                                                    .and_then(|artists| artists.first().cloned())
                                                                    .unwrap_or_else(|| "Unknown Artist".to_string()),
                                                            ),
                                                    ),
                                            )
                                    })),
                                ),
                        ),
                )
            })
            .when(!recently_added.is_empty(), |this| {
                this.child(
                    v_flex()
                        .px(px(24.0))
                        .child(carousel_header(
                            "Recently Added",
                            "home-recent-albums-prev",
                            "home-recent-albums-next",
                            cx,
                            move |this, cx| {
                                let handle = this.recent_albums_scroll_handle.clone();
                                this.scroll_carousel_by(&handle, recent_album_card_width, cx);
                            },
                            move |this, cx| {
                                let handle = this.recent_albums_scroll_handle.clone();
                                this.scroll_carousel_by(&handle, -recent_album_card_width, cx);
                            },
                        ))
                        .child(
                            div()
                                .id("home-recent-albums-scroll")
                                .w_full()
                                .track_scroll(&self.recent_albums_scroll_handle)
                                .overflow_x_scroll()
                                .on_scroll_wheel(cx.listener(|_, _, window, cx| {
                                    window.prevent_default();
                                    cx.stop_propagation();
                                }))
                                .child(
                                    h_flex().gap(px(12.0)).children(recently_added.into_iter().map(|album| {
                                        let art_url = self.album_image_url(&album, 156.0);
                                        let album_id = album.id.clone();
                                        let artist_id = album.artist_id.clone();
                                        div()
                                            .w(px(180.0))
                                            .flex_shrink_0()
                                            .rounded(px(12.0))
                                            .border_1()
                                            .border_color(rgb(theme::BORDER))
                                            .bg(rgb(theme::CARD))
                                            .p(px(12.0))
                                            .cursor_pointer()
                                            .hover(|style| style.bg(rgb(theme::SIDEBAR_ITEM_HOVER)))
                                            .on_mouse_up(
                                                MouseButton::Left,
                                                cx.listener(move |this, _, _, cx| {
                                                    if let Some(album_id) = album_id.clone() {
                                                        this.show_album(album_id, cx);
                                                    }
                                                }),
                                            )
                                            .child(cover_art(art_url.as_deref(), 156.0, 12.0, false, AppIcon::Disc3))
                                            .child(
                                                v_flex()
                                                    .gap(px(4.0))
                                                    .mt(px(10.0))
                                                    .min_w(px(0.0))
                                                    .child(
                                                        div()
                                                            .w_full()
                                                            .text_size(px(13.0))
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .truncate()
                                                            .child(album.name.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .w_full()
                                                            .text_size(px(11.0))
                                                            .text_color(rgb(theme::MUTED_FG))
                                                            .truncate()
                                                            .child(album.artist.clone()),
                                                    )
                                                    .when_some(artist_id, |this, artist_id| {
                                                        this.child(
                                                            Button::new(SharedString::from(format!("open-artist-{artist_id}")))
                                                                .label("Artist")
                                                                .ghost()
                                                                .xsmall()
                                                                .on_click(cx.listener(move |this, _, _, cx| {
                                                                    this.show_artist(artist_id.clone(), cx);
                                                                })),
                                                        )
                                                    }),
                                            )
                                    })),
                                ),
                        ),
                )
            })
            .when(!highlights.is_empty(), |this| {
                this.child(
                    v_flex()
                        .px(px(24.0))
                        .child(section_header("Library Highlights"))
                        .child(
                            card_container().children(highlights.into_iter().enumerate().map(|(i, album)| {
                                let art_url = self.album_image_url(&album, 52.0);
                                let album_id = album.id.clone();
                                h_flex()
                                    .justify_between()
                                    .px(px(12.0))
                                    .py(px(10.0))
                                    .when(i > 0, |this| this.border_t_1().border_color(rgb(theme::BORDER)))
                                    .child(
                                        h_flex()
                                            .gap(px(12.0))
                                            .items_center()
                                            .child(cover_art(art_url.as_deref(), 52.0, 10.0, false, AppIcon::Disc3))
                                            .child(
                                                v_flex()
                                                    .gap(px(2.0))
                                                    .min_w(px(0.0))
                                                    .child(
                                                        div()
                                                            .w_full()
                                                            .text_size(px(13.0))
                                                            .font_weight(gpui::FontWeight::MEDIUM)
                                                            .truncate()
                                                            .child(album.name.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .w_full()
                                                            .text_size(px(11.0))
                                                            .text_color(rgb(theme::MUTED_FG))
                                                            .truncate()
                                                            .child(album.artist.clone()),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        Button::new(SharedString::from(format!("home-highlight-{i}")))
                                            .label("Open")
                                            .ghost()
                                            .xsmall()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                if let Some(album_id) = album_id.clone() {
                                                    this.show_album(album_id, cx);
                                                }
                                            })),
                                    )
                            })),
                        ),
                )
            })
    }

    fn render_songs(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let songs = self.state.filtered_songs();
        let count = songs.len();
        if let Some(selected_id) = self.state.selected_song_id.as_ref()
            && let Some(index) = songs.iter().position(|song| &song.id == selected_id)
        {
            self.songs_scroll_handle
                .scroll_to_item(index, ScrollStrategy::Center);
        }
        v_flex()
            .gap(px(16.0))
            .child(page_header("Songs", Some(&format!("{count} songs"))))
            .child(
                card_container().child(
                    uniform_list(
                        "songs-list",
                        songs.len(),
                        cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
                            range
                                .map(|index| {
                                    let song = songs[index].clone();
                                    let song_id = song.id.clone();
                                    let album_id = song.album_id.clone();
                                    let selected = this
                                        .state
                                        .selected_song_id
                                        .as_ref()
                                        .is_some_and(|id| id == &song.id);
                                    let is_playing = this
                                        .state
                                        .player
                                        .current_song()
                                        .is_some_and(|current| current.id == song.id);

                                    div()
                                        .id(("song", index))
                                        .cursor_pointer()
                                        .rounded(px(6.0))
                                        .bg(if selected {
                                            rgb(theme::SIDEBAR_ITEM_ACTIVE)
                                        } else {
                                            rgba(0x00000000)
                                        })
                                        .hover(|style| {
                                            if selected {
                                                style
                                            } else {
                                                style.bg(rgb(theme::SIDEBAR_ITEM_HOVER))
                                            }
                                        })
                                        .on_mouse_up(
                                            MouseButton::Left,
                                            cx.listener(move |this, _, _, cx| {
                                                this.state.selected_song_id = Some(song_id.clone());
                                                this.state.selected_album_id = album_id.clone();
                                                cx.notify();
                                            }),
                                        )
                                        .child(song_row(
                                            &song,
                                            this.song_image_url(&song, 44.0),
                                            is_playing,
                                            index == 0,
                                        ))
                                })
                                .collect::<Vec<_>>()
                        }),
                    )
                    .track_scroll(&self.songs_scroll_handle)
                    .h(px(700.0)),
                ),
            )
    }

    fn render_albums(&mut self, cx: &mut Context<Self>) -> AnyElement {
        if let Some(album) = self.state.selected_album().cloned() {
            return self.render_album_detail(album, cx).into_any_element();
        }

        let albums = self.state.library.albums.clone();
        let count = albums.len();
        v_flex()
            .gap(px(16.0))
            .child(page_header("Albums", Some(&format!("{count} albums"))))
            .child(
                card_container().children(albums.into_iter().map(|album| {
                    let album_id = album.id.clone();
                    let album_match_id = album.id.clone();
                    let selected = self
                        .state
                        .selected_album_id
                        .as_ref()
                        .is_some_and(|id| Some(id) == album.id.as_ref());
                    div()
                        .cursor_pointer()
                        .rounded(px(6.0))
                        .bg(if selected {
                            rgb(theme::SIDEBAR_ITEM_ACTIVE)
                        } else {
                            rgba(0x00000000)
                        })
                        .hover(|style| {
                            if selected { style } else { style.bg(rgb(theme::SIDEBAR_ITEM_HOVER)) }
                        })
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.state.selected_album_id = album_id.clone();
                                if let Some(song) = this
                                    .state
                                    .library
                                    .songs
                                    .iter()
                                    .find(|song| song.album_id == album_match_id)
                                {
                                    this.state.selected_song_id = Some(song.id.clone());
                                }
                                cx.notify();
                            }),
                        )
                        .child(album_line(&album, self.album_image_url(&album, 52.0)))
                })),
            )
            .into_any_element()
    }

    fn render_artists(&mut self, cx: &mut Context<Self>) -> AnyElement {
        if let Some(artist) = self.state.selected_artist().cloned() {
            return self.render_artist_detail(artist, cx).into_any_element();
        }

        let artists = self.state.library.artists.clone();
        let count = artists.len();
        v_flex()
            .gap(px(16.0))
            .child(page_header("Artists", Some(&format!("{count} artists"))))
            .child(
                card_container().children(artists.into_iter().map(|artist| {
                    let artist_id = artist.id.clone();
                    let selected = self
                        .state
                        .selected_artist_id
                        .as_ref()
                        .is_some_and(|id| id == &artist.id);
                    div()
                        .cursor_pointer()
                        .rounded(px(6.0))
                        .bg(if selected {
                            rgb(theme::SIDEBAR_ITEM_ACTIVE)
                        } else {
                            rgba(0x00000000)
                        })
                        .hover(|style| {
                            if selected { style } else { style.bg(rgb(theme::SIDEBAR_ITEM_HOVER)) }
                        })
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.state.selected_artist_id = Some(artist_id.clone());
                                cx.notify();
                            }),
                        )
                        .child(artist_line(&artist, self.artist_image_url(&artist, 52.0)))
                })),
            )
            .into_any_element()
    }

    fn render_album_detail(&mut self, album: Album, cx: &mut Context<Self>) -> impl IntoElement {
        let album_id = album.id.clone();
        let album_songs = album_id
            .as_deref()
            .map(|id| self.state.songs_for_album(id))
            .unwrap_or_default();
        let album_art_url = self.album_image_url(&album, 220.0);
        let song_count = album_songs.len().max(album.song_count as usize);
        let total_duration = album_songs.iter().map(|song| song.duration.unwrap_or_default()).sum::<f64>();
        let artist_id = album.artist_id.clone();
        let play_album_id = album_id.clone();
        let shuffle_album_id = album_id.clone();

        v_flex()
            .gap(px(24.0))
            .child(
                Button::new("back-to-albums")
                    .icon(Icon::new(IconName::ChevronLeft))
                    .label("Back to albums")
                    .ghost()
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.state.selected_album_id = None;
                        cx.notify();
                    })),
            )
            .child(
                h_flex()
                    .gap(px(24.0))
                    .items_end()
                    .child(cover_art(album_art_url.as_deref(), 220.0, 18.0, false, AppIcon::Disc3))
                    .child(
                        v_flex()
                            .gap(px(12.0))
                            .flex_1()
                            .child(section_header("Album"))
                            .child(
                                div()
                                    .text_size(px(34.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child(album.name.clone()),
                            )
                            .child(
                                h_flex()
                                    .gap(px(8.0))
                                    .items_center()
                                    .child(
                                        div()
                                            .text_size(px(14.0))
                                            .text_color(rgb(theme::MUTED_FG))
                                            .when_some(artist_id.clone(), |this, artist_id| {
                                                this.cursor_pointer()
                                                    .hover(|style| style.text_color(rgb(theme::FOREGROUND)))
                                                    .on_mouse_up(
                                                        MouseButton::Left,
                                                        cx.listener(move |this, _, _, cx| this.show_artist(artist_id.clone(), cx)),
                                                    )
                                            })
                                            .child(album.artist.clone()),
                                    )
                                    .child(detail_dot())
                                    .child(detail_meta_text(&format!("{song_count} track{}", if song_count == 1 { "" } else { "s" })))
                                    .when(total_duration > 0.0, |this| {
                                        this.child(detail_dot()).child(detail_meta_text(&format_duration_long(total_duration)))
                                    }),
                            )
                            .child(
                                h_flex()
                                    .gap(px(8.0))
                                    .child(
                                        Button::new("play-album-detail")
                                            .label("Play")
                                            .icon(Icon::new(AppIcon::Play))
                                            .primary()
                                            .small()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                if let Some(album_id) = play_album_id.clone() {
                                                    this.play_album_by_id(album_id, cx);
                                                }
                                            })),
                                    )
                                    .child(
                                        Button::new("shuffle-album-detail")
                                            .label("Shuffle")
                                            .icon(Icon::new(AppIcon::Shuffle))
                                            .ghost()
                                            .small()
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                if let Some(album_id) = shuffle_album_id.clone() {
                                                    this.shuffle_album_by_id(album_id, cx);
                                                }
                                            })),
                                    ),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .gap(px(12.0))
                    .child(stat_card("Tracks", song_count))
                    .child(stat_card("Minutes", (total_duration / 60.0).round() as usize)),
            )
            .child(section_header("Tracks"))
            .child(
                card_container().children(album_songs.into_iter().enumerate().map(|(index, song)| {
                    let song_id = song.id.clone();
                    let is_playing = self.state.player.current_song().is_some_and(|current| current.id == song.id);
                    let play_album_id = album_id.clone();
                    div()
                        .cursor_pointer()
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                this.state.selected_song_id = Some(song_id.clone());
                                this.state.selected_album_id = play_album_id.clone();
                                if let Some(album_id) = play_album_id.clone() {
                                    let songs = this.state.songs_for_album(&album_id);
                                    if let Some(song_index) = songs.iter().position(|candidate| candidate.id == song_id) {
                                        this.play_queue(songs, song_index, cx);
                                    }
                                }
                            }),
                        )
                        .child(song_row(&song, self.song_image_url(&song, 44.0), is_playing, index == 0))
                })),
            )
    }

    fn render_artist_detail(&mut self, artist: Artist, cx: &mut Context<Self>) -> impl IntoElement {
        let artist_songs = self.state.songs_for_artist_id(&artist.id);
        let artist_albums = self.state.albums_for_artist_id(&artist.id);
        let total_duration = artist_songs.iter().map(|song| song.duration.unwrap_or_default()).sum::<f64>();
        let artist_id = artist.id.clone();

        v_flex()
            .gap(px(24.0))
            .child(
                Button::new("back-to-artists")
                    .icon(Icon::new(IconName::ChevronLeft))
                    .label("Back to artists")
                    .ghost()
                    .small()
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.state.selected_artist_id = None;
                        cx.notify();
                    })),
            )
            .child(
                h_flex()
                    .gap(px(24.0))
                    .items_end()
                    .child(artist_art(self.artist_image_url(&artist, 220.0).as_deref(), &artist.name, 220.0))
                    .child(
                        v_flex()
                            .gap(px(12.0))
                            .flex_1()
                            .child(section_header("Artist"))
                            .child(
                                div()
                                    .text_size(px(34.0))
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .child(artist.name.clone()),
                            )
                            .child(
                                h_flex()
                                    .gap(px(8.0))
                                    .items_center()
                                    .child(detail_meta_text(&format!("{} song{}", artist_songs.len(), if artist_songs.len() == 1 { "" } else { "s" })))
                                    .child(detail_dot())
                                    .child(detail_meta_text(&format!("{} album{}", artist_albums.len(), if artist_albums.len() == 1 { "" } else { "s" })))
                                    .when(total_duration > 0.0, |this| {
                                        this.child(detail_dot()).child(detail_meta_text(&format_duration_long(total_duration)))
                                    }),
                            )
                            .when_some(artist.overview.as_ref(), |this, overview| {
                                this.child(
                                    div()
                                        .text_size(px(13.0))
                                        .text_color(rgb(theme::MUTED_FG))
                                        .max_w(px(760.0))
                                        .child(overview.clone()),
                                )
                            })
                            .child(
                                Button::new("play-artist-detail")
                                    .label("Play artist")
                                    .icon(Icon::new(AppIcon::Play))
                                    .primary()
                                    .small()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        let songs = this.state.songs_for_artist_id(&artist_id);
                                        if !songs.is_empty() {
                                            this.play_queue(songs, 0, cx);
                                        }
                                    })),
                            ),
                    ),
            )
            .child(
                h_flex()
                    .gap(px(12.0))
                    .child(stat_card("Songs", artist_songs.len()))
                    .child(stat_card("Albums", artist_albums.len())),
            )
            .when(!artist_albums.is_empty(), |this| {
                this.child(section_header("Albums")).child(
                    card_container().children(artist_albums.iter().map(|album| {
                        let album_id = album.id.clone();
                        div()
                            .cursor_pointer()
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    if let Some(album_id) = album_id.clone() {
                                        this.show_album(album_id, cx);
                                    }
                                }),
                            )
                            .child(album_line(album, self.album_image_url(album, 52.0)))
                    })),
                )
            })
            .child(section_header("Top songs"))
            .child(
                card_container().children(artist_songs.into_iter().enumerate().map(|(index, song)| {
                    let song_id = song.id.clone();
                    let is_playing = self.state.player.current_song().is_some_and(|current| current.id == song.id);
                    let artist_id = artist.id.clone();
                    div()
                        .cursor_pointer()
                        .on_mouse_up(
                            MouseButton::Left,
                            cx.listener(move |this, _, _, cx| {
                                let songs = this.state.songs_for_artist_id(&artist_id);
                                if let Some(song_index) = songs.iter().position(|candidate| candidate.id == song_id) {
                                    this.state.selected_song_id = Some(song_id.clone());
                                    this.play_queue(songs, song_index, cx);
                                }
                            }),
                        )
                        .child(song_row(&song, self.song_image_url(&song, 44.0), is_playing, index == 0))
                })),
            )
    }

    fn render_playlists(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let playlists = self.state.library.playlists.clone();
        let count = playlists.len();
        let playlist_name = self.playlist_name.clone();

        v_flex()
            .gap(px(16.0))
            .child(page_header("Playlists", Some(&format!("{count} playlists"))))
            // Create controls
            .child(
                h_flex()
                    .gap(px(8.0))
                    .child(div().flex_1().child(Input::new(&playlist_name)))
                    .child(
                        Button::new("create-playlist")
                            .label("Create")
                            .primary()
                            .small()
                            .on_click(cx.listener(|this, _, window, cx| this.create_playlist(window, cx))),
                    )
                    .child(
                        Button::new("rename-playlist")
                            .label("Rename")
                            .small()
                            .on_click(cx.listener(|this, _, window, cx| this.rename_selected_playlist(window, cx))),
                    )
                    .child(
                        Button::new("delete-playlist")
                            .label("Delete")
                            .ghost()
                            .small()
                            .on_click(cx.listener(|this, _, _, cx| this.delete_selected_playlist(cx))),
                    ),
            )
            // Playlist list
            .when(!playlists.is_empty(), |this| {
                this.child(
                    card_container().children(playlists.into_iter().map(|playlist| {
                        let selected = self
                            .state
                            .selected_playlist_id
                            .as_ref()
                            .is_some_and(|id| id == &playlist.id);
                        let playlist_id = playlist.id.clone();
                        let play_playlist_id = playlist.id.clone();
                        h_flex()
                            .justify_between()
                            .cursor_pointer()
                            .px(px(12.0))
                            .py(px(10.0))
                            .rounded(px(6.0))
                            .bg(if selected {
                                rgb(theme::SIDEBAR_ITEM_ACTIVE)
                            } else {
                                rgba(0x00000000)
                            })
                            .hover(|style| {
                                if selected { style } else { style.bg(rgb(theme::SIDEBAR_ITEM_HOVER)) }
                            })
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.state.selected_playlist_id = Some(playlist_id.clone());
                                    cx.notify();
                                }),
                            )
                            .child(playlist_line(&playlist))
                            .child(
                                Button::new(SharedString::from(format!("play-pl-{}", play_playlist_id)))
                                    .icon(Icon::new(AppIcon::Play))
                                    .ghost()
                                    .xsmall()
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.play_playlist(play_playlist_id.clone(), cx);
                                    })),
                            )
                    })),
                )
            })
    }

    fn render_search(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let songs = self.state.filtered_songs();
        let count = songs.len();
        v_flex()
            .gap(px(16.0))
            .child(page_header("Search", Some(&format!("{count} results"))))
            .when(!songs.is_empty(), |this| {
                this.child(
                    card_container().children(songs.into_iter().take(200).enumerate().map(|(i, song)| {
                        let song_id = song.id.clone();
                        div()
                            .cursor_pointer()
                            .rounded(px(6.0))
                            .hover(|style| style.bg(rgb(theme::SIDEBAR_ITEM_HOVER)))
                            .on_mouse_up(
                                MouseButton::Left,
                                cx.listener(move |this, _, _, cx| {
                                    this.state.selected_song_id = Some(song_id.clone());
                                    this.play_song_by_id(song_id.clone(), cx);
                                }),
                            )
                            .child(song_row(&song, self.song_image_url(&song, 44.0), false, i == 0))
                    })),
                )
            })
    }

    fn render_settings(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let minimize_to_tray = self.state.minimize_to_tray;
        let close_to_tray = self.state.close_to_tray;
        let lyrics_server_input = self.lyrics_server_input.clone();

        v_flex()
            .gap(px(16.0))
            .max_w(px(560.0))
            .child(page_header("Settings", None))
            // Behavior section
            .child(section_header("Behavior"))
            .child(
                card_container()
                    .child(
                        h_flex()
                            .justify_between()
                            .px(px(12.0))
                            .py(px(10.0))
                            .child(
                                v_flex()
                                    .gap(px(2.0))
                                    .child(div().text_size(px(13.0)).child("Minimize to tray"))
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(rgb(theme::MUTED_FG))
                                            .child("Keep running in the system tray when minimized"),
                                    ),
                            )
                            .child(
                                Switch::new("minimize-tray")
                                    .checked(minimize_to_tray)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.state.minimize_to_tray = *checked;
                                        this.persist_setting("minimizeToTray", checked.to_string());
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(separator())
                    .child(
                        h_flex()
                            .justify_between()
                            .px(px(12.0))
                            .py(px(10.0))
                            .child(
                                v_flex()
                                    .gap(px(2.0))
                                    .child(div().text_size(px(13.0)).child("Close to tray"))
                                    .child(
                                        div()
                                            .text_size(px(11.0))
                                            .text_color(rgb(theme::MUTED_FG))
                                            .child("Keep running when the window is closed"),
                                    ),
                            )
                            .child(
                                Switch::new("close-tray")
                                    .checked(close_to_tray)
                                    .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                        this.state.close_to_tray = *checked;
                                        this.persist_setting("closeToTray", checked.to_string());
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // Lyrics section
            .child(section_header("Lyrics"))
            .child(
                card_container()
                    .child(
                        v_flex()
                            .gap(px(8.0))
                            .px(px(12.0))
                            .py(px(10.0))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(rgb(theme::MUTED_FG))
                                    .child("Optional custom lyrics server URL used when fetching lyrics."),
                            )
                            .child(
                                h_flex()
                                    .gap(px(8.0))
                                    .items_center()
                                    .child(div().flex_1().child(Input::new(&lyrics_server_input).cleanable(true)))
                                    .child(
                                        Button::new("save-lyrics-url")
                                            .label("Save")
                                            .ghost()
                                            .xsmall()
                                            .on_click(cx.listener(|this, _, _, cx| this.save_lyrics_server_url(cx))),
                                    ),
                            ),
                    ),
            )
    }

    fn render_right_panel(&mut self, cx: &mut Context<Self>) -> impl IntoElement {
        let title = match self.state.right_panel {
            RightPanel::Queue => "QUEUE",
            RightPanel::Lyrics => "LYRICS",
            RightPanel::None => "",
        };

        v_flex()
            .id("right-panel-scroll")
            .w(px(360.0))
            .flex_shrink_0()
            .h_full()
            .border_l_1()
            .border_color(rgb(theme::BORDER))
            .bg(rgb(theme::BACKGROUND_DARK))
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .px(px(16.0))
                    .py(px(14.0))
                    .border_b_1()
                    .border_color(rgb(theme::BORDER))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(theme::MUTED_FG))
                            .child(title),
                    )
                    .child(
                        Button::new("close-right-panel")
                            .icon(Icon::new(IconName::PanelRight))
                            .ghost()
                            .small()
                            .tooltip("Close panel")
                            .on_click(cx.listener(|this, _, _, cx| this.close_right_panel(cx))),
                    ),
            )
            .child(match self.state.right_panel {
                RightPanel::Queue => self.render_queue_panel(cx).into_any_element(),
                RightPanel::Lyrics => self.render_lyrics_panel().into_any_element(),
                RightPanel::None => div().into_any_element(),
            })
    }

    fn render_queue_panel(&mut self, _cx: &mut Context<Self>) -> impl IntoElement {
        let queue_len = self.state.player.queue.len();
        let current_index = self.state.player.current_index.unwrap_or_default();
        let start = current_index.saturating_sub(6);
        let hidden_before = start;
        let hidden_after = queue_len.saturating_sub(start + 18);

        div()
            .flex_1()
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .p(px(16.0))
                    .gap(px(12.0))
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child("Up next"),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(rgb(theme::MUTED_FG))
                                    .child(format!("{} track{}", queue_len, if queue_len == 1 { "" } else { "s" })),
                            ),
                    )
                    .when(queue_len == 0, |this| {
                        this.child(panel_empty_state(
                            "No songs in queue",
                            "Start playback from Songs, Albums, or Playlists.",
                        ))
                    })
                    .when(hidden_before > 0, |this| {
                        this.child(
                            div()
                                .text_size(px(10.0))
                                .text_color(rgb(theme::MUTED_FG))
                                .child(format!("{} earlier track{}", hidden_before, if hidden_before == 1 { "" } else { "s" })),
                        )
                    })
                    .children(
                        self.state
                            .player
                            .queue
                            .iter()
                            .enumerate()
                            .skip(start)
                            .take(18)
                            .map(|(index, song)| {
                                let current = self.state.player.current_index == Some(index);
                                queue_item(song, index + 1, current)
                            }),
                    )
                    .when(hidden_after > 0, |this| {
                        this.child(
                            div()
                                .text_size(px(10.0))
                                .text_color(rgb(theme::MUTED_FG))
                                .child(format!("{} more track{}", hidden_after, if hidden_after == 1 { "" } else { "s" })),
                        )
                    }),
            )
    }

    fn render_lyrics_panel(&mut self) -> impl IntoElement {
        let has_lyrics = self.state.lyrics.is_some();

        div()
            .flex_1()
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .p(px(16.0))
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .text_color(rgb(theme::MUTED_FG))
                            .child(self.state.lyrics_status.clone()),
                    )
                    .when(!has_lyrics, |this| {
                        this.child(panel_empty_state(
                            "No lyrics to show",
                            "Lyrics appear here when the current song has synced or plain text lyrics.",
                        ))
                    })
                    .children(
                        self.state
                            .lyrics
                            .as_ref()
                            .map(|lyrics| {
                                if !lyrics.synced.is_empty() {
                                    lyrics
                                        .synced
                                        .iter()
                                        .map(|line| lyrics_line(&line.line))
                                        .collect::<Vec<_>>()
                                } else {
                                    lyrics.plain.iter().map(|line| lyrics_line(line)).collect::<Vec<_>>()
                                }
                            })
                            .unwrap_or_default(),
                    ),
            )
    }

    fn render_player_bar(&mut self, _progress: f64, _right_panel_open: bool, cx: &mut Context<Self>) -> impl IntoElement {
        let song = self.state.player.current_song().cloned();
        let position = self.state.player.position_secs.floor();
        let duration = self.state.player.duration_secs;
        let is_playing = self.state.player.is_playing;
        let seek_slider = self.seek_slider.clone();
        let volume_slider = self.volume_slider.clone();
        let is_queue_open = self.state.right_panel == RightPanel::Queue;
        let is_lyrics_open = self.state.right_panel == RightPanel::Lyrics;
        let is_shuffled = self.state.player.is_shuffled;
        let repeat_mode = self.state.player.repeat_mode;
        let is_favorite = song
            .as_ref()
            .is_some_and(|s| self.state.favorite_ids.iter().any(|id| id == &s.id));
        let song_art = song.as_ref().and_then(|song| self.song_image_url(song, 68.0));
        let artist_name = song
            .as_ref()
            .and_then(|s| s.artists.as_ref().map(|artists| artists.join(", ")))
            .unwrap_or_else(|| "Nothing playing".to_string());
        let album_name = song
            .as_ref()
            .and_then(|s| s.album.clone())
            .unwrap_or_else(|| "Choose something from your library".to_string());
        let song_file_info = song
            .as_ref()
            .map(format_song_file_info)
            .unwrap_or_else(|| "Unknown format".to_string());

        div()
            .absolute()
            .bottom_0()
            .left_0()
            .right_0()
            .px(px(24.0))
            .pb(px(18.0))
            .occlude()
            .child(
                h_flex()
                    .justify_center()
                    .w_full()
                    .child(
                        h_flex()
                            .justify_between()
                            .items_center()
                            .gap(px(16.0))
                            .w_full()
                            .max_w(px(980.0))
                            .px(px(16.0))
                            .py(px(10.0))
                            .rounded(px(18.0))
                            .border_1()
                            .border_color(rgb(theme::BORDER))
                            .bg(rgb(theme::PLAYER_BG))
                            .child(
                                h_flex()
                                    .gap(px(12.0))
                                    .items_center()
                                    .w(px(252.0))
                                    .min_w(px(0.0))
                                    .flex_shrink_0()
                                    .child(cover_art(song_art.as_deref(), 46.0, 10.0, false, AppIcon::Music))
                                    .child(
                                        v_flex()
                                            .gap(px(2.0))
                                            .min_w(px(0.0))
                                            .overflow_hidden()
                                            .flex_1()
                                            .child(
                                                div()
                                                    .w_full()
                                                    .text_size(px(12.0))
                                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                                    .truncate()
                                                    .child(
                                                        song.as_ref()
                                                            .map(|s| s.name.clone())
                                                            .unwrap_or_else(|| "Nothing playing".to_string()),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .w_full()
                                                    .text_size(px(10.0))
                                                    .text_color(rgb(theme::MUTED_FG))
                                                    .truncate()
                                                    .child(format!("{artist_name}  ·  {album_name}")),
                                            )
                                            .child(
                                                div()
                                                    .w_full()
                                                    .text_size(px(10.0))
                                                    .text_color(rgb(theme::MUTED_FG))
                                                    .truncate()
                                                    .child(song_file_info),
                                            ),
                                    )
                            )
                            .child(
                                v_flex()
                                    .gap(px(6.0))
                                    .flex_1()
                                    .max_w(px(360.0))
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .justify_center()
                                            .gap(px(4.0))
                                            .child(
                                                Button::new("shuffle-btn")
                                                    .icon(Icon::new(AppIcon::Shuffle))
                                                    .small()
                                                    .tooltip("Shuffle")
                                                    .when(is_shuffled, |this| this.primary())
                                                    .when(!is_shuffled, |this| this.ghost())
                                                    .on_click(cx.listener(|this, _, _, cx| this.toggle_shuffle(cx))),
                                            )
                                            .child(
                                                Button::new("prev-btn")
                                                    .icon(Icon::new(AppIcon::SkipBack))
                                                    .ghost()
                                                    .small()
                                                    .on_click(cx.listener(|this, _, _, cx| this.previous_track_action(cx))),
                                            )
                                            .child(
                                                Button::new("play-pause-btn")
                                                    .icon(Icon::new(if is_playing { AppIcon::Pause } else { AppIcon::Play }))
                                                    .primary()
                                                    .rounded(px(999.0))
                                                    .w(px(42.0))
                                                    .h(px(42.0))
                                                    .on_click(cx.listener(|this, _, _, cx| this.toggle_play_pause(cx))),
                                            )
                                            .child(
                                                Button::new("next-btn")
                                                    .icon(Icon::new(AppIcon::SkipForward))
                                                    .ghost()
                                                    .small()
                                                    .on_click(cx.listener(|this, _, _, cx| this.next_track_action(cx))),
                                            )
                                            .child(
                                                Button::new("repeat-btn")
                                                    .icon(Icon::new(AppIcon::Repeat))
                                                    .small()
                                                    .tooltip(match repeat_mode {
                                                        RepeatMode::None => "Repeat off",
                                                        RepeatMode::All => "Repeat all",
                                                        RepeatMode::One => "Repeat one",
                                                    })
                                                    .when(repeat_mode != RepeatMode::None, |this| this.primary())
                                                    .when(repeat_mode == RepeatMode::None, |this| this.ghost())
                                                    .on_click(cx.listener(|this, _, _, cx| this.cycle_repeat_mode(cx))),
                                            ),
                                    )
                                    .child(
                                        h_flex()
                                            .items_center()
                                            .gap(px(8.0))
                                            .child(
                                                div()
                                                    .text_size(px(10.0))
                                                    .text_color(rgb(theme::MUTED_FG))
                                                    .w(px(34.0))
                                                    .text_right()
                                                    .child(format_duration(position)),
                                            )
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .child(Slider::new(&seek_slider))
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(11.0))
                                                    .text_color(rgb(theme::MUTED_FG))
                                                    .w(px(34.0))
                                                    .child(format_duration(duration)),
                                            ),
                                    ),
                            )
                            .child(
                                h_flex()
                                    .gap(px(6.0))
                                    .items_center()
                                    .justify_end()
                                    .w(px(214.0))
                                    .flex_shrink_0()
                                    .child(
                                        Button::new("favorite-current")
                                            .icon(Icon::new(if is_favorite { IconName::HeartOff } else { IconName::Heart }))
                                            .ghost()
                                            .small()
                                            .tooltip(if is_favorite { "Unfavorite" } else { "Favorite" })
                                            .on_click(cx.listener(|this, _, _, cx| this.toggle_favorite_current(cx))),
                                    )
                                    .child(
                                        Button::new("toggle-queue-panel")
                                            .icon(Icon::new(AppIcon::ListMusic))
                                            .small()
                                            .tooltip("Queue")
                                            .when(is_queue_open, |this| this.primary())
                                            .when(!is_queue_open, |this| this.ghost())
                                            .on_click(cx.listener(|this, _, _, cx| this.toggle_queue_panel(cx))),
                                    )
                                    .child(
                                        Button::new("toggle-lyrics-panel")
                                            .icon(Icon::new(AppIcon::MicVocal))
                                            .small()
                                            .tooltip("Lyrics")
                                            .when(is_lyrics_open, |this| this.primary())
                                            .when(!is_lyrics_open, |this| this.ghost())
                                            .on_click(cx.listener(|this, _, _, cx| this.toggle_lyrics_panel(cx))),
                                    )
                                    .child(
                                        h_flex()
                                            .gap(px(6.0))
                                            .items_center()
                                            .child(Icon::new(AppIcon::Volume2).small().text_color(rgb(theme::MUTED_FG)))
                                            .child(div().w(px(64.0)).child(Slider::new(&volume_slider))),
                                    ),
                            ),
                    ),
            )
    }
}

impl Focusable for DesktopApp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for DesktopApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Flush any pending input clears from async callbacks
        self.flush_pending_clears(window, cx);
        let seek_value = (self.state.player.position_secs / self.state.player.duration_secs.max(1.0) * 1000.0)
            .clamp(0.0, 1000.0) as f32;
        self.set_seek_slider_value(seek_value, window, cx);

        div()
            .track_focus(&self.focus_handle(cx))
            .size_full()
            .child(if self.state.session.is_some() {
                self.render_shell(window, cx).into_any_element()
            } else {
                self.render_login(cx).into_any_element()
            })
    }
}

// ---------------------------------------------------------------
// Standalone helper functions
// ---------------------------------------------------------------

fn resolve_app_data_dir() -> PathBuf {
    let base = dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
    let path = base.join(APP_DIR_NAME);
    let _ = std::fs::create_dir_all(&path);
    path
}

fn normalize_server_url(url: &str) -> String {
    let trimmed = url.trim();
    let with_scheme = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    with_scheme.trim_end_matches('/').to_string()
}

fn derive_albums(songs: &[Song], credentials: Option<&Credentials>) -> Vec<Album> {
    let mut seen = std::collections::HashSet::new();
    let mut albums = Vec::new();
    for song in songs {
        let Some(album_id) = song.album_id.clone() else {
            continue;
        };
        if !seen.insert(album_id.clone()) {
            continue;
        }
        let art_url = credentials.and_then(|credentials| {
            aurelia_core::build_image_url(
                credentials.server_url.clone(),
                credentials.token.clone(),
                album_id.clone(),
                "Primary".to_string(),
                Some(300),
                Some(90),
            )
            .ok()
            .flatten()
        });
        albums.push(Album {
            id: Some(album_id),
            name: song.album.clone().unwrap_or_else(|| "Unknown Album".to_string()),
            artist: song
                .artists
                .as_ref()
                .and_then(|artists| artists.first().cloned())
                .unwrap_or_else(|| "Unknown Artist".to_string()),
            artist_id: song.artist_ids.as_ref().and_then(|ids| ids.first().cloned()),
            album_art_url: art_url,
            song_count: songs
                .iter()
                .filter(|candidate| candidate.album_id == song.album_id)
                .count() as i64,
            songs: None,
            image_tags: None,
            provider_ids: None,
            date_created: song.date_created.clone(),
            date_modified: song.date_modified.clone(),
        });
    }
    albums.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    albums
}

fn derive_artists(songs: &[Song], credentials: Option<&Credentials>) -> Vec<Artist> {
    let mut seen = std::collections::HashSet::new();
    let mut artists = Vec::new();
    for song in songs {
        if let Some(names) = &song.artists {
            for (index, name) in names.iter().enumerate() {
                let id = song
                    .artist_ids
                    .as_ref()
                    .and_then(|ids| ids.get(index).cloned())
                    .unwrap_or_else(|| name.clone());
                if !seen.insert(id.clone()) {
                    continue;
                }
                let image_url = credentials.and_then(|credentials| {
                    aurelia_core::build_image_url(
                        credentials.server_url.clone(),
                        credentials.token.clone(),
                        id.clone(),
                        "Primary".to_string(),
                        Some(300),
                        Some(90),
                    )
                    .ok()
                    .flatten()
                });
                artists.push(Artist {
                    name: name.clone(),
                    id,
                    image_tags: None,
                    image_url,
                    overview: None,
                    provider_ids: None,
                    community_rating: None,
                    song_count: Some(
                        songs
                            .iter()
                            .filter(|candidate| {
                                candidate
                                    .artists
                                    .as_ref()
                                    .is_some_and(|artists| artists.iter().any(|artist| artist == name))
                            })
                            .count() as i64,
                    ),
                    date_modified: None,
                    songs: None,
                });
            }
        }
    }
    artists.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    artists
}

fn image_request_size(size_px: f32) -> u32 {
    size_px.ceil().max(1.0) as u32
}

fn raw_hwnd(window: &Window) -> Option<*mut std::ffi::c_void> {
    let handle = raw_window_handle::HasWindowHandle::window_handle(window).ok()?;
    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get() as *mut std::ffi::c_void),
        _ => None,
    }
}

// -- UI helper functions --

fn page_header(title: &str, subtitle: Option<&str>) -> impl IntoElement {
    h_flex()
        .items_end()
        .gap(px(8.0))
        .mb(px(4.0))
        .child(
            div()
                .text_size(px(24.0))
                .font_weight(gpui::FontWeight::BOLD)
                .child(title.to_string()),
        )
        .when_some(subtitle, |this, sub| {
            this.child(
                div()
                    .text_size(px(12.0))
                    .text_color(rgb(theme::MUTED_FG))
                    .pb(px(3.0))
                    .child(sub.to_string()),
            )
        })
}

fn section_header(title: &str) -> impl IntoElement {
    div()
        .text_size(px(11.0))
        .font_weight(gpui::FontWeight::SEMIBOLD)
        .text_color(rgb(theme::MUTED_FG))
        .mt(px(8.0))
        .child(title.to_uppercase())
}

fn detail_dot() -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .text_color(rgb(theme::MUTED_FG))
        .child("-")
}

fn detail_meta_text(text: &str) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .text_color(rgb(theme::MUTED_FG))
        .child(text.to_string())
}

fn carousel_header(
    title: &str,
    prev_id: &'static str,
    next_id: &'static str,
    cx: &mut Context<DesktopApp>,
    on_prev: impl Fn(&mut DesktopApp, &mut Context<DesktopApp>) + 'static,
    on_next: impl Fn(&mut DesktopApp, &mut Context<DesktopApp>) + 'static,
) -> impl IntoElement {
    h_flex()
        .justify_between()
        .items_center()
        .child(section_header(title))
        .child(
            h_flex()
                .gap(px(8.0))
                .child(
                    Button::new(prev_id)
                        .icon(Icon::new(IconName::ChevronLeft))
                        .ghost()
                        .small()
                        .on_click(cx.listener(move |this, _, _, cx| on_prev(this, cx))),
                )
                .child(
                    Button::new(next_id)
                        .icon(Icon::new(IconName::ChevronRight))
                        .ghost()
                        .small()
                        .on_click(cx.listener(move |this, _, _, cx| on_next(this, cx))),
                ),
        )
}

fn field_label(text: &str) -> impl IntoElement {
    div()
        .text_size(px(12.0))
        .font_weight(gpui::FontWeight::MEDIUM)
        .text_color(rgb(theme::MUTED_FG))
        .child(text.to_string())
}

fn card_container() -> gpui::Div {
    v_flex()
        .rounded(px(8.0))
        .border_1()
        .border_color(rgb(theme::BORDER))
        .bg(rgb(theme::CARD))
        .overflow_hidden()
}

fn stat_card(label: &str, value: usize) -> impl IntoElement {
    v_flex()
        .flex_1()
        .gap(px(4.0))
        .p(px(16.0))
        .rounded(px(8.0))
        .border_1()
        .border_color(rgb(theme::BORDER))
        .bg(rgb(theme::CARD))
        .child(
            div()
                .text_size(px(24.0))
                .font_weight(gpui::FontWeight::BOLD)
                .child(value.to_string()),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(theme::MUTED_FG))
                .child(label.to_string()),
        )
}

fn separator() -> impl IntoElement {
    div()
        .h(px(1.0))
        .w_full()
        .bg(rgb(theme::BORDER))
}

fn panel_empty_state(title: &str, body: &str) -> impl IntoElement {
    v_flex()
        .gap(px(6.0))
        .rounded(px(12.0))
        .border_1()
        .border_color(rgb(theme::BORDER))
        .bg(rgb(theme::CARD))
        .p(px(16.0))
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(title.to_string()),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(theme::MUTED_FG))
                .child(body.to_string()),
        )
}

fn lyrics_line(line: &str) -> impl IntoElement {
    div()
        .text_size(px(13.0))
        .py(px(2.0))
        .text_color(rgb(theme::FOREGROUND))
        .child(line.to_string())
}

fn song_row(song: &Song, image_url: Option<String>, is_playing: bool, is_first: bool) -> impl IntoElement {
    let duration = format_duration(song.duration.unwrap_or_default());
    let artist = song
        .artists
        .as_ref()
        .and_then(|a| a.first().cloned())
        .unwrap_or_else(|| "Unknown Artist".to_string());
    let album = song
        .album
        .clone()
        .unwrap_or_else(|| "Unknown Album".to_string());

    h_flex()
        .justify_between()
        .px(px(12.0))
        .py(px(8.0))
        .when(!is_first, |this| {
            this.border_t_1().border_color(rgb(theme::BORDER))
        })
        .child(
            h_flex()
                .gap(px(12.0))
                .items_center()
                .flex_1()
                .child(cover_art(image_url.as_deref(), 44.0, 8.0, false, AppIcon::Music))
                .child(
                    v_flex()
                        .gap(px(2.0))
                        .overflow_hidden()
                        .flex_1()
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(if is_playing {
                                    gpui::FontWeight::SEMIBOLD
                                } else {
                                    gpui::FontWeight::NORMAL
                                })
                                .text_color(rgb(theme::FOREGROUND))
                                .child(song.name.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(theme::MUTED_FG))
                                .child(format!("{artist}  ·  {album}")),
                        ),
                ),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(theme::MUTED_FG))
                .flex_shrink_0()
                .pl(px(12.0))
                .child(duration),
        )
}

fn queue_item(song: &Song, number: usize, is_current: bool) -> impl IntoElement {
    h_flex()
        .gap(px(8.0))
        .px(px(8.0))
        .py(px(4.0))
        .rounded(px(4.0))
        .bg(if is_current {
            rgb(theme::SIDEBAR_ITEM_ACTIVE)
        } else {
            rgba(0x00000000)
        })
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(theme::MUTED_FG))
                .w(px(20.0))
                .text_right()
                .child(number.to_string()),
        )
        .child(
            v_flex()
                .flex_1()
                .overflow_hidden()
                .child(
                    div()
                        .text_size(px(12.0))
                        .font_weight(if is_current {
                            gpui::FontWeight::MEDIUM
                        } else {
                            gpui::FontWeight::NORMAL
                        })
                        .child(song.name.clone()),
                )
                .child(
                    div()
                        .text_size(px(10.0))
                        .text_color(rgb(theme::MUTED_FG))
                        .child(
                            song.artists
                                .as_ref()
                                .and_then(|a| a.first().cloned())
                                .unwrap_or_default(),
                        ),
                ),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(rgb(theme::MUTED_FG))
                .child(format_duration(song.duration.unwrap_or_default())),
        )
}

fn album_line(album: &Album, image_url: Option<String>) -> impl IntoElement {
    h_flex()
        .justify_between()
        .px(px(12.0))
        .py(px(10.0))
        .child(
            h_flex()
                .gap(px(12.0))
                .items_center()
                .child(cover_art(image_url.as_deref(), 52.0, 10.0, false, AppIcon::Disc3))
                .child(
                    v_flex()
                        .gap(px(2.0))
                        .child(
                            div()
                                .text_size(px(13.0))
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .child(album.name.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(theme::MUTED_FG))
                                .child(album.artist.clone()),
                        ),
                ),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(theme::MUTED_FG))
                .child(format!(
                    "{} track{}",
                    album.song_count,
                    if album.song_count == 1 { "" } else { "s" }
                )),
        )
}

fn artist_line(artist: &Artist, image_url: Option<String>) -> impl IntoElement {
    let count = artist.song_count.unwrap_or_default();
    h_flex()
        .justify_between()
        .px(px(12.0))
        .py(px(10.0))
        .child(
            h_flex()
                .gap(px(12.0))
                .items_center()
                .child(artist_art(image_url.as_deref(), &artist.name, 52.0))
                .child(
                    div()
                        .text_size(px(13.0))
                        .font_weight(gpui::FontWeight::MEDIUM)
                        .child(artist.name.clone()),
                ),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(theme::MUTED_FG))
                .child(format!(
                    "{count} song{}",
                    if count == 1 { "" } else { "s" }
                )),
        )
}

fn playlist_line(playlist: &Playlist) -> impl IntoElement {
    let count = playlist.child_count.unwrap_or_default();
    v_flex()
        .gap(px(2.0))
        .child(
            div()
                .text_size(px(13.0))
                .font_weight(gpui::FontWeight::MEDIUM)
                .child(playlist.name.clone()),
        )
        .child(
            div()
                .text_size(px(11.0))
                .text_color(rgb(theme::MUTED_FG))
                .child(format!(
                    "{count} item{}",
                    if count == 1 { "" } else { "s" }
                )),
        )
}

fn cover_art(
    image_url: Option<&str>,
    size_px: f32,
    radius_px: f32,
    round: bool,
    icon: AppIcon,
) -> AnyElement {
    let placeholder = move || art_placeholder(size_px, radius_px, round, icon);
    if let Some(image_url) = image_url.filter(|url| !url.trim().is_empty()) {
        let image = img(image_url.to_string())
            .w(px(size_px))
            .h(px(size_px))
            .object_fit(ObjectFit::Cover)
            .with_fallback(placeholder)
            .with_loading(placeholder);

        if round {
            image.rounded_full().into_any_element()
        } else {
            image.rounded(px(radius_px)).into_any_element()
        }
    } else {
        placeholder()
    }
}

fn artist_art(image_url: Option<&str>, name: &str, size_px: f32) -> AnyElement {
    if let Some(image_url) = image_url.filter(|url| !url.trim().is_empty()) {
        Avatar::new()
            .src(image_url.to_string())
            .name(name.to_string())
            .w(px(size_px))
            .h(px(size_px))
            .into_any_element()
    } else {
        Avatar::new()
            .name(name.to_string())
            .w(px(size_px))
            .h(px(size_px))
            .into_any_element()
    }
}

fn art_placeholder(
    size_px: f32,
    radius_px: f32,
    round: bool,
    icon: AppIcon,
) -> AnyElement {
    let base = div()
        .w(px(size_px))
        .h(px(size_px))
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(theme::SIDEBAR_ITEM_HOVER))
        .text_color(rgb(theme::MUTED_FG))
        .border_1()
        .border_color(rgb(theme::BORDER))
        .child(Icon::new(icon));

    if round {
        base.rounded_full().into_any_element()
    } else {
        base.rounded(px(radius_px)).into_any_element()
    }
}

struct ReqwestHttpClient {
    client: reqwest::Client,
    user_agent: http::HeaderValue,
    runtime: Arc<Runtime>,
}

impl ReqwestHttpClient {
    fn new(user_agent: &str) -> anyhow::Result<Self> {
        let client = reqwest::Client::builder().build()?;
        let user_agent = http::HeaderValue::from_str(user_agent)?;
        Ok(Self {
            client,
            user_agent,
            runtime: image_http_runtime(),
        })
    }
}

impl HttpClient for ReqwestHttpClient {
    fn user_agent(&self) -> Option<&http::HeaderValue> {
        Some(&self.user_agent)
    }

    fn send(
        &self,
        req: Request<AsyncBody>,
    ) -> futures::future::BoxFuture<'static, anyhow::Result<Response<AsyncBody>>> {
        let client = self.client.clone();
        let user_agent = self.user_agent.clone();
        let runtime = Arc::clone(&self.runtime);
        Box::pin(async move {
            let (parts, body) = req.into_parts();
            let body_bytes = match body.0 {
                gpui::http_client::Inner::Empty => Vec::new(),
                gpui::http_client::Inner::Bytes(cursor) => cursor.into_inner().to_vec(),
                gpui::http_client::Inner::AsyncReader(mut reader) => {
                    let mut bytes = Vec::new();
                    futures::AsyncReadExt::read_to_end(&mut reader, &mut bytes).await?;
                    bytes
                }
            };

            let response = runtime
                .spawn(async move {
                    let uri = parts.uri.to_string();
                    let method = parts.method.clone();
                    let mut request = client.request(method, uri);

                    for (name, value) in &parts.headers {
                        request = request.header(name, value);
                    }

                    if !parts.headers.contains_key(http::header::USER_AGENT) {
                        request = request.header(http::header::USER_AGENT, user_agent);
                    }

                    let response = request.body(body_bytes).send().await?;
                    let status = response.status();
                    let headers = response.headers().clone();
                    let bytes = response.bytes().await?;
                    Ok::<_, anyhow::Error>((status, headers, bytes))
                })
                .await??;

            let (status, headers, bytes) = response;

            let mut builder = http::Response::builder().status(status);
            for (name, value) in &headers {
                builder = builder.header(name, value);
            }

            Ok(builder.body(AsyncBody::from(bytes.to_vec()))?)
        })
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }
}

fn image_http_runtime() -> Arc<Runtime> {
    static RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| Arc::new(Runtime::new().expect("failed to create image http runtime")))
        .clone()
}

async fn load_blurred_featured_background(
    runtime: Arc<Runtime>,
    image_url: String,
) -> anyhow::Result<Arc<GpuiImage>> {
    let bytes = runtime
        .spawn(async move {
            let response = reqwest::get(&image_url).await?;
            let status = response.status();
            if !status.is_success() {
                anyhow::bail!("image request failed with status {status}");
            }

            let bytes = response.bytes().await?;
            Ok::<_, anyhow::Error>(bytes.to_vec())
        })
        .await??;

    let blurred = runtime
        .spawn_blocking(move || build_blurred_featured_background(bytes))
        .await??;

    Ok(Arc::new(blurred))
}

fn build_blurred_featured_background(bytes: Vec<u8>) -> anyhow::Result<GpuiImage> {
    let format = image::guess_format(&bytes)?;
    let image = image::load_from_memory_with_format(&bytes, format)?;
    let resized = image.resize_to_fill(1400, 420, FilterType::Lanczos3);
    let blurred = resized.blur(32.0).into_rgba8();

    let mut encoded = Vec::new();
    image::DynamicImage::ImageRgba8(blurred)
        .write_to(&mut Cursor::new(&mut encoded), image::ImageFormat::Png)?;

    Ok(GpuiImage::from_bytes(GpuiImageFormat::Png, encoded))
}
