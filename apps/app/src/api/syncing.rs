use crate::api::Result;
use same_file::is_same_file;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use theseus::prelude::*;

const DEFAULT_FOLDER_TARGETS: &[&str] = &[
    "resourcepacks",
    "shaderpacks",
    "screenshots",
    "config",
    "saves",
    "schematics",
    "xaero",
    ".bobby",
    ".voxy",
];

pub fn init<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("syncing")
        .invoke_handler(tauri::generate_handler![
            syncing_get_state,
            syncing_set_target,
            syncing_remove_target,
            syncing_apply_all,
        ])
        .build()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SyncConfig {
    folders: Vec<String>,
    files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncTargetState {
    pub path: String,
    pub is_file: bool,
    pub enabled: bool,
    pub default_target: bool,
    pub sync_count: usize,
    pub cannot_sync_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    pub sync_folder: PathBuf,
    pub total_count: usize,
    pub folders: Vec<SyncTargetState>,
    pub files: Vec<SyncTargetState>,
}

#[tauri::command]
pub async fn syncing_get_state() -> Result<SyncState> {
    build_sync_state().await
}

#[tauri::command]
pub async fn syncing_set_target(
    path: &str,
    is_file: bool,
    enabled: bool,
) -> Result<SyncState> {
    let normalized = normalize_target(path).ok_or_else(|| {
        theseus::Error::from(theseus::ErrorKind::InputError(format!(
            "Invalid sync target path: {path}"
        )))
    })?;

    let state = State::get().await?;
    let config_path = config_path(&state.directories);
    let mut config = read_config(&config_path)?;
    let sync_root = sync_root(&state.directories);
    let instances = collect_instance_paths().await?;
    let targets = if is_file {
        &mut config.files
    } else {
        &mut config.folders
    };

    if enabled {
        if !targets.iter().any(|target| target == &normalized) {
            targets.push(normalized.clone());
            targets.sort();
        }
        if !is_file {
            enable_folder_target(&normalized, &instances, &sync_root)?;
        }
    } else {
        if is_file {
            disable_file_target(&normalized, &instances, &sync_root)?;
        } else {
            disable_folder_target(&normalized, &instances, &sync_root)?;
        }
        targets.retain(|target| target != &normalized);
    }

    write_config(&config_path, &config)?;
    build_sync_state().await
}

#[tauri::command]
pub async fn syncing_remove_target(path: &str, is_file: bool) -> Result<SyncState> {
    let normalized = normalize_target(path).ok_or_else(|| {
        theseus::Error::from(theseus::ErrorKind::InputError(format!(
            "Invalid sync target path: {path}"
        )))
    })?;

    let state = State::get().await?;
    let config_path = config_path(&state.directories);
    let mut config = read_config(&config_path)?;
    let sync_root = sync_root(&state.directories);
    let instances = collect_instance_paths().await?;
    let targets = if is_file {
        &mut config.files
    } else {
        &mut config.folders
    };
    if is_file {
        disable_file_target(&normalized, &instances, &sync_root)?;
    } else {
        disable_folder_target(&normalized, &instances, &sync_root)?;
    }
    targets.retain(|target| target != &normalized);
    write_config(&config_path, &config)?;
    build_sync_state().await
}

#[tauri::command]
pub async fn syncing_apply_all() -> Result<SyncState> {
    apply_configured_syncs().await?;
    build_sync_state().await
}

pub async fn apply_syncing_to_profile(path: &str) -> Result<()> {
    let state = State::get().await?;
    let config = read_config(&config_path(&state.directories))?;
    let sync_root = sync_root(&state.directories);
    let instances = collect_instance_paths().await?;
    let instance_path = theseus::profile::get_full_path(path).await?;

    fs::create_dir_all(&sync_root)?;
    apply_config_to_instance(&config, &instances, &sync_root, &instance_path)
        .map_err(Into::into)
}

pub async fn apply_configured_syncs() -> Result<()> {
    let state = State::get().await?;
    let config = read_config(&config_path(&state.directories))?;
    let sync_root = sync_root(&state.directories);
    let instances = collect_instance_paths().await?;

    if instances.is_empty() {
        fs::create_dir_all(&sync_root)?;
        return Ok(());
    }

    fs::create_dir_all(&sync_root)?;

    for folder in &config.folders {
        enable_folder_target(folder, &instances, &sync_root)?;
    }

    for file in &config.files {
        for instance in &instances {
            apply_file_target_to_instance(file, &instances, &sync_root, instance)?;
        }
    }

    Ok(())
}

fn apply_config_to_instance(
    config: &SyncConfig,
    instances: &[PathBuf],
    sync_root: &Path,
    instance: &Path,
) -> io::Result<()> {
    for folder in &config.folders {
        apply_folder_target_to_instance(folder, sync_root, instance)?;
    }

    for file in &config.files {
        apply_file_target_to_instance(file, instances, sync_root, instance)?;
    }

    Ok(())
}

async fn build_sync_state() -> Result<SyncState> {
    let state = State::get().await?;
    let config = read_config(&config_path(&state.directories))?;
    let sync_root = sync_root(&state.directories);
    let instances = collect_instance_paths().await?;
    fs::create_dir_all(&sync_root)?;

    let folders = build_folder_states(&config, &instances, &sync_root);
    let files = build_file_states(&config, &instances, &sync_root);

    Ok(SyncState {
        sync_folder: sync_root,
        total_count: instances.len(),
        folders,
        files,
    })
}

fn build_folder_states(
    config: &SyncConfig,
    instances: &[PathBuf],
    sync_root: &Path,
) -> Vec<SyncTargetState> {
    let mut ordered_targets = Vec::new();
    let mut seen = BTreeSet::new();

    for target in DEFAULT_FOLDER_TARGETS {
        let value = (*target).to_string();
        if seen.insert(value.clone()) {
            ordered_targets.push((value, true));
        }
    }

    for target in &config.folders {
        if seen.insert(target.clone()) {
            ordered_targets.push((target.clone(), false));
        }
    }

    ordered_targets
        .into_iter()
        .map(|(target, default_target)| {
            let shared = sync_target_path(sync_root, &target);
            let mut sync_count = 0;
            let mut cannot_sync_count = 0;

            for instance in instances {
                let path = sync_target_path(instance, &target);
                if is_dir_link_targeting(&shared, &path) {
                    sync_count += 1;
                } else if path.exists() {
                    cannot_sync_count += 1;
                }
            }

            SyncTargetState {
                path: target.clone(),
                is_file: false,
                enabled: config.folders.iter().any(|value| value == &target),
                default_target,
                sync_count,
                cannot_sync_count,
            }
        })
        .collect()
}

fn build_file_states(
    config: &SyncConfig,
    instances: &[PathBuf],
    sync_root: &Path,
) -> Vec<SyncTargetState> {
    config
        .files
        .iter()
        .map(|target| {
            let shared = sync_target_path(sync_root, target);
            let mut sync_count = 0;
            let mut cannot_sync_count = 0;

            for instance in instances {
                let path = sync_target_path(instance, target);
                if shared.exists() && path.exists() && is_same_file(&shared, &path).unwrap_or(false)
                {
                    sync_count += 1;
                } else if path.exists() {
                    cannot_sync_count += 1;
                }
            }

            SyncTargetState {
                path: target.clone(),
                is_file: true,
                enabled: true,
                default_target: false,
                sync_count,
                cannot_sync_count,
            }
        })
        .collect()
}

async fn collect_instance_paths() -> Result<Vec<PathBuf>> {
    let profiles = theseus::profile::list().await?;
    let mut paths = Vec::with_capacity(profiles.len());

    for profile in profiles {
        paths.push(theseus::profile::get_full_path(&profile.path).await?);
    }

    Ok(paths)
}

fn config_path(directories: &DirectoryInfo) -> PathBuf {
    directories.settings_dir.join("syncing.json")
}

fn sync_root(directories: &DirectoryInfo) -> PathBuf {
    directories.config_dir.join("synced")
}

fn read_config(path: &Path) -> io::Result<SyncConfig> {
    if !path.exists() {
        return Ok(SyncConfig::default());
    }

    let bytes = fs::read(path)?;
    Ok(serde_json::from_slice(&bytes).unwrap_or_default())
}

fn write_config(path: &Path, config: &SyncConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_vec_pretty(config)
        .map_err(io::Error::other)?;
    fs::write(path, json)
}

fn normalize_target(input: &str) -> Option<String> {
    let trimmed = input.trim().replace('\\', "/");
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with('/') || trimmed.starts_with("./") {
        return None;
    }
    if trimmed.len() > 1 && trimmed.as_bytes()[1] == b':' {
        return None;
    }

    let mut parts = Vec::new();
    for component in Path::new(&trimmed).components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            Component::CurDir => {}
            _ => return None,
        }
    }

    if parts.is_empty() || parts.iter().any(|part| part == "..") {
        return None;
    }

    Some(parts.join("/"))
}

