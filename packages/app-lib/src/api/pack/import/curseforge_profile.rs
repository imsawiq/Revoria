use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Cursor;
use std::path::PathBuf;

use async_zip::base::read::seek::ZipFileReader;
use futures::{StreamExt, TryStreamExt};
use path_util::SafeRelativeUtf8UnixPathBuf;
use reqwest::Method;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::{
    State,
    event::{
        LoadingBarId, LoadingBarType, ProfilePayloadType,
        emit::{
            check_loading_cancelled, emit_loading, emit_profile, init_loading,
            init_or_edit_loading,
        },
    },
    prelude::ModLoader,
    state::{ProfileInstallStage, ProjectType},
    util::fetch::{
        REQWEST_CLIENT, fetch, fetch_advanced, sha1_async, write,
        write_cached_icon,
    },
};

const CURSEFORGE_API_BASE_URL: &str = "https://api.curseforge.com/v1";
const CURSEFORGE_SHARED_PROFILE_URL: &str =
    "https://api.curseforge.com/v1/shared-profile";
const CURSEFORGE_API_KEY: &str =
    "$2a$10$vB69qL5rZBOO5DQICbYBO.vAK9U3rN3okLrin.WgqGTg1AjUnE4CC";
const CURSEFORGE_RELATION_REQUIRED_DEPENDENCY: u32 = 3;
const CURSEFORGE_FILE_BATCH_SIZE: usize = 50;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeManifest {
    pub minecraft: CurseForgeMinecraft,
    pub manifest_type: Option<String>,
    pub manifest_version: Option<i32>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub author: Option<String>,
    #[serde(default)]
    pub files: Vec<CurseForgeManifestFile>,
    pub overrides: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeMinecraft {
    pub version: Option<String>,
    #[serde(default)]
    pub mod_loaders: Vec<CurseForgeModLoader>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeModLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CurseForgeManifestFile {
    #[serde(rename = "projectID")]
    pub project_id: u32,
    #[serde(rename = "fileID")]
    pub file_id: u32,
    #[serde(default)]
    pub required: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurseForgeProfileMetadata {
    pub name: String,
    pub download_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeModFilesResponse {
    data: Vec<CurseForgeModFile>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeFilesRequest {
    file_ids: Vec<u32>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeDownloadUrlResponse {
    data: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CurseForgeModFile {
    id: u32,
    mod_id: u32,
    file_name: String,
    #[serde(default)]
    file_length: u64,
    download_url: Option<String>,
    #[serde(default)]
    dependencies: Vec<CurseForgeFileDependency>,
    #[serde(default)]
    hashes: Vec<CurseForgeFileHash>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CurseForgeFileDependency {
    mod_id: u32,
    relation_type: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CurseForgeFileHash {
    value: String,
    algo: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CurseForgeProjectKind {
    Mod,
    Modpack,
    ResourcePack,
    ShaderPack,
    DataPack,
}

impl CurseForgeProjectKind {
    fn from_project_type(value: Option<&str>) -> crate::Result<Self> {
        match value.unwrap_or("mod").to_ascii_lowercase().as_str() {
            "mod" => Ok(Self::Mod),
            "modpack" => Ok(Self::Modpack),
            "resourcepack" | "resource_pack" => Ok(Self::ResourcePack),
            "shader" | "shaderpack" | "shader_pack" => Ok(Self::ShaderPack),
            "datapack" | "data_pack" => Ok(Self::DataPack),
            other => Err(crate::ErrorKind::InputError(format!(
                "Unsupported CurseForge project type: {other}"
            ))
            .into()),
        }
    }

    fn profile_project_type(self) -> Option<ProjectType> {
        match self {
            Self::Mod => Some(ProjectType::Mod),
            Self::ResourcePack => Some(ProjectType::ResourcePack),
            Self::ShaderPack => Some(ProjectType::ShaderPack),
            Self::DataPack => Some(ProjectType::DataPack),
            Self::Modpack => None,
        }
    }

    fn needs_mod_loader(self) -> bool {
        matches!(self, Self::Mod | Self::Modpack)
    }
}

/// Fetch CurseForge profile metadata from profile code.
pub async fn fetch_curseforge_profile_metadata(
    profile_code: &str,
) -> crate::Result<CurseForgeProfileMetadata> {
    let download_url = shared_profile_url(profile_code);
    let state = State::get().await?;
    let zip_bytes =
        fetch(&download_url, None, &state.fetch_semaphore, &state.pool).await?;
    let manifest = read_manifest(zip_bytes).await?;

    Ok(CurseForgeProfileMetadata {
        name: manifest_pack_name(&manifest)
            .unwrap_or_else(|| format!("CurseForge Profile {profile_code}")),
        download_url,
    })
}

/// Import a CurseForge shared profile from profile code.
pub async fn import_curseforge_profile(
    profile_code: &str,
    profile_path: &str,
) -> crate::Result<()> {
    let state = State::get().await?;
    let download_url = shared_profile_url(profile_code);
    let loading_bar = init_loading(
        LoadingBarType::CurseForgeProfileDownload {
            profile_name: profile_path.to_string(),
        },
        100.0,
        "Downloading CurseForge profile",
    )
    .await?;

    let zip_bytes = fetch_advanced(
        Method::GET,
        &download_url,
        None,
        None,
        None,
        Some((&loading_bar, 100.0)),
        &state.fetch_semaphore,
        &state.pool,
    )
    .await?;

    install_curseforge_modpack_zip(
        zip_bytes,
        profile_path,
        None,
        None,
        Some(loading_bar),
    )
    .await
}

/// Install a CurseForge project into an existing profile.
///
/// Modpacks are installed from their CurseForge zip manifest. Other project
/// types use the exact CurseForge file API and refuse third-party blocked
/// files instead of constructing a guessed CDN URL.
pub async fn install_curseforge_project(
    profile_path: &str,
    project_id: u32,
    project_type: Option<&str>,
    project_name: Option<&str>,
    icon_url: Option<&str>,
    use_profile_hints: bool,
) -> crate::Result<()> {
    let kind = CurseForgeProjectKind::from_project_type(project_type)?;
    match kind {
        CurseForgeProjectKind::Modpack => {
            install_curseforge_modpack_project(
                profile_path,
                project_id,
                project_name,
                icon_url,
                use_profile_hints,
            )
            .await
        }
        _ => {
            install_curseforge_addon_project(profile_path, project_id, kind)
                .await
        }
    }
}

async fn install_curseforge_addon_project(
    profile_path: &str,
    project_id: u32,
    kind: CurseForgeProjectKind,
) -> crate::Result<()> {
    let state = State::get().await?;
    let profile =
        crate::api::profile::get(profile_path)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::UnmanagedProfileError(
                    profile_path.to_string(),
                )
            })?;

    if kind == CurseForgeProjectKind::Mod
        && profile.loader == ModLoader::Vanilla
    {
        return Err(crate::ErrorKind::InputError(
            "CurseForge mods require a non-vanilla instance".to_string(),
        )
        .into());
    }

    let loading_bar = init_loading(
        LoadingBarType::PackFileDownload {
            profile_path: profile.path.clone(),
            pack_name: format!("CurseForge project {project_id}"),
            icon: profile.icon_path.clone(),
            pack_version: String::new(),
        },
        100.0,
        "Downloading CurseForge content",
    )
    .await?;

    emit_loading(&loading_bar, 0.0, Some("Resolving CurseForge files"))?;

    let mut resolved_files = Vec::new();
    let mut queued_projects = VecDeque::from([project_id]);
    let mut seen_projects = HashSet::from([project_id]);

    while let Some(next_project_id) = queued_projects.pop_front() {
        let file =
            fetch_best_project_file(next_project_id, Some(&profile), kind)
                .await?;

        for dep in file.dependencies.iter().filter(|dep| {
            dep.relation_type == CURSEFORGE_RELATION_REQUIRED_DEPENDENCY
        }) {
            if seen_projects.insert(dep.mod_id) {
                queued_projects.push_back(dep.mod_id);
            }
        }

        resolved_files.push((next_project_id, file));
    }

    emit_loading(&loading_bar, 10.0, Some("Downloading CurseForge files"))?;

    let progress_per_file = if resolved_files.is_empty() {
        90.0
    } else {
        90.0 / resolved_files.len() as f64
    };

    for (resolved_project_id, file) in resolved_files {
        let bytes =
            download_curseforge_file(&file, &loading_bar, progress_per_file)
                .await?;

        let file_name = sanitize_file_name(&file.file_name);
        let sha1 = file_sha1(&file);
        crate::state::Profile::add_project_bytes(
            profile_path,
            &file_name,
            bytes,
            sha1.as_deref(),
            kind.profile_project_type(),
            &state.io_semaphore,
            &state.pool,
        )
        .await?;

        tracing::info!(
            "Installed CurseForge project {} file {} into {}",
            resolved_project_id,
            file.id,
            profile_path
        );
    }

    emit_profile(profile_path, ProfilePayloadType::Edited).await?;
    Ok(())
}

async fn install_curseforge_modpack_project(
    profile_path: &str,
    project_id: u32,
    project_name: Option<&str>,
    icon_url: Option<&str>,
    use_profile_hints: bool,
) -> crate::Result<()> {
    let state = State::get().await?;
    let profile =
        crate::api::profile::get(profile_path)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::UnmanagedProfileError(
                    profile_path.to_string(),
                )
            })?;

    let file = if use_profile_hints {
        match fetch_best_project_file(
            project_id,
            Some(&profile),
            CurseForgeProjectKind::Modpack,
        )
        .await
        {
            Ok(file) => file,
            Err(_) => {
                fetch_best_project_file(
                    project_id,
                    None,
                    CurseForgeProjectKind::Modpack,
                )
                .await?
            }
        }
    } else {
        fetch_best_project_file(
            project_id,
            None,
            CurseForgeProjectKind::Modpack,
        )
        .await?
    };

    let loading_bar = init_loading(
        LoadingBarType::PackFileDownload {
            profile_path: profile.path.clone(),
            pack_name: project_name.unwrap_or("CurseForge modpack").to_string(),
            icon: icon_url.map(ToString::to_string),
            pack_version: file.id.to_string(),
        },
        100.0,
        "Downloading CurseForge modpack",
    )
    .await?;

    let zip_bytes =
        download_curseforge_file(&file, &loading_bar, 100.0).await?;
    let icon_path = cache_icon(icon_url, &state).await;

    install_curseforge_modpack_zip(
        zip_bytes,
        profile_path,
        Some(CurseForgeModpackSource {
            project_id,
            file_id: file.id,
            fallback_name: project_name.map(ToString::to_string),
            icon_path,
        }),
        project_name,
        Some(loading_bar),
    )
    .await
}

#[derive(Debug)]
struct CurseForgeModpackSource {
    project_id: u32,
    file_id: u32,
    fallback_name: Option<String>,
    icon_path: Option<PathBuf>,
}

async fn install_curseforge_modpack_zip(
    zip_bytes: bytes::Bytes,
    profile_path: &str,
    source: Option<CurseForgeModpackSource>,
    fallback_name: Option<&str>,
    existing_loading_bar: Option<LoadingBarId>,
) -> crate::Result<()> {
    let state = State::get().await?;
    let cursor = Cursor::new(zip_bytes);
    let mut zip_reader =
        ZipFileReader::with_tokio(cursor).await.map_err(|err| {
            crate::ErrorKind::InputError(format!(
                "Failed to read CurseForge modpack zip: {err}"
            ))
        })?;

    let manifest = read_manifest_from_reader(&mut zip_reader).await?;
    let pack_name = manifest_pack_name(&manifest)
        .or_else(|| source.as_ref().and_then(|it| it.fallback_name.clone()))
        .or_else(|| fallback_name.map(ToString::to_string))
        .unwrap_or_else(|| "CurseForge Modpack".to_string());

    let loading_bar = init_or_edit_loading(
        existing_loading_bar,
        LoadingBarType::PackDownload {
            profile_path: profile_path.to_string(),
            pack_name: pack_name.clone(),
            icon: source.as_ref().and_then(|it| it.icon_path.clone()),
            pack_id: source.as_ref().map(|it| it.project_id.to_string()),
            pack_version: source.as_ref().map(|it| it.file_id.to_string()),
        },
        100.0,
        "Installing CurseForge modpack",
    )
    .await?;

    let game_version = manifest.minecraft.version.clone().ok_or_else(|| {
        crate::ErrorKind::InputError(
            "CurseForge modpack manifest did not specify Minecraft version"
                .to_string(),
        )
    })?;
    let (mod_loader, loader_version) =
        select_manifest_modloader(&manifest.minecraft);
    let resolved_loader_version = if mod_loader != ModLoader::Vanilla {
        crate::launcher::get_loader_version_from_profile(
            &game_version,
            mod_loader,
            loader_version.as_deref(),
        )
        .await?
    } else {
        None
    };
    let cached_icon = source.as_ref().and_then(|it| {
        it.icon_path
            .clone()
            .map(|path| path.to_string_lossy().to_string())
    });

    crate::api::profile::edit(profile_path, |prof| {
        prof.name = pack_name.clone();
        prof.install_stage = ProfileInstallStage::PackInstalling;
        prof.game_version = game_version.clone();
        prof.loader = mod_loader;
        prof.loader_version = resolved_loader_version.clone().map(|it| it.id);
        if cached_icon.is_some() {
            prof.icon_path.clone_from(&cached_icon);
        }
        prof.linked_data = None;

        async { Ok(()) }
    })
    .await?;

    emit_loading(&loading_bar, 0.0, Some("Fetching CurseForge modpack files"))?;
    let curseforge_files = fetch_manifest_files(&manifest.files).await?;

    let file_progress_total = 70.0;
    let progress_per_file = if curseforge_files.is_empty() {
        file_progress_total
    } else {
        file_progress_total / curseforge_files.len() as f64
    };

    futures::stream::iter(curseforge_files.into_iter())
        .map(Ok::<CurseForgeModFile, crate::Error>)
        .try_for_each_concurrent(Some(6), |file| {
            let loading_bar = loading_bar.clone();
            let state = state.clone();
            let profile_path = profile_path.to_string();

            async move {
                check_loading_cancelled(&loading_bar)?;
                let bytes = download_curseforge_file(
                    &file,
                    &loading_bar,
                    progress_per_file,
                )
                .await?;
                let file_name = sanitize_file_name(&file.file_name);
                let sha1 = file_sha1(&file);
                crate::state::Profile::add_project_bytes(
                    &profile_path,
                    &file_name,
                    bytes,
                    sha1.as_deref(),
                    Some(ProjectType::Mod),
                    &state.io_semaphore,
                    &state.pool,
                )
                .await?;
                check_loading_cancelled(&loading_bar)?;

                Ok(())
            }
        })
        .await?;

    extract_overrides(
        &mut zip_reader,
        &manifest,
        profile_path,
        &loading_bar,
        30.0,
    )
    .await?;

    check_loading_cancelled(&loading_bar)?;
    if let Some(profile) = crate::api::profile::get(profile_path).await? {
        crate::launcher::install_minecraft(&profile, Some(loading_bar), false)
            .await?;
    }

    emit_profile(profile_path, ProfilePayloadType::Synced).await?;

    Ok(())
}

async fn read_manifest(
    zip_bytes: bytes::Bytes,
) -> crate::Result<CurseForgeManifest> {
    let cursor = Cursor::new(zip_bytes);
    let mut zip_reader =
        ZipFileReader::with_tokio(cursor).await.map_err(|err| {
            crate::ErrorKind::InputError(format!(
                "Failed to read CurseForge modpack zip: {err}"
            ))
        })?;

    read_manifest_from_reader(&mut zip_reader).await
}

async fn read_manifest_from_reader<R>(
    zip_reader: &mut ZipFileReader<R>,
) -> crate::Result<CurseForgeManifest>
where
    R: futures::io::AsyncBufRead + futures::io::AsyncSeek + Unpin,
{
    let manifest_index = zip_reader
        .file()
        .entries()
        .iter()
        .position(|entry| {
            entry.filename().as_str().unwrap_or_default() == "manifest.json"
        })
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "No manifest.json found in CurseForge modpack".to_string(),
            )
        })?;

    let mut manifest_content = String::new();
    let mut reader = zip_reader
        .reader_with_entry(manifest_index)
        .await
        .map_err(|err| {
            crate::ErrorKind::InputError(format!(
                "Failed to read manifest.json: {err}"
            ))
        })?;
    reader.read_to_string_checked(&mut manifest_content).await?;

    Ok(serde_json::from_str(&manifest_content)?)
}

async fn extract_overrides<R>(
    zip_reader: &mut ZipFileReader<R>,
    manifest: &CurseForgeManifest,
    profile_path: &str,
    loading_bar: &LoadingBarId,
    progress_total: f64,
) -> crate::Result<()>
where
    R: futures::io::AsyncBufRead + futures::io::AsyncSeek + Unpin,
{
    let Some(overrides) = manifest
        .overrides
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        emit_loading(loading_bar, progress_total, Some("No overrides"))?;
        return Ok(());
    };

    let prefix = format!("{}/", overrides.trim_matches('/'));
    let entries_to_extract = zip_reader
        .file()
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let file_path = entry.filename().as_str().unwrap_or_default();
            (file_path.starts_with(&prefix) && !file_path.ends_with('/'))
                .then(|| (index, file_path.to_string()))
        })
        .collect::<Vec<_>>();

    if entries_to_extract.is_empty() {
        emit_loading(loading_bar, progress_total, Some("No overrides"))?;
        return Ok(());
    }

    let profile_full_path =
        crate::api::profile::get_full_path(profile_path).await?;
    let state = State::get().await?;
    let progress_per_file = progress_total / entries_to_extract.len() as f64;

    for (index, file_path) in entries_to_extract {
        check_loading_cancelled(loading_bar)?;
        let relative_path =
            file_path.strip_prefix(&prefix).ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Invalid override path in CurseForge modpack: {file_path}"
                ))
            })?;
        let safe_path =
            SafeRelativeUtf8UnixPathBuf::try_from(relative_path.to_string())?;

        let mut reader =
            zip_reader.reader_with_entry(index).await.map_err(|err| {
                crate::ErrorKind::InputError(format!(
                    "Failed to read override {file_path}: {err}"
                ))
            })?;
        let mut file_bytes = Vec::new();
        reader.read_to_end_checked(&mut file_bytes).await?;

        write(
            &profile_full_path.join(safe_path.as_str()),
            &file_bytes,
            &state.io_semaphore,
        )
        .await?;
        emit_loading(
            loading_bar,
            progress_per_file,
            Some("Extracting overrides"),
        )?;
    }

    Ok(())
}

