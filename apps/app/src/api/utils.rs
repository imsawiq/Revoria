use serde::{Deserialize, Serialize};
use tauri::Runtime;
use tauri_plugin_opener::OpenerExt;
use theseus::{
    LoadingBarType, emit_loading, handler, init_loading,
    prelude::{CommandPayload, DirectoryInfo},
};

use crate::api::{Result, TheseusSerializableError};
use dashmap::DashMap;
use std::path::{Path, PathBuf};
use tauri_plugin_http::reqwest;
use tauri_plugin_http::reqwest::Client;
use theseus::prelude::canonicalize;
use theseus::util::utils;
use tokio::io::AsyncWriteExt;
use tokio::fs;
use url::Url;

pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("utils")
        .invoke_handler(tauri::generate_handler![
            init_authlib_patching,
            apply_migration_fix,
            init_update_launcher,
            get_os,
            is_network_metered,
            should_disable_mouseover,
            download_url_to_temp,
            highlight_in_folder,
            open_path,
            show_launcher_logs_folder,
            progress_bars_list,
            get_opening_command,
            list_dir_files,
            get_dir_size,
            delete_paths,
            read_text_file,
            ollama_chat
        ])
        .build()
}

#[tauri::command]
pub async fn download_url_to_temp(
    url: String,
    filename: Option<String>,
    profile_path: Option<String>,
) -> Result<String> {
    let file_name = filename.unwrap_or_else(|| "download.bin".to_string());
    let loading_bar = init_loading(
        LoadingBarType::PackFileDownload {
            profile_path: profile_path.unwrap_or_default(),
            pack_name: file_name.clone(),
            icon: None,
            pack_version: String::new(),
        },
        100.0,
        "Downloading file",
    )
    .await?;
    let response = reqwest::get(&url).await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;

    if !response.status().is_success() {
        return Err(theseus::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Download failed with status: {}", response.status()),
        ))
        .into());
    }

    let mut bytes = Vec::new();
    let mut last_progress = 0.0;
    if let Some(total_size) = response.content_length() {
        use futures_util::StreamExt;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;
        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?;
            downloaded = downloaded.saturating_add(chunk.len() as u64);
            bytes.extend_from_slice(&chunk);
            let progress =
                ((downloaded as f64 / total_size as f64) * 100.0).min(100.0);
            let increment = (progress - last_progress).max(0.0);
            if increment > 0.0 {
                let _ = emit_loading(&loading_bar, increment, None);
                last_progress = progress;
            }
        }
    } else {
        let body = response.bytes().await.map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
        })?;
        bytes.extend_from_slice(&body);
        let _ = emit_loading(&loading_bar, 100.0, None);
    }
    let _ = emit_loading(&loading_bar, 0.0, Some("Downloaded file"));

    let mut out_path = std::env::temp_dir();
    out_path.push("revoria");
    tokio::fs::create_dir_all(&out_path).await?;
    out_path.push(file_name);

    let mut file = tokio::fs::File::create(&out_path).await?;
    file.write_all(&bytes).await?;
    file.flush().await?;

    Ok(out_path.to_string_lossy().to_string())
}

/// [AR] Feature. Ely.by
#[tauri::command]
pub async fn init_authlib_patching(
    minecraft_version: &str,
    is_mojang: bool,
) -> Result<bool> {
    let result =
        utils::init_authlib_patching(minecraft_version, is_mojang).await?;
    Ok(result)
}

/// [AR] Migration. Patch
#[tauri::command]
pub async fn apply_migration_fix(eol: &str) -> Result<bool> {
    let result = utils::apply_migration_fix(eol).await?;
    Ok(result)
}