fn sync_target_path(base: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(base.to_path_buf(), |path, part| path.join(part))
}

fn enable_folder_target(
    target: &str,
    instances: &[PathBuf],
    sync_root: &Path,
) -> io::Result<()> {
    let shared = sync_target_path(sync_root, target);
    fs::create_dir_all(&shared)?;

    for instance in instances {
        let instance_target = sync_target_path(instance, target);

        if is_dir_link_targeting(&shared, &instance_target) {
            continue;
        }

        if instance_target.exists() {
            if instance_target.is_file() {
                continue;
            }
            merge_dir_into(&instance_target, &shared)?;
            remove_existing_path(&instance_target)?;
        } else if let Some(parent) = instance_target.parent() {
            fs::create_dir_all(parent)?;
        }

        create_dir_link(&shared, &instance_target)?;
    }

    Ok(())
}

fn apply_folder_target_to_instance(
    target: &str,
    sync_root: &Path,
    instance: &Path,
) -> io::Result<()> {
    let shared = sync_target_path(sync_root, target);
    fs::create_dir_all(&shared)?;

    let instance_target = sync_target_path(instance, target);
    if is_dir_link_targeting(&shared, &instance_target) || instance_target.exists()
    {
        return Ok(());
    }

    if let Some(parent) = instance_target.parent() {
        fs::create_dir_all(parent)?;
    }

    create_dir_link(&shared, &instance_target)
}