fn manifest_pack_name(manifest: &CurseForgeManifest) -> Option<String> {
    manifest
        .name
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn shared_profile_url(profile_code: &str) -> String {
    format!("{CURSEFORGE_SHARED_PROFILE_URL}/{profile_code}")
}

fn select_manifest_modloader(
    minecraft: &CurseForgeMinecraft,
) -> (ModLoader, Option<String>) {
    minecraft
        .mod_loaders
        .iter()
        .find(|loader| loader.primary)
        .or_else(|| minecraft.mod_loaders.first())
        .map(|loader| parse_modloader(&loader.id))
        .unwrap_or((ModLoader::Vanilla, None))
}

/// Parse CurseForge modloader ID into ModLoader and version.
fn parse_modloader(id: &str) -> (ModLoader, Option<String>) {
    if let Some(version) = id.strip_prefix("forge-") {
        (ModLoader::Forge, Some(version.to_string()))
    } else if let Some(version) = id.strip_prefix("fabric-") {
        (ModLoader::Fabric, Some(version.to_string()))
    } else if let Some(version) = id.strip_prefix("quilt-") {
        (ModLoader::Quilt, Some(version.to_string()))
    } else if let Some(version) = id.strip_prefix("neoforge-") {
        (ModLoader::NeoForge, Some(version.to_string()))
    } else {
        (ModLoader::Vanilla, None)
    }
}

async fn fetch_best_project_file(
    project_id: u32,
    profile: Option<&crate::state::Profile>,
    kind: CurseForgeProjectKind,
) -> crate::Result<CurseForgeModFile> {
    let game_version = profile.map(|profile| profile.game_version.as_str());
    let mod_loader_type = if kind.needs_mod_loader() {
        profile.and_then(|profile| curseforge_loader_type(profile.loader))
    } else {
        None
    };

    let response =
        fetch_project_files(project_id, game_version, mod_loader_type, Some(1))
            .await?;
    let file = response.data.into_iter().next().ok_or_else(|| {
        let version_hint = game_version.unwrap_or("latest");
        crate::ErrorKind::InputError(format!(
            "Unable to find a CurseForge file for project {project_id} matching {version_hint}"
        ))
    })?;

    if file.mod_id != project_id {
        return Err(crate::ErrorKind::InputError(format!(
            "Mismatched CurseForge file: expected project {project_id}, got {}",
            file.mod_id
        ))
        .into());
    }

    Ok(file)
}

async fn fetch_project_files(
    project_id: u32,
    game_version: Option<&str>,
    mod_loader_type: Option<u32>,
    page_size: Option<u32>,
) -> crate::Result<CurseForgeModFilesResponse> {
    let mut query = Vec::<(&str, String)>::new();
    if let Some(game_version) = game_version {
        query.push(("gameVersion", game_version.to_string()));
    }
    if let Some(mod_loader_type) = mod_loader_type {
        query.push(("modLoaderType", mod_loader_type.to_string()));
    }
    if let Some(page_size) = page_size {
        query.push(("pageSize", page_size.to_string()));
    }

    curseforge_get_json(&format!("/mods/{project_id}/files"), &query).await
}

async fn fetch_manifest_files(
    manifest_files: &[CurseForgeManifestFile],
) -> crate::Result<Vec<CurseForgeModFile>> {
    let file_ids = manifest_files
        .iter()
        .map(|file| file.file_id)
        .collect::<Vec<_>>();
    let mut files = Vec::new();

    for chunk in file_ids.chunks(CURSEFORGE_FILE_BATCH_SIZE) {
        let response: CurseForgeModFilesResponse = curseforge_post_json(
            "/mods/files",
            &CurseForgeFilesRequest {
                file_ids: chunk.to_vec(),
            },
        )
        .await?;
        files.extend(response.data);
    }

    let files_by_id = files
        .into_iter()
        .map(|file| (file.id, file))
        .collect::<HashMap<_, _>>();

    manifest_files
        .iter()
        .map(|manifest_file| {
            let file = files_by_id
                .get(&manifest_file.file_id)
                .cloned()
                .ok_or_else(|| {
                    crate::ErrorKind::InputError(format!(
                        "CurseForge file {} from project {} was not found",
                        manifest_file.file_id, manifest_file.project_id
                    ))
                })?;

            if file.mod_id != manifest_file.project_id {
                return Err(crate::ErrorKind::InputError(format!(
                    "Mismatched CurseForge manifest file {}: expected project {}, got {}",
                    manifest_file.file_id, manifest_file.project_id, file.mod_id
                ))
                .into());
            }

            Ok(file)
        })
        .collect()
}

async fn curseforge_get_json<T: DeserializeOwned>(
    path: &str,
    query: &[(&str, String)],
) -> crate::Result<T> {
    let url = format!("{CURSEFORGE_API_BASE_URL}{path}");
    let response = REQWEST_CLIENT
        .get(&url)
        .header("x-api-key", CURSEFORGE_API_KEY)
        .header("Accept", "application/json")
        .query(query)
        .send()
        .await?;

    parse_curseforge_response(response).await
}

async fn curseforge_post_json<T, B>(path: &str, body: &B) -> crate::Result<T>
where
    T: DeserializeOwned,
    B: Serialize + ?Sized,
{
    let url = format!("{CURSEFORGE_API_BASE_URL}{path}");
    let response = REQWEST_CLIENT
        .post(&url)
        .header("x-api-key", CURSEFORGE_API_KEY)
        .header("Accept", "application/json")
        .json(body)
        .send()
        .await?;

    parse_curseforge_response(response).await
}

async fn parse_curseforge_response<T: DeserializeOwned>(
    response: reqwest::Response,
) -> crate::Result<T> {
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(crate::ErrorKind::InputError(format!(
            "CurseForge API request failed ({status}): {body}"
        ))
        .into());
    }

    Ok(response.json().await?)
}

