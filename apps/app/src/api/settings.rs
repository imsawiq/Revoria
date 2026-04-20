use crate::api::Result;
use serde::Serialize;
use theseus::prelude::*;

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("settings")
        .invoke_handler(tauri::generate_handler![
            settings_get,
            settings_set,
            settings_test_proxy,
            cancel_directory_change
        ])
        .build()
}

#[derive(Serialize)]
pub struct ProxyTestResult {
    pub ok: bool,
    pub message: String,
    pub ip: Option<String>,
    pub minecraft_status: Option<u16>,
    pub xbox_status: Option<u16>,
}

// Get full settings
// invoke('plugin:settings|settings_get')
#[tauri::command]
pub async fn settings_get() -> Result<Settings> {
    let res = settings::get().await?;
    Ok(res)
}

// Set full settings
// invoke('plugin:settings|settings_set', settings)
#[tauri::command]
pub async fn settings_set(settings: Settings) -> Result<()> {
    settings::set(settings).await?;
    Ok(())
}

#[tauri::command]
pub async fn settings_test_proxy(
    settings: Settings,
) -> Result<ProxyTestResult> {
    #[derive(serde::Deserialize)]
    struct IpResponse {
        ip: String,
    }

    let client =
        theseus::util::fetch::build_reqwest_client_from_settings(&settings)?;

    let ip = client
        .get("https://api.ipify.org?format=json")
        .send()
        .await
        .map_err(|err| {
            theseus::ErrorKind::OtherError(format!(
                "Proxy test failed while resolving external IP: {err}"
            ))
            .as_error()
        })?
        .error_for_status()
        .map_err(|err| {
            theseus::ErrorKind::OtherError(format!(
                "Proxy test failed while resolving external IP: {err}"
            ))
            .as_error()
        })?
        .json::<IpResponse>()
        .await
        .map_err(|err| {
            theseus::ErrorKind::OtherError(format!(
                "Proxy test failed while parsing external IP response: {err}"
            ))
            .as_error()
        })?
        .ip;

    let minecraft_status = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .send()
        .await
        .map_err(|err| {
            theseus::ErrorKind::OtherError(format!(
                "Proxy test failed while contacting Minecraft services: {err}"
            ))
            .as_error()
        })?
        .status()
        .as_u16();

    let xbox_status = client
        .post("https://xsts.auth.xboxlive.com/xsts/authorize")
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .body("{}")
        .send()
        .await
        .map_err(|err| {
            theseus::ErrorKind::OtherError(format!(
                "Proxy test failed while contacting Xbox device auth: {err}"
            ))
            .as_error()
        })?
        .status()
        .as_u16();

    Ok(ProxyTestResult {
        ok: true,
        message: format!(
            "Proxy reachable. External IP: {ip}. Minecraft endpoint responded with {minecraft_status}, Xbox endpoint responded with {xbox_status}."
        ),
        ip: Some(ip),
        minecraft_status: Some(minecraft_status),
        xbox_status: Some(xbox_status),
    })
}

#[tauri::command]
pub async fn cancel_directory_change() -> Result<()> {
    settings::cancel_directory_change().await?;
    Ok(())
}