fn apply_file_target_to_instance(
    target: &str,
    instances: &[PathBuf],
    sync_root: &Path,
    instance: &Path,
) -> io::Result<()> {
    let shared = sync_target_path(sync_root, target);
    if let Some(parent) = shared.parent() {
        fs::create_dir_all(parent)?;
    }

    if !shared.exists() {
        if let Some(seed) = pick_latest_file_candidate(target, instances) {
            copy_file(&seed, &shared)?;
        } else {
            return Ok(());
        }
    }

    let instance_target = sync_target_path(instance, target);
    if instance_target.exists() {
        if instance_target.is_dir() {
            return Ok(());
        }
        if is_same_file(&shared, &instance_target).unwrap_or(false) {
            remove_existing_path(&instance_target)?;
            return copy_file(&shared, &instance_target);
        }
        remove_existing_path(&instance_target)?;
    } else if let Some(parent) = instance_target.parent() {
        fs::create_dir_all(parent)?;
    }

    copy_file(&shared, &instance_target)
}

fn disable_folder_target(
    target: &str,
    instances: &[PathBuf],
    sync_root: &Path,
) -> io::Result<()> {
    let shared = sync_target_path(sync_root, target);
    if !shared.exists() {
        return Ok(());
    }

    for instance in instances {
        let instance_target = sync_target_path(instance, target);
        if is_dir_link_targeting(&shared, &instance_target) {
            remove_dir_link(&instance_target)?;
            copy_dir_all(&shared, &instance_target)?;
        }
    }

    Ok(())
}

