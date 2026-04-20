// [AR] Feature
use std::{
    sync::{Arc, OnceLock, RwLock as StdRwLock, atomic::AtomicBool},
    time::{SystemTime, UNIX_EPOCH},
};

use discord_rich_presence::{
    DiscordIpc,
    DiscordIpcClient,
    activity::{Activity, Assets, Timestamps}, // [AR] Feature
};
use rand::seq::SliceRandom; // [AR] Feature
use tokio::sync::RwLock;

use crate::State;
use crate::util::utils; // [AR] Feature

pub struct DiscordGuard {
    client: Arc<RwLock<DiscordIpcClient>>,
    connected: Arc<AtomicBool>,
}

static RPC_LANGUAGE: OnceLock<StdRwLock<String>> = OnceLock::new();

fn rpc_language_store() -> &'static StdRwLock<String> {
    RPC_LANGUAGE.get_or_init(|| StdRwLock::new(String::from("en")))
}

pub fn set_rpc_language(language: &str) {
    if let Ok(mut current) = rpc_language_store().write() {
        *current = language.to_string();
    }
}

fn get_rpc_language() -> String {
    rpc_language_store()
        .read()
        .map(|value| value.clone())
        .unwrap_or_else(|_| "en".to_string())
}

fn active_state(language: &str) -> [&'static str; 6] {
    match language {
        "ru" => [
            "Исследует",
            "Путешествует с",
            "Пиратит с",
            "Изучает",
            "Занят вместе с",
            "Запускает с",
        ],
        "uk" => [
            "Досліджує",
            "Подорожує з",
            "Піратствує з",
            "Вивчає",
            "Зайнятий разом із",
            "Запускає з",
        ],
        "de" => [
            "Erkundet",
            "Reist mit",
            "Piratet mit",
            "Untersucht",
            "Beschäftigt sich mit",
            "Startet mit",
        ],
        "ro" => [
            "Exploreaza",
            "Calatoreste cu",
            "Pirateaza cu",
            "Investigheaza",
            "Este ocupat cu",
            "Porneste cu",
        ],
        _ => [
            "Explores",
            "Travels with",
            "Pirating",
            "Investigating",
            "Engaged with",
            "Launching with",
        ],
    }
}

fn inactive_state(language: &str) -> [&'static str; 6] {
    match language {
        "ru" => [
            "Бездельничает...",
            "Ждёт пиратскую команду...",
            "Делает перерыв...",
            "Отдыхает...",
            "Наготове...",
            "Завис в ожидании...",
        ],
        "uk" => [
            "Нудьгує...",
            "Чекає піратську команду...",
            "Робить перерву...",
            "Відпочиває...",
            "Напоготові...",
            "Завмер у режимі очікування...",
        ],
        "de" => [
            "Leerlauf...",
            "Wartet auf das Piratenteam...",
            "Macht eine Pause...",
            "Ruht sich aus...",
            "In Bereitschaft...",
            "Im Wartemodus...",
        ],
        "ro" => [
            "Sta degeaba...",
            "Asteapta echipa de pirati...",
            "Ia o pauza...",
            "Se odihneste...",
            "In asteptare...",
            "Ramane in standby...",
        ],
        _ => [
            "Idling...",
            "Waiting for the pirate team...",
            "Taking a break...",
            "Resting...",
            "On standby...",
            "In a holding pattern...",
        ],
    }
}

pub fn select_active_phrase() -> &'static str {
    let language = get_rpc_language();
    let phrases = active_state(&language);
    phrases
        .choose(&mut rand::thread_rng())
        .copied()
        .unwrap_or("Playing")
}

pub fn select_inactive_phrase() -> &'static str {
    let language = get_rpc_language();
    let phrases = inactive_state(&language);
    phrases
        .choose(&mut rand::thread_rng())
        .copied()
        .unwrap_or("Idling...")
}

impl DiscordGuard {
    /// Initialize discord IPC client, and attempt to connect to it
    /// If it fails, it will still return a DiscordGuard, but the client will be unconnected
    pub fn init() -> crate::Result<DiscordGuard> {
        let dipc = DiscordIpcClient::new("1474801363199201441");

        Ok(DiscordGuard {
            client: Arc::new(RwLock::new(dipc)),
            connected: Arc::new(AtomicBool::new(false)),
        })
    }