/// [AR] Feature. Updater
#[tauri::command]
pub async fn init_update_launcher(
    download_url: &str,
    filename: &str,
    os_type: &str,
    auto_update_supported: bool,
) -> Result<()> {
    let _ = utils::init_update_launcher(
        download_url,
        filename,
        os_type,
        auto_update_supported,
    )
    .await;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum OS {
    Windows,
    Linux,
    MacOS,
}

/// Gets OS
#[tauri::command]
pub fn get_os() -> OS {
    #[cfg(target_os = "windows")]
    let os = OS::Windows;
    #[cfg(target_os = "linux")]
    let os = OS::Linux;
    #[cfg(target_os = "macos")]
    let os = OS::MacOS;
    os
}

#[tauri::command]
pub async fn is_network_metered() -> Result<bool> {
    Ok(theseus::prelude::is_network_metered().await?)
}

// Lists active progress bars
// Create a new HashMap with the same keys
// Values provided should not be used directly, as they are not guaranteed to be up-to-date
#[tauri::command]
pub async fn progress_bars_list()
-> Result<DashMap<uuid::Uuid, theseus::LoadingBar>> {
    let res = theseus::EventState::list_progress_bars().await?;
    Ok(res)
}

// disables mouseover and fixes a random crash error only fixed by recent versions of macos
#[tauri::command]
pub async fn should_disable_mouseover() -> bool {
    if cfg!(target_os = "macos") {
        // We try to match version to 12.2 or higher. If unrecognizable to pattern or lower, we default to the css with disabled mouseover for safety
        if let tauri_plugin_os::Version::Semantic(major, minor, _) =
            tauri_plugin_os::version()
            && major >= 12
            && minor >= 3
        {
            // Mac os version is 12.3 or higher, we allow mouseover
            return false;
        }
        true
    } else {
        // Not macos, we allow mouseover
        false
    }
}

#[tauri::command]
pub fn highlight_in_folder<R: Runtime>(
    app: tauri::AppHandle<R>,
    path: PathBuf,
) {
    if let Err(e) = app.opener().reveal_item_in_dir(path) {
        eprintln!("Failed to highlight file in folder: {}", e);
    }
}

#[tauri::command]
pub fn open_path<R: Runtime>(app: tauri::AppHandle<R>, path: PathBuf) {
    if let Err(e) = app.opener().open_path(path.to_string_lossy(), None::<&str>)
    {
        eprintln!("Failed to open path: {}", e);
    }
}

#[tauri::command]
pub fn show_launcher_logs_folder<R: Runtime>(app: tauri::AppHandle<R>) {
    let path = DirectoryInfo::launcher_logs_dir().unwrap_or_default();
    // failure to get folder just opens filesystem
    // (ie: if in debug mode only and launcher_logs never created)
    open_path(app, path);
}

// Get opening command
// For example, if a user clicks on an .mrpack to open the app.
// This should be called once and only when the app is done booting up and ready to receive a command
// Returns a Command struct- see events.js
#[tauri::command]
#[cfg(target_os = "macos")]
pub async fn get_opening_command(
    state: tauri::State<'_, crate::macos::deep_link::InitialPayload>,
) -> Result<Option<CommandPayload>> {
    let payload = state.payload.lock().await;

    return if let Some(payload) = payload.as_ref() {
        println!("opening command {payload}");

        Ok(Some(handler::parse_command(payload).await?))
    } else {
        Ok(None)
    };
}

#[tauri::command]
#[cfg(not(target_os = "macos"))]
pub async fn get_opening_command() -> Result<Option<CommandPayload>> {
    // Tauri is not CLI, we use arguments as path to file to call
    let cmd_arg = std::env::args_os().nth(1);

    println!("opening command {cmd_arg:?}");

    let cmd_arg = cmd_arg.map(|path| path.to_string_lossy().to_string());
    if let Some(cmd) = cmd_arg {
        println!("Opening command: {:?}", cmd);
        return Ok(Some(handler::parse_command(&cmd).await?));
    }
    Ok(None)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirFileEntry {
    pub name: String,
    pub path: String,
    pub size: u64,
}

#[tauri::command]
pub async fn list_dir_files(path: String, extensions: Option<Vec<String>>) -> Result<Vec<DirFileEntry>> {
    let dir = PathBuf::from(&path);
    if !dir.exists() || !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut entries = Vec::new();
    let mut read_dir = tokio::fs::read_dir(&dir).await?;
    while let Some(entry) = read_dir.next_entry().await? {
        let metadata = entry.metadata().await?;
        if !metadata.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if let Some(ref exts) = extensions {
            let lower = name.to_lowercase();
            if !exts.iter().any(|ext| lower.ends_with(ext)) {
                continue;
            }
        }
        entries.push(DirFileEntry {
            name,
            path: entry.path().to_string_lossy().to_string(),
            size: metadata.len(),
        });
    }
    entries.sort_by(|a, b| b.name.cmp(&a.name));
    Ok(entries)
}

#[tauri::command]
pub async fn get_dir_size(path: String) -> Result<u64> {
    let dir = PathBuf::from(&path);
    if !dir.exists() {
        return Ok(0);
    }
    path_size(dir).await
}

async fn path_size(path: PathBuf) -> Result<u64> {
    let size = tokio::task::spawn_blocking(move || {
        fn walk(path: &Path) -> std::io::Result<u64> {
            let metadata = std::fs::metadata(path)?;
            if metadata.is_file() {
                return Ok(metadata.len());
            }

            let mut total = 0u64;
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                total = total.saturating_add(walk(&entry.path())?);
            }
            Ok(total)
        }

        walk(&path)
    })
    .await
    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))??;
    Ok(size)
}