fn disable_file_target(
    target: &str,
    instances: &[PathBuf],
    sync_root: &Path,
) -> io::Result<()> {
    let shared = sync_target_path(sync_root, target);
    if !shared.exists() {
        return Ok(());
    }

    for instance in instances {
        let instance_target = sync_target_path(instance, target);
        if instance_target.exists() && is_same_file(&shared, &instance_target).unwrap_or(false) {
            remove_existing_path(&instance_target)?;
            copy_file(&shared, &instance_target)?;
        }
    }

    Ok(())
}

fn pick_latest_file_candidate(target: &str, instances: &[PathBuf]) -> Option<PathBuf> {
    instances
        .iter()
        .map(|instance| sync_target_path(instance, target))
        .filter(|path| path.is_file())
        .max_by_key(|path| {
            fs::metadata(path)
                .and_then(|metadata| metadata.modified())
                .ok()
        })
}

fn merge_dir_into(source: &Path, target: &Path) -> io::Result<()> {
    if is_dir_link(source) {
        return Ok(());
    }

    fs::create_dir_all(target)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)?;

        if metadata.is_dir() {
            merge_dir_into(&source_path, &target_path)?;
            remove_empty_dir(&source_path)?;
            continue;
        }

        if target_path.exists() {
            let source_modified = metadata.modified().ok();
            let target_modified = fs::metadata(&target_path).and_then(|meta| meta.modified()).ok();
            if source_modified > target_modified {
                remove_existing_path(&target_path)?;
                copy_file(&source_path, &target_path)?;
            }
            remove_existing_path(&source_path)?;
        } else {
            copy_file(&source_path, &target_path)?;
            remove_existing_path(&source_path)?;
        }
    }

    remove_empty_dir(source)
}

fn copy_dir_all(source: &Path, target: &Path) -> io::Result<()> {
    fs::create_dir_all(target)?;

    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)?;

        if metadata.is_dir() {
            copy_dir_all(&source_path, &target_path)?;
        } else {
            copy_file(&source_path, &target_path)?;
        }
    }

    Ok(())
}

fn copy_file(source: &Path, target: &Path) -> io::Result<()> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, target)?;
    Ok(())
}

fn remove_empty_dir(path: &Path) -> io::Result<()> {
    match fs::remove_dir(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::DirectoryNotEmpty => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err),
    }
}

fn remove_existing_path(path: &Path) -> io::Result<()> {
    if !path.exists() && !is_dir_link(path) {
        return Ok(());
    }

    if path.is_file() {
        fs::remove_file(path)
    } else if is_dir_link(path) {
        remove_dir_link(path)
    } else {
        fs::remove_dir_all(path)
    }
}

#[cfg(unix)]
fn create_dir_link(original: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(original, link)
}

#[cfg(windows)]
fn create_dir_link(original: &Path, link: &Path) -> io::Result<()> {
    junction::create(original, link)
}

#[cfg(unix)]
fn remove_dir_link(link: &Path) -> io::Result<()> {
    fs::remove_file(link)
}

#[cfg(windows)]
fn remove_dir_link(link: &Path) -> io::Result<()> {
    junction::delete(link)
}

#[cfg(unix)]
fn is_dir_link(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_dir_link(path: &Path) -> bool {
    junction::get_target(path).is_ok()
}

#[cfg(unix)]
fn is_dir_link_targeting(original: &Path, link: &Path) -> bool {
    match fs::read_link(link) {
        Ok(target) => target == original,
        Err(_) => false,
    }
}

#[cfg(windows)]
fn is_dir_link_targeting(original: &Path, link: &Path) -> bool {
    match junction::get_target(link) {
        Ok(target) => target == original,
        Err(_) => false,
    }
}