async fn download_curseforge_file(
    file: &CurseForgeModFile,
    loading_bar: &LoadingBarId,
    progress_total: f64,
) -> crate::Result<bytes::Bytes> {
    let sha1 = file_sha1(file).ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "CurseForge file {} has no sha1 hash",
            file.id
        ))
    })?;
    let download_urls = curseforge_download_urls(file).await;
    check_loading_cancelled(loading_bar)?;

    emit_loading(
        loading_bar,
        0.0,
        Some(&format!("Downloading {}", file.file_name)),
    )?;

    let mut last_status = None;
    let mut response = None;
    for download_url in download_urls {
        check_loading_cancelled(loading_bar)?;
        let candidate = REQWEST_CLIENT.get(&download_url).send().await?;
        if candidate.status().is_success() {
            response = Some(candidate);
            break;
        }

        last_status = Some(candidate.status());
    }

    let Some(response) = response else {
        let status = last_status
            .map(|status| status.to_string())
            .unwrap_or_else(|| "no usable URL".to_string());
        return Err(crate::ErrorKind::InputError(format!(
            "CurseForge download failed for {} ({status})",
            file.file_name
        ))
        .into());
    };

    let expected_size =
        file.file_length.max(response.content_length().unwrap_or(0));
    let mut stream = response.bytes_stream();
    let mut bytes = Vec::new();
    let mut emitted_progress = 0.0;

    while let Some(chunk) = stream.next().await {
        check_loading_cancelled(loading_bar)?;
        let chunk = chunk?;
        bytes.extend_from_slice(&chunk);

        if expected_size > 0 {
            let increment =
                (chunk.len() as f64 / expected_size as f64) * progress_total;
            emitted_progress += increment;
            emit_loading(loading_bar, increment, None)?;
        }
    }

    let bytes = bytes::Bytes::from(bytes);
    if file.file_length > 0 && bytes.len() as u64 != file.file_length {
        return Err(crate::ErrorKind::InputError(format!(
            "CurseForge file {} had the wrong size",
            file.file_name
        ))
        .into());
    }

    let actual_sha1 = sha1_async(bytes.clone()).await?;
    if actual_sha1 != sha1 {
        return Err(crate::ErrorKind::HashError(sha1, actual_sha1).into());
    }

    if expected_size == 0 || emitted_progress < progress_total {
        emit_loading(
            loading_bar,
            (progress_total - emitted_progress).max(0.0),
            None,
        )?;
    }

    Ok(bytes)
}