    /// If the client failed connecting during init(), this will check for connection and attempt to reconnect
    /// This MUST be called first in any client method that requires a connection, because those can PANIC if the client is not connected
    /// (No connection is different than a failed connection, the latter will not panic and can be retried)
    pub async fn retry_if_not_ready(&self) -> bool {
        let mut client = self.client.write().await;
        if !self.connected.load(std::sync::atomic::Ordering::Relaxed) {
            if client.connect().is_ok() {
                self.connected
                    .store(true, std::sync::atomic::Ordering::Relaxed);
                return true;
            }
            return false;
        }
        true
    }

    /// Set the activity to the given message
    /// First checks if discord is disabled, and if so, clear the activity instead
    pub async fn set_activity(
        &self,
        msg: &str,
        reconnect_if_fail: bool,
    ) -> crate::Result<()> {
        // Check if discord is disabled, and if so, clear the activity instead
        let state = State::get().await?;
        let settings = crate::state::Settings::get(&state.pool).await?;
        if !settings.discord_rpc {
            Ok(self.clear_activity(true).await?)
        } else {
            Ok(self.force_set_activity(msg, reconnect_if_fail).await?)
        }
    }

    /// Sets the activity to the given message, regardless of if discord is disabled or offline
    /// Should not be used except for in the above method, or if it is already known that discord is enabled (specifically for state initialization) and we are connected to the internet
    pub async fn force_set_activity(
        &self,
        msg: &str,
        reconnect_if_fail: bool,
    ) -> crate::Result<()> {
        // Attempt to connect if not connected. Do not continue if it fails, as the client.set_activity can panic if it never was connected
        if !self.retry_if_not_ready().await {
            return Ok(());
        }

        // let activity = Activity::new().state(msg).assets(
        //     Assets::new()
        //         .large_image("modrinth_simple")
        //         .large_text("Modrinth Logo"),
        // );

        let launcher =
            utils::read_package_json().expect("Failed to read package.json");

        let build_info = format!("Revoria • v{}", launcher.version);
        let build_download = "https://github.com/imsawiq/Revoria/releases";

        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get system time")
            .as_secs() as i64;
        let activity = Activity::new()
            .state(msg)
            .assets(
                Assets::new()
                    .large_image("revoria_logo")
                    .large_text(&build_info)
                    .small_image("revoria_logo")
                    .small_text(&build_download),
            )
            .timestamps(Timestamps::new().start(time));

        // Attempt to set the activity
        // If the existing connection fails, attempt to reconnect and try again
        let mut client: tokio::sync::RwLockWriteGuard<'_, DiscordIpcClient> =
            self.client.write().await;
        let res = client.set_activity(activity.clone());

        if reconnect_if_fail {
            if let Err(_e) = res {
                client.reconnect()?;
                return Ok(client.set_activity(activity)?); // try again, but don't reconnect if it fails again
            }
        } else {
            res?;
        }

        Ok(())
    }

    /// Clear the activity entirely ('disabling' the RPC until the next set_activity)
    pub async fn clear_activity(
        &self,
        reconnect_if_fail: bool,
    ) -> crate::Result<()> {
        // Attempt to connect if not connected. Do not continue if it fails, as the client.clear_activity can panic if it never was connected
        if !self.retry_if_not_ready().await {
            return Ok(());
        }

        // Attempt to clear the activity
        // If the existing connection fails, attempt to reconnect and try again
        let mut client = self.client.write().await;
        let res = client.clear_activity();

        if reconnect_if_fail {
            if res.is_err() {
                client.reconnect()?;
                return Ok(client.clear_activity()?); // try again, but don't reconnect if it fails again
            }
        } else {
            res?;
        }
        Ok(())
    }

    /// Clear the activity, but if there is a running profile, set the activity to that instead
    pub async fn clear_to_default(
        &self,
        reconnect_if_fail: bool,
    ) -> crate::Result<()> {
        let state = State::get().await?;

        let settings = crate::state::Settings::get(&state.pool).await?;
        if !settings.discord_rpc {
            println!("Discord is disabled, clearing activity");
            return self.clear_activity(true).await;
        }

        let selected_phrase = select_inactive_phrase();
        self.set_activity(&format!("{}", selected_phrase), reconnect_if_fail)
            .await?;
        Ok(())
    }
}