#[tauri::command]
pub async fn delete_paths(paths: Vec<String>) -> Result<u64> {
    let mut deleted = 0u64;
    for raw in paths {
        let path = PathBuf::from(&raw);
        if !path.exists() {
            continue;
        }
        let size = path_size(path.clone()).await.unwrap_or(0);
        if path.is_dir() {
            let _ = fs::remove_dir_all(&path).await;
        } else {
            let _ = fs::remove_file(&path).await;
        }
        deleted = deleted.saturating_add(size);
    }
    Ok(deleted)
}

#[tauri::command]
pub async fn read_text_file(path: String) -> Result<String> {
    let content = fs::read_to_string(path).await?;
    Ok(content)
}

#[tauri::command]
pub async fn ollama_chat(
    endpoint: String,
    model: String,
    api_key: Option<String>,
    prompt: String,
) -> Result<String> {
    let client = Client::new();
    let mut request = client
        .post(&endpoint)
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                { "role": "system", "content": "You analyze Minecraft crash logs." },
                { "role": "user", "content": prompt }
            ],
            "stream": false
        }));

    if let Some(token) = api_key.filter(|key| !key.is_empty()) {
        request = request.bearer_auth(token);
    }

    let response = request.send().await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;

    let status = response.status();
    let body = response.text().await.map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
    })?;

    if !status.is_success() {
        return Err(theseus::Error::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Ollama request failed with status {status}: {body}"),
        ))
        .into());
    }

    Ok(body)
}

// helper function called when redirected by a weblink (ie: modrith://do-something) or when redirected by a .mrpack file (in which case its a filepath)
// We hijack the deep link library (which also contains functionality for instance-checking)
pub async fn handle_command(command: String) -> Result<()> {
    println!("handle command: {command}");
    Ok(theseus::handler::parse_and_emit_command(&command).await?)
}

// Remove when (and if) https://github.com/tauri-apps/tauri/issues/12022 is implemented
pub(crate) fn tauri_convert_file_src(path: &Path) -> Result<Url> {
    #[cfg(any(windows, target_os = "android"))]
    const BASE: &str = "http://asset.localhost/";
    #[cfg(not(any(windows, target_os = "android")))]
    const BASE: &str = "asset://localhost/";

    macro_rules! theseus_try {
        ($test:expr) => {
            match $test {
                Ok(val) => val,
                Err(e) => {
                    return Err(TheseusSerializableError::Theseus(e.into()))
                }
            }
        };
    }

    let path = theseus_try!(canonicalize(path));
    let path = path.to_string_lossy();
    let encoded = urlencoding::encode(&path);

    Ok(theseus_try!(Url::parse(&format!("{BASE}{encoded}"))))
}