async fn curseforge_download_urls(file: &CurseForgeModFile) -> Vec<String> {
    let mut urls = Vec::new();
    if let Some(download_url) = file.download_url.as_deref() {
        urls.push(download_url.to_string());
    } else if let Ok(download_url) =
        fetch_curseforge_file_download_url(file.mod_id, file.id).await
    {
        urls.push(download_url);
    }

    let cdn_url = curseforge_cdn_url(file.id, &file.file_name);
    if !urls.iter().any(|url| url == &cdn_url) {
        urls.push(cdn_url);
    }

    urls
}

async fn fetch_curseforge_file_download_url(
    mod_id: u32,
    file_id: u32,
) -> crate::Result<String> {
    let response: CurseForgeDownloadUrlResponse = curseforge_get_json(
        &format!("/mods/{mod_id}/files/{file_id}/download-url"),
        &[],
    )
    .await?;

    response.data.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "CurseForge file {file_id} has no download URL"
        ))
        .into()
    })
}

fn curseforge_cdn_url(file_id: u32, file_name: &str) -> String {
    let id = file_id.to_string();
    let split_at = id.len().min(4);
    let (first, rest) = id.split_at(split_at);
    let encoded_file_name = urlencoding::encode(file_name);

    format!(
        "https://edge.forgecdn.net/files/{first}/{rest}/{encoded_file_name}"
    )
}

