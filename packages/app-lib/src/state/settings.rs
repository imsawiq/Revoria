//! Theseus settings file

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;

// Types
/// Global Theseus settings
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub max_concurrent_downloads: usize,
    pub max_concurrent_writes: usize,

    pub theme: Theme,
    pub default_page: DefaultPage,
    pub collapsed_navigation: bool,
    pub hide_nametag_skins_page: bool,
    pub advanced_rendering: bool,
    pub glass_blur: i64,
    pub glass_border_opacity: f64,
    pub background_effect: String,
    pub background_effect_intensity: i64,
    pub page_background_path: String,
    pub page_background_opacity: f64,
    pub proxy_enabled: bool,
    pub proxy_type: ProxyType,
    pub proxy_host: String,
    pub proxy_port: u16,
    pub proxy_auth_enabled: bool,
    pub proxy_username: String,
    pub proxy_password: String,
    pub native_decorations: bool,
    pub toggle_sidebar: bool,

    pub telemetry: bool,
    pub discord_rpc: bool,
    pub personalized_ads: bool,

    pub onboarded: bool,

    pub extra_launch_args: Vec<String>,
    pub custom_env_vars: Vec<(String, String)>,
    pub memory: MemorySettings,
    pub force_fullscreen: bool,
    pub game_resolution: WindowSize,
    pub hide_on_process_start: bool,
    pub hooks: Hooks,

    pub custom_dir: Option<String>,
    pub prev_custom_dir: Option<String>,
    pub migrated: bool,

    pub developer_mode: bool,
    pub feature_flags: HashMap<FeatureFlag, bool>,

    pub skipped_update: Option<String>,
    pub pending_update_toast_for_version: Option<String>,
    pub auto_download_updates: Option<bool>,

    pub version: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlag {
    PagePath,
    ProjectBackground,
    WorldsTab,
    WorldsInHome,
}

impl Settings {
    const CURRENT_VERSION: usize = 7;