fn file_sha1(file: &CurseForgeModFile) -> Option<String> {
    file.hashes
        .iter()
        .find(|hash| hash.algo == 1)
        .map(|hash| hash.value.clone())
}

fn curseforge_loader_type(loader: ModLoader) -> Option<u32> {
    match loader {
        ModLoader::Forge => Some(1),
        ModLoader::Fabric => Some(4),
        ModLoader::Quilt => Some(5),
        ModLoader::NeoForge => Some(6),
        ModLoader::Vanilla => None,
    }
}

async fn cache_icon(icon_url: Option<&str>, state: &State) -> Option<PathBuf> {
    let icon_url = icon_url?;
    let icon_bytes = fetch(icon_url, None, &state.fetch_semaphore, &state.pool)
        .await
        .ok()?;
    let filename = icon_url.rsplit('/').next()?;

    write_cached_icon(
        filename,
        &state.directories.caches_dir(),
        icon_bytes,
        &state.io_semaphore,
    )
    .await
    .ok()
}

fn sanitize_file_name(file_name: &str) -> String {
    let sanitized = file_name
        .chars()
        .map(|ch| {
            if matches!(
                ch,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|'
            ) || ch.is_control()
            {
                '_'
            } else {
                ch
            }
        })
        .collect::<String>()
        .trim()
        .trim_matches('.')
        .to_string();

    if sanitized.is_empty() {
        "curseforge-file.jar".to_string()
    } else {
        sanitized
    }
}