    pub async fn get(
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<Self> {
        let res = sqlx::query!(
            "
            SELECT
                max_concurrent_writes, max_concurrent_downloads,
                theme, default_page, collapsed_navigation, hide_nametag_skins_page, advanced_rendering, native_decorations,
                glass_blur, glass_border_opacity, background_effect, background_effect_intensity,
                page_background_path, page_background_opacity,
                proxy_enabled, proxy_type, proxy_host, proxy_port, proxy_auth_enabled, proxy_username, proxy_password,
                discord_rpc, developer_mode, telemetry, personalized_ads,
                onboarded,
                json(extra_launch_args) extra_launch_args, json(custom_env_vars) custom_env_vars,
                mc_memory_max, mc_force_fullscreen, mc_game_resolution_x, mc_game_resolution_y, hide_on_process_start,
                hook_pre_launch, hook_wrapper, hook_post_exit,
                custom_dir, prev_custom_dir, migrated, json(feature_flags) feature_flags, toggle_sidebar,
                skipped_update, pending_update_toast_for_version, auto_download_updates,
                version
            FROM settings
            "
        )
            .fetch_one(exec)
            .await?;

        Ok(Self {
            max_concurrent_downloads: res.max_concurrent_downloads as usize,
            max_concurrent_writes: res.max_concurrent_writes as usize,
            theme: Theme::from_string(&res.theme),
            default_page: DefaultPage::from_string(&res.default_page),
            collapsed_navigation: res.collapsed_navigation == 1,
            hide_nametag_skins_page: res.hide_nametag_skins_page == 1,
            advanced_rendering: res.advanced_rendering == 1,
            glass_blur: res.glass_blur,
            glass_border_opacity: res.glass_border_opacity,
            background_effect: res.background_effect,
            background_effect_intensity: res.background_effect_intensity,
            page_background_path: res.page_background_path,
            page_background_opacity: res.page_background_opacity,
            proxy_enabled: res.proxy_enabled == 1,
            proxy_type: ProxyType::from_string(&res.proxy_type),
            proxy_host: res.proxy_host,
            proxy_port: res.proxy_port as u16,
            proxy_auth_enabled: res.proxy_auth_enabled == 1,
            proxy_username: res.proxy_username,
            proxy_password: res.proxy_password,
            native_decorations: res.native_decorations == 1,
            toggle_sidebar: res.toggle_sidebar == 1,
            telemetry: res.telemetry == 1,
            discord_rpc: res.discord_rpc == 1,
            developer_mode: res.developer_mode == 1,
            personalized_ads: res.personalized_ads == 1,
            onboarded: res.onboarded == 1,
            extra_launch_args: res
                .extra_launch_args
                .as_ref()
                .and_then(|x| serde_json::from_str(x).ok())
                .unwrap_or_default(),
            custom_env_vars: res
                .custom_env_vars
                .as_ref()
                .and_then(|x| serde_json::from_str(x).ok())
                .unwrap_or_default(),
            memory: MemorySettings {
                maximum: res.mc_memory_max as u32,
            },
            force_fullscreen: res.mc_force_fullscreen == 1,
            game_resolution: WindowSize(
                res.mc_game_resolution_x as u16,
                res.mc_game_resolution_y as u16,
            ),
            hide_on_process_start: res.hide_on_process_start == 1,
            hooks: Hooks {
                pre_launch: res.hook_pre_launch,
                wrapper: res.hook_wrapper,
                post_exit: res.hook_post_exit,
            },
            custom_dir: res.custom_dir,
            prev_custom_dir: res.prev_custom_dir,
            migrated: res.migrated == 1,
            feature_flags: res
                .feature_flags
                .as_ref()
                .and_then(|x| serde_json::from_str(x).ok())
                .unwrap_or_default(),
            skipped_update: res.skipped_update,
            pending_update_toast_for_version: res
                .pending_update_toast_for_version,
            auto_download_updates: res.auto_download_updates.map(|x| x == 1),
            version: res.version as usize,
        })
    }

    pub async fn update(
        &self,
        exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
    ) -> crate::Result<()> {
        let max_concurrent_writes = self.max_concurrent_writes as i32;
        let max_concurrent_downloads = self.max_concurrent_downloads as i32;
        let theme = self.theme.as_str();
        let default_page = self.default_page.as_str();
        let background_effect = self.background_effect.as_str();
        let page_background_path = self.page_background_path.as_str();
        let proxy_type = self.proxy_type.as_str();
        let extra_launch_args = serde_json::to_string(&self.extra_launch_args)?;
        let custom_env_vars = serde_json::to_string(&self.custom_env_vars)?;
        let feature_flags = serde_json::to_string(&self.feature_flags)?;
        let version = self.version as i64;

        sqlx::query!(
            "
            UPDATE settings
            SET
                max_concurrent_writes = $1,
                max_concurrent_downloads = $2,

                theme = $3,
                default_page = $4,
                collapsed_navigation = $5,
                advanced_rendering = $6,
                native_decorations = $7,

                glass_blur = $8,
                glass_border_opacity = $9,
                background_effect = $10,
                background_effect_intensity = $11,
                page_background_path = $12,
                page_background_opacity = $13,
                proxy_enabled = $14,
                proxy_type = $15,
                proxy_host = $16,
                proxy_port = $17,
                proxy_auth_enabled = $18,
                proxy_username = $19,
                proxy_password = $20,

                discord_rpc = $21,
                developer_mode = $22,
                telemetry = $23,
                personalized_ads = $24,

                onboarded = $25,

                extra_launch_args = json($26),
                custom_env_vars = json($27),
                mc_memory_max = $28,
                mc_force_fullscreen = $29,
                mc_game_resolution_x = $30,
                mc_game_resolution_y = $31,
                hide_on_process_start = $32,

                hook_pre_launch = $33,
                hook_wrapper = $34,
                hook_post_exit = $35,

                custom_dir = $36,
                prev_custom_dir = $37,
                migrated = $38,

                toggle_sidebar = $39,
                feature_flags = $40,
                hide_nametag_skins_page = $41,

                skipped_update = $42,
                pending_update_toast_for_version = $43,
                auto_download_updates = $44,

                version = $45
            ",
            max_concurrent_writes,
            max_concurrent_downloads,
            theme,
            default_page,
            self.collapsed_navigation,
            self.advanced_rendering,
            self.native_decorations,
            self.glass_blur,
            self.glass_border_opacity,
            background_effect,
            self.background_effect_intensity,
            page_background_path,
            self.page_background_opacity,
            self.proxy_enabled,
            proxy_type,
            self.proxy_host,
            self.proxy_port,
            self.proxy_auth_enabled,
            self.proxy_username,
            self.proxy_password,
            self.discord_rpc,
            self.developer_mode,
            self.telemetry,
            self.personalized_ads,
            self.onboarded,
            extra_launch_args,
            custom_env_vars,
            self.memory.maximum,
            self.force_fullscreen,
            self.game_resolution.0,
            self.game_resolution.1,
            self.hide_on_process_start,
            self.hooks.pre_launch,
            self.hooks.wrapper,
            self.hooks.post_exit,
            self.custom_dir,
            self.prev_custom_dir,
            self.migrated,
            self.toggle_sidebar,
            feature_flags,
            self.hide_nametag_skins_page,
            self.skipped_update,
            self.pending_update_toast_for_version,
            self.auto_download_updates,
            version,
        )
        .execute(exec)
        .await?;

        Ok(())
    }

    pub async fn migrate(exec: &Pool<Sqlite>) -> crate::Result<()> {
        let mut settings = Self::get(exec).await?;

        if settings.version < Settings::CURRENT_VERSION {
            tracing::info!(
                "Migrating settings version {} to {:?}",
                settings.version,
                Settings::CURRENT_VERSION
            );
        }
        while settings.version < Settings::CURRENT_VERSION {
            if let Err(err) = settings.perform_migration() {
                tracing::error!(
                    "Failed to migrate settings from version {}: {}",
                    settings.version,
                    err
                );
                return Err(err);
            }
        }

        settings.update(exec).await?;

        Ok(())
    }

    pub fn perform_migration(&mut self) -> crate::Result<()> {
        match self.version {
            1 => {
                let quoter = shlex::Quoter::new().allow_nul(true);

                // Previously split by spaces
                if let Some(pre_launch) = self.hooks.pre_launch.as_ref() {
                    self.hooks.pre_launch =
                        Some(quoter.join(pre_launch.split(' ')).unwrap())
                }

                // Previously treated as complete path to command
                if let Some(wrapper) = self.hooks.wrapper.as_ref() {
                    self.hooks.wrapper =
                        Some(quoter.quote(wrapper).unwrap().to_string())
                }

                // Previously split by spaces
                if let Some(post_exit) = self.hooks.post_exit.as_ref() {
                    self.hooks.post_exit =
                        Some(quoter.join(post_exit.split(' ')).unwrap())
                }

                self.version = 2;
            }
            2 => {
                // Settings schema v3 adds glass/effect settings. Clamp values to safe defaults.
                if self.glass_blur < 0 {
                    self.glass_blur = 0;
                }
                if self.glass_blur > 72 {
                    self.glass_blur = 72;
                }

                if !(0.0..=1.0).contains(&self.glass_border_opacity) {
                    self.glass_border_opacity = 0.075;
                }

                if !matches!(
                    self.background_effect.as_str(),
                    "off" | "snow" | "stars" | "rain"
                ) {
                    self.background_effect = "off".to_string();
                }

                self.version = 3;
            }
            3 => {
                if self.background_effect_intensity < 10 {
                    self.background_effect_intensity = 100;
                }
                if self.background_effect_intensity > 400 {
                    self.background_effect_intensity = 400;
                }

                self.version = 4;
            }
            4 => {
                if self.proxy_port == 0 {
                    self.proxy_port = 8080;
                }

                self.proxy_host = self.proxy_host.trim().to_string();
                if !matches!(
                    self.proxy_type,
                    ProxyType::Http | ProxyType::Https | ProxyType::Socks5
                ) {
                    self.proxy_type = ProxyType::Http;
                }

                if self.proxy_host.is_empty() {
                    self.proxy_enabled = false;
                }

                self.version = 5;
            }
            5 => {
                self.page_background_path =
                    self.page_background_path.trim().to_string();

                if !(0.0..=1.0).contains(&self.page_background_opacity) {
                    self.page_background_opacity = 0.22;
                }

                self.version = 6;
            }
            6 => {
                if self.auto_download_updates.is_none() {
                    self.auto_download_updates = Some(true);
                }

                self.version = 7;
            }
            version => {
                return Err(crate::ErrorKind::OtherError(format!(
                    "Invalid settings version: {version}"
                ))
                .into());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyType {
    Http,
    Https,
    Socks5,
}

impl ProxyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProxyType::Http => "http",
            ProxyType::Https => "https",
            ProxyType::Socks5 => "socks5",
        }
    }

    pub fn from_string(string: &str) -> Self {
        match string {
            "http" => Self::Http,
            "https" => Self::Https,
            "socks5" => Self::Socks5,
            _ => Self::Http,
        }
    }
}

/// Theseus theme
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    Dark,
    Light,
    Oled,
    Retro,
    Sapphire,
    Amethyst,
    Sunset,
    Aurora,
    Nord,
    #[serde(rename = "cherry-cola")]
    CherryCola,
    Slate,
    #[serde(rename = "rose-gold")]
    RoseGold,
    #[serde(rename = "obsidian-gold")]
    ObsidianGold,
    #[serde(rename = "cherry-blossom")]
    CherryBlossom,
    System,
}

impl Theme {
    pub fn as_str(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::Oled => "oled",
            Theme::Retro => "retro",
            Theme::Sapphire => "sapphire",
            Theme::Amethyst => "amethyst",
            Theme::Sunset => "sunset",
            Theme::Aurora => "aurora",
            Theme::Nord => "nord",
            Theme::CherryCola => "cherry-cola",
            Theme::Slate => "slate",
            Theme::RoseGold => "rose-gold",
            Theme::ObsidianGold => "obsidian-gold",
            Theme::CherryBlossom => "cherry-blossom",
            Theme::System => "system",
        }
    }

    pub fn from_string(string: &str) -> Theme {
        match string {
            "dark" => Theme::Dark,
            "light" => Theme::Light,
            "oled" => Theme::Oled,
            "retro" => Theme::Retro,
            "sapphire" => Theme::Sapphire,
            "amethyst" => Theme::Amethyst,
            "sunset" => Theme::Sunset,
            "aurora" => Theme::Aurora,
            "nord" => Theme::Nord,
            "cherry-cola" => Theme::CherryCola,
            "slate" => Theme::Slate,
            "rose-gold" => Theme::RoseGold,
            "obsidian-gold" => Theme::ObsidianGold,
            "cherry-blossom" => Theme::CherryBlossom,
            "system" => Theme::System,
            _ => Theme::Dark,
        }
    }
}

/// Minecraft memory settings
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct MemorySettings {
    pub maximum: u32,
}

/// Game window size
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct WindowSize(pub u16, pub u16);

/// Game initialization hooks
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde_with::serde_as]
pub struct Hooks {
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub pre_launch: Option<String>,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub wrapper: Option<String>,
    #[serde_as(as = "serde_with::NoneAsEmptyString")]
    pub post_exit: Option<String>,
}

/// Opening window to start with
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum DefaultPage {
    Home,
    Library,
}

impl DefaultPage {
    pub fn as_str(&self) -> &'static str {
        match self {
            DefaultPage::Home => "home",
            DefaultPage::Library => "library",
        }
    }

    pub fn from_string(string: &str) -> Self {
        match string {
            "home" => Self::Home,
            "library" => Self::Library,
            _ => Self::Home,
        }
    }
}
