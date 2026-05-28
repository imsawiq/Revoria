//! # Content API
//!
//! ## Data Flow
//!
//! 1. Frontend calls `get_content_items(profile_path)`
//! 2. Backend fetches all installed files via `Profile::get_projects()`
//! 3. If profile is linked to a modpack:
//!    - Fetch modpack file hashes from cache (populated during installation)
//!    - Fallback: re-download .mrpack if cache miss (cleared/expired)
//!    - Filter out files that belong to the modpack
//! 4. For remaining files, fetch project/version/owner metadata in parallel
//! 5. Return sorted `ContentItem` list
//!
//! ## Caching
//!
//! Modpack file hashes are cached in `CacheValueType::ModpackFiles`
//! during modpack installation. The cache never expires (version_id is
//! immutable), so re-download is only needed if cache was cleared or
//! profile predates this caching mechanism.

use crate::pack::install_from::{PackFileHash, PackFormat};
use crate::state::profiles::{Profile, ProfileFile, ProjectType};
use crate::state::{CacheBehaviour, CachedEntry};
use crate::util::fetch::{FetchSemaphore, fetch_mirrors, sha1_async};
use async_zip::base::read::seek::ZipFileReader;
use base64::Engine;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashSet;
use std::io::{Cursor, Read, Seek};
use std::path::Path;
use zip::ZipArchive;

/// Content item with rich metadata for frontend display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentItem {
    /// Unique identifier (the file name)
    pub file_name: String,
    /// Relative path to the file within the profile
    pub file_path: String,
    /// SHA1 hash of the file
    pub hash: String,
    /// File size in bytes
    pub size: u64,
    /// Whether the file is enabled (not .disabled)
    pub enabled: bool,
    /// Type of project (mod, resourcepack, etc.)
    pub project_type: ProjectType,
    /// Modrinth project info if recognized
    pub project: Option<ContentItemProject>,
    /// Version info if recognized
    pub version: Option<ContentItemVersion>,
    /// Owner info (organization or user)
    pub owner: Option<ContentItemOwner>,
    /// Whether an update is available
    pub has_update: bool,
    /// The recommended version ID to update to (if has_update is true)
    pub update_version_id: Option<String>,
    /// When the file was added to the instance (file modification time)
    pub date_added: Option<String>,
}

/// Project information for content item display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentItemProject {
    pub id: String,
    pub slug: Option<String>,
    pub title: String,
    pub icon_url: Option<String>,
    pub description: Option<String>,
}

/// Version information for content item display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentItemVersion {
    pub id: String,
    pub version_number: String,
    pub file_name: String,
    pub date_published: Option<String>,
}

/// Owner information for content item display
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ContentItemOwner {
    pub id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    #[serde(rename = "type")]
    pub owner_type: OwnerType,
}

/// Type of content owner
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OwnerType {
    User,
    Organization,
}

use crate::state::cache::{Dependency, Organization, TeamMember};
use crate::state::{Project, Version};

/// Full linked modpack information including owner and update status
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkedModpackInfo {
    pub project: Project,
    pub version: Version,
    pub owner: Option<ContentItemOwner>,
    /// Whether an update is available for this modpack
    pub has_update: bool,
    /// The version ID to update to (if has_update is true)
    pub update_version_id: Option<String>,
    /// The full version info for the update (if has_update is true)
    pub update_version: Option<Version>,
}

/// Get linked modpack info including project, version, owner, and update status.
/// Returns None if the profile is not linked to a modpack.
pub async fn get_linked_modpack_info(
    profile: &Profile,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Option<LinkedModpackInfo>> {
    let Some(linked_data) = &profile.linked_data else {
        return Ok(None);
    };

    // Vanilla server projects have linked_data with an empty version_id
    if linked_data.version_id.is_empty() {
        return Ok(None);
    }

    // Fetch project, version, and all project versions in parallel
    let (project, version, all_versions) = tokio::try_join!(
        CachedEntry::get_project(
            &linked_data.project_id,
            cache_behaviour,
            pool,
            fetch_semaphore,
        ),
        CachedEntry::get_version(
            &linked_data.version_id,
            cache_behaviour,
            pool,
            fetch_semaphore,
        ),
        CachedEntry::get_project_versions(
            &linked_data.project_id,
            cache_behaviour,
            pool,
            fetch_semaphore,
        ),
    )?;

    let version = version.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Linked modpack version {} not found",
            linked_data.version_id
        ))
    })?;

    // For server instances, linked_data.project_id is the server project,
    // but the version may belong to a different (modpack) project.
    // If so, fetch the actual modpack project for display and update checking.
    let (project, all_versions) =
        if version.project_id != linked_data.project_id {
            let (modpack_project, modpack_versions) = tokio::try_join!(
                CachedEntry::get_project(
                    &version.project_id,
                    cache_behaviour,
                    pool,
                    fetch_semaphore,
                ),
                CachedEntry::get_project_versions(
                    &version.project_id,
                    cache_behaviour,
                    pool,
                    fetch_semaphore,
                ),
            )?;
            (modpack_project.or(project), modpack_versions)
        } else {
            (project, all_versions)
        };

    let project = project.ok_or_else(|| {
        crate::ErrorKind::InputError(format!(
            "Linked modpack project {} not found",
            linked_data.project_id
        ))
    })?;

    // Resolve owner - prefer organization, fall back to team owner
    let owner = if let Some(org_id) = &project.organization {
        let org = CachedEntry::get_organization(
            org_id,
            cache_behaviour,
            pool,
            fetch_semaphore,
        )
        .await?;
        org.map(|o| ContentItemOwner {
            id: o.id,
            name: o.name,
            avatar_url: o.icon_url,
            owner_type: OwnerType::Organization,
        })
    } else {
        let team = CachedEntry::get_team(
            &project.team,
            cache_behaviour,
            pool,
            fetch_semaphore,
        )
        .await?;
        team.and_then(|t| {
            t.into_iter()
                .find(|m| m.is_owner)
                .map(|m| ContentItemOwner {
                    id: m.user.id,
                    name: m.user.username,
                    avatar_url: m.user.avatar_url,
                    owner_type: OwnerType::User,
                })
        })
    };

    // Check for updates
    let (has_update, update_version_id, update_version) = check_modpack_update(
        profile,
        &linked_data.version_id,
        &version,
        all_versions,
    );

    Ok(Some(LinkedModpackInfo {
        project,
        version,
        owner,
        has_update,
        update_version_id,
        update_version,
    }))
}

/// Check if a newer compatible version exists for the linked modpack.
/// Returns (has_update, update_version_id, update_version).
fn check_modpack_update(
    profile: &Profile,
    installed_version_id: &str,
    installed_version: &Version,
    all_versions: Option<Vec<Version>>,
) -> (bool, Option<String>, Option<Version>) {
    let Some(versions) = all_versions else {
        return (false, None, None);
    };

    // Get the loader as a string for comparison
    let loader_str = profile.loader.as_str().to_lowercase();
    let game_version = &profile.game_version;

    // Filter to compatible versions
    let mut compatible_versions: Vec<&Version> = versions
        .iter()
        .filter(|v| {
            // Must support the profile's game version
            let supports_game = v.game_versions.contains(game_version);

            // Must support the profile's loader
            // The v2 API replaces "mrpack" with actual loaders from mrpack_loaders,
            // but if mrpack_loaders is missing, loaders may be just ["mrpack"].
            // In that case we can't filter by loader, so accept the version.
            let real_loaders: Vec<_> = v
                .loaders
                .iter()
                .filter(|l| l.to_lowercase() != "mrpack")
                .collect();
            let supports_loader = real_loaders.is_empty()
                || real_loaders.iter().any(|l| l.to_lowercase() == loader_str);

            supports_game && supports_loader
        })
        .collect();

    // Sort by date_published descending (newest first)
    compatible_versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));

    // Find the newest compatible version
    if let Some(newest) = compatible_versions.first() {
        // Check if the newest version is different and newer than installed
        if newest.id != installed_version_id
            && newest.date_published > installed_version.date_published
        {
            return (true, Some(newest.id.clone()), Some((*newest).clone()));
        }
    }

    (false, None, None)
}

/// Get content items with rich metadata, filtered to exclude modpack content.
/// Returns only user-added content (not part of the linked modpack).
pub async fn get_content_items(
    profile: &Profile,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
    let all_files = profile
        .get_projects(cache_behaviour, pool, fetch_semaphore)
        .await?;

    let modpack_hashes: HashSet<String> = if let Some(ref linked_data) =
        profile.linked_data
    {
        if linked_data.version_id.is_empty() {
            HashSet::new()
        } else {
            tracing::info!(
                "Fetching modpack file hashes for version_id={}, project_id={}",
                linked_data.version_id,
                linked_data.project_id
            );
            match get_modpack_file_hashes(
                &linked_data.version_id,
                pool,
                fetch_semaphore,
            )
            .await
            {
                Ok(hashes) => {
                    tracing::info!(
                        "Got {} modpack file hashes for version {}",
                        hashes.len(),
                        linked_data.version_id
                    );
                    hashes
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to fetch modpack file hashes for version {}: {}",
                        linked_data.version_id,
                        e
                    );
                    HashSet::new()
                }
            }
        }
    } else {
        HashSet::new()
    };

    let user_files: Vec<(String, ProfileFile)> = all_files
        .into_iter()
        .filter(|(_, file)| !modpack_hashes.contains(&file.hash))
        .collect();

    profile_files_to_content_items(
        &profile.path,
        &user_files,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await
}

/// Pre-fetched metadata for projects, versions, teams, and organizations.
struct ResolvedMetadata {
    projects: Vec<Project>,
    versions: Vec<Version>,
    teams: Vec<Vec<TeamMember>>,
    organizations: Vec<Organization>,
}

#[derive(Debug, Default, Clone)]
struct LocalContentMetadata {
    title: Option<String>,
    version_number: Option<String>,
    author: Option<String>,
    description: Option<String>,
    icon_url: Option<String>,
}

fn load_local_content_metadata(path: &Path) -> Option<LocalContentMetadata> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;

    load_fabric_like_metadata(&mut archive, "fabric.mod.json")
        .or_else(|| load_fabric_like_metadata(&mut archive, "quilt.mod.json"))
        .or_else(|| load_forge_metadata(&mut archive, "META-INF/mods.toml"))
        .or_else(|| {
            load_forge_metadata(&mut archive, "META-INF/neoforge.mods.toml")
        })
        .or_else(|| load_mcmod_info_metadata(&mut archive))
        .or_else(|| load_resource_pack_metadata(&mut archive, path))
        .or_else(|| load_manifest_metadata(&mut archive))
}

fn display_file_name(file_name: &str) -> String {
    file_name
        .strip_suffix(".disabled")
        .unwrap_or(file_name)
        .to_string()
}

fn load_fabric_like_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    entry_name: &str,
) -> Option<LocalContentMetadata> {
    let bytes = read_zip_entry(archive, entry_name)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let icon_path = fabric_icon_path(value.get("icon"));
    let icon_url =
        icon_path.and_then(|path| read_icon_data_url(archive, &path));

    Some(LocalContentMetadata {
        title: value
            .get("name")
            .and_then(json_string)
            .or_else(|| value.get("id").and_then(json_string)),
        version_number: value.get("version").and_then(json_string),
        author: authors_from_json(value.get("authors"))
            .or_else(|| authors_from_json(value.get("author"))),
        description: value.get("description").and_then(json_text),
        icon_url,
    })
}

fn load_forge_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    entry_name: &str,
) -> Option<LocalContentMetadata> {
    let bytes = read_zip_entry(archive, entry_name)?;
    let text = String::from_utf8_lossy(&bytes);
    let mod_block = text.split("[[mods]]").nth(1).unwrap_or(text.as_ref());
    let icon_path = extract_toml_string(mod_block, "logoFile");
    let icon_url =
        icon_path.and_then(|path| read_icon_data_url(archive, &path));

    let version_number = extract_toml_string(mod_block, "version")
        .filter(|version| !version.contains("${"));
    let author = extract_toml_string(mod_block, "authors").or_else(|| {
        extract_toml_array_strings(mod_block, "authors").map(|v| v.join(", "))
    });

    Some(LocalContentMetadata {
        title: extract_toml_string(mod_block, "displayName")
            .or_else(|| extract_toml_string(mod_block, "modId")),
        version_number,
        author,
        description: extract_toml_multiline_string(mod_block, "description")
            .or_else(|| extract_toml_string(mod_block, "description")),
        icon_url,
    })
}

fn load_mcmod_info_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Option<LocalContentMetadata> {
    let bytes = read_zip_entry(archive, "mcmod.info")?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let entry = match &value {
        serde_json::Value::Array(items) => items.first()?,
        serde_json::Value::Object(_) => value
            .get("modList")
            .and_then(|list| list.as_array())
            .and_then(|items| items.first())
            .unwrap_or(&value),
        _ => return None,
    };

    let icon_path = entry
        .get("logoFile")
        .and_then(json_string)
        .or_else(|| entry.get("logo").and_then(json_string));
    let icon_url =
        icon_path.and_then(|path| read_icon_data_url(archive, &path));

    Some(LocalContentMetadata {
        title: entry
            .get("name")
            .and_then(json_string)
            .or_else(|| entry.get("modid").and_then(json_string)),
        version_number: entry.get("version").and_then(json_string),
        author: authors_from_json(entry.get("authorList"))
            .or_else(|| authors_from_json(entry.get("authors"))),
        description: entry.get("description").and_then(json_text),
        icon_url,
    })
}

fn load_resource_pack_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    path: &Path,
) -> Option<LocalContentMetadata> {
    let bytes = read_zip_entry(archive, "pack.mcmeta")?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let pack = value.get("pack")?;
    let description = pack.get("description").and_then(json_text);
    let icon_url = read_icon_data_url(archive, "pack.png");

    Some(LocalContentMetadata {
        title: path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(display_file_name),
        version_number: None,
        author: None,
        description,
        icon_url,
    })
}

fn load_manifest_metadata<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Option<LocalContentMetadata> {
    let bytes = read_zip_entry(archive, "META-INF/MANIFEST.MF")?;
    let manifest = String::from_utf8_lossy(&bytes);

    let title = manifest_value(&manifest, "Implementation-Title")
        .or_else(|| manifest_value(&manifest, "Specification-Title"))
        .or_else(|| manifest_value(&manifest, "Automatic-Module-Name"));
    let version_number = manifest_value(&manifest, "Implementation-Version")
        .or_else(|| manifest_value(&manifest, "Specification-Version"));
    let author = manifest_value(&manifest, "Implementation-Vendor")
        .or_else(|| manifest_value(&manifest, "Specification-Vendor"));

    if title.is_none() && version_number.is_none() && author.is_none() {
        return None;
    }

    Some(LocalContentMetadata {
        title,
        version_number,
        author,
        description: None,
        icon_url: None,
    })
}

fn read_zip_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    entry_name: &str,
) -> Option<Vec<u8>> {
    let mut file = archive.by_name(entry_name.trim_start_matches('/')).ok()?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

fn read_icon_data_url<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    icon_path: &str,
) -> Option<String> {
    let icon_path = icon_path.trim().trim_start_matches('/');
    let bytes = read_zip_entry(archive, icon_path)?;

    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
    } else {
        None
    }
}

fn fabric_icon_path(value: Option<&serde_json::Value>) -> Option<String> {
    match value? {
        serde_json::Value::String(path) => non_empty(path),
        serde_json::Value::Object(map) => map
            .get("64")
            .and_then(json_string)
            .or_else(|| map.values().find_map(json_string)),
        _ => None,
    }
}

fn authors_from_json(value: Option<&serde_json::Value>) -> Option<String> {
    match value? {
        serde_json::Value::String(value) => non_empty(value),
        serde_json::Value::Array(items) => {
            let authors: Vec<String> = items
                .iter()
                .filter_map(|item| {
                    json_string(item)
                        .or_else(|| item.get("name").and_then(json_string))
                })
                .collect();
            (!authors.is_empty()).then(|| authors.join(", "))
        }
        serde_json::Value::Object(map) => map.get("name").and_then(json_string),
        _ => None,
    }
}

fn json_string(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => non_empty(value),
        serde_json::Value::Number(value) => Some(value.to_string()),
        serde_json::Value::Bool(value) => Some(value.to_string()),
        _ => None,
    }
}

fn json_text(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(value) => non_empty(value),
        serde_json::Value::Array(items) => {
            let text: String = items.iter().filter_map(json_text).collect();
            non_empty(&text)
        }
        serde_json::Value::Object(map) => {
            let mut text = String::new();
            if let Some(part) = map.get("text").and_then(json_text) {
                text.push_str(&part);
            }
            if let Some(extra) = map.get("extra").and_then(json_text) {
                text.push_str(&extra);
            }
            non_empty(&text)
        }
        _ => None,
    }
}

fn extract_toml_string(source: &str, key: &str) -> Option<String> {
    source.lines().find_map(|line| {
        let line = line.trim();
        let (line_key, value) = line.split_once('=')?;
        (line_key.trim() == key)
            .then(|| parse_toml_string_value(value.trim()))
            .flatten()
    })
}

fn extract_toml_array_strings(source: &str, key: &str) -> Option<Vec<String>> {
    let value = source.lines().find_map(|line| {
        let line = line.trim();
        let (line_key, value) = line.split_once('=')?;
        (line_key.trim() == key).then_some(value.trim())
    })?;
    let value = value.strip_prefix('[')?.strip_suffix(']')?;
    let values: Vec<String> = value
        .split(',')
        .filter_map(|part| parse_toml_string_value(part.trim()))
        .collect();

    (!values.is_empty()).then_some(values)
}

fn extract_toml_multiline_string(source: &str, key: &str) -> Option<String> {
    let start = format!("{key} = \"\"\"");
    let mut lines = source.lines();

    while let Some(line) = lines.next() {
        let Some(after_start) = line.trim().strip_prefix(&start) else {
            continue;
        };

        let mut collected = Vec::new();
        if let Some((first, _)) = after_start.split_once("\"\"\"") {
            return non_empty(first.trim());
        }
        if !after_start.is_empty() {
            collected.push(after_start.to_string());
        }

        for line in lines.by_ref() {
            if let Some((last, _)) = line.split_once("\"\"\"") {
                if !last.is_empty() {
                    collected.push(last.to_string());
                }
                return non_empty(&collected.join("\n"));
            }
            collected.push(line.to_string());
        }
    }

    None
}

fn parse_toml_string_value(value: &str) -> Option<String> {
    let value = value.trim().trim_end_matches(',');
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' {
        return non_empty(value);
    }

    let mut escaped = false;
    let mut output = String::new();
    for ch in value[quote.len_utf8()..].chars() {
        if escaped {
            output.push(ch);
            escaped = false;
            continue;
        }
        if quote == '"' && ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return non_empty(output.trim());
        }
        output.push(ch);
    }

    non_empty(output.trim())
}

fn manifest_value(manifest: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    manifest.lines().find_map(|line| {
        line.strip_prefix(&prefix)
            .and_then(|value| non_empty(value.trim()))
    })
}

fn non_empty(value: impl AsRef<str>) -> Option<String> {
    let value = value.as_ref().trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// Fetch project, version, team, and organization metadata in parallel batches.
async fn resolve_metadata(
    project_ids: &HashSet<String>,
    version_ids: &HashSet<String>,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<ResolvedMetadata> {
    let project_ids_vec: Vec<&str> =
        project_ids.iter().map(|s| s.as_str()).collect();
    let version_ids_vec: Vec<&str> =
        version_ids.iter().map(|s| s.as_str()).collect();

    let (projects, versions) =
        if !project_ids.is_empty() || !version_ids.is_empty() {
            tokio::try_join!(
                async {
                    if project_ids.is_empty() {
                        Ok(Vec::new())
                    } else {
                        CachedEntry::get_project_many(
                            &project_ids_vec,
                            cache_behaviour,
                            pool,
                            fetch_semaphore,
                        )
                        .await
                    }
                },
                async {
                    if version_ids.is_empty() {
                        Ok(Vec::new())
                    } else {
                        CachedEntry::get_version_many(
                            &version_ids_vec,
                            cache_behaviour,
                            pool,
                            fetch_semaphore,
                        )
                        .await
                    }
                }
            )?
        } else {
            (Vec::new(), Vec::new())
        };

    let team_ids: HashSet<String> =
        projects.iter().map(|p| p.team.clone()).collect();
    let org_ids: HashSet<String> = projects
        .iter()
        .filter_map(|p| p.organization.clone())
        .collect();

    let team_ids_vec: Vec<&str> = team_ids.iter().map(|s| s.as_str()).collect();
    let org_ids_vec: Vec<&str> = org_ids.iter().map(|s| s.as_str()).collect();

    let (teams, organizations) = if !team_ids.is_empty() || !org_ids.is_empty()
    {
        tokio::try_join!(
            async {
                if team_ids.is_empty() {
                    Ok(Vec::new())
                } else {
                    CachedEntry::get_team_many(
                        &team_ids_vec,
                        cache_behaviour,
                        pool,
                        fetch_semaphore,
                    )
                    .await
                }
            },
            async {
                if org_ids.is_empty() {
                    Ok(Vec::new())
                } else {
                    CachedEntry::get_organization_many(
                        &org_ids_vec,
                        cache_behaviour,
                        pool,
                        fetch_semaphore,
                    )
                    .await
                }
            }
        )?
    } else {
        (Vec::new(), Vec::new())
    };

    Ok(ResolvedMetadata {
        projects,
        versions,
        teams,
        organizations,
    })
}

/// Shared helper: convert profile files to ContentItems with rich metadata.
/// Used by both `get_content_items` (user-added files) and
/// `get_linked_modpack_content` (modpack-bundled files).
async fn profile_files_to_content_items(
    profile_path: &str,
    files: &[(String, ProfileFile)],
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
    let project_ids: HashSet<String> = files
        .iter()
        .filter_map(|(_, f)| f.metadata.as_ref().map(|m| m.project_id.clone()))
        .collect();

    let version_ids: HashSet<String> = files
        .iter()
        .filter_map(|(_, f)| f.metadata.as_ref().map(|m| m.version_id.clone()))
        .collect();

    let meta = resolve_metadata(
        &project_ids,
        &version_ids,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await?;

    let profile_base_path =
        crate::api::profile::get_full_path(profile_path).await?;

    let paths: Vec<std::path::PathBuf> = files
        .iter()
        .map(|(path, _)| profile_base_path.join(path))
        .collect();

    let metadata_paths = paths.clone();
    let local_metadata: Vec<Option<LocalContentMetadata>> =
        tokio::task::spawn_blocking(move || {
            metadata_paths
                .iter()
                .map(|path| load_local_content_metadata(path))
                .collect()
        })
        .await?;

    // Batch-read file modification times off the main async runtime
    let modification_times: Vec<Option<String>> =
        tokio::task::spawn_blocking(move || {
            paths
                .iter()
                .map(|path| {
                    std::fs::metadata(path).and_then(|m| m.modified()).ok().map(
                        |t| {
                            chrono::DateTime::<chrono::Utc>::from(t)
                                .to_rfc3339()
                        },
                    )
                })
                .collect()
        })
        .await?;

    let mut items: Vec<ContentItem> = files
        .iter()
        .enumerate()
        .map(|(i, (path, file))| {
            let local = local_metadata[i].as_ref();
            let project = file.metadata.as_ref().and_then(|m| {
                meta.projects.iter().find(|p| p.id == m.project_id)
            });

            let version = file.metadata.as_ref().and_then(|m| {
                meta.versions.iter().find(|v| v.id == m.version_id)
            });

            let owner = project.and_then(|p| {
                resolve_owner(p, &meta.teams, &meta.organizations)
            });
            let owner = owner.or_else(|| {
                local.and_then(|m| {
                    m.author.as_ref().map(|author| ContentItemOwner {
                        id: author.clone(),
                        name: author.clone(),
                        avatar_url: None,
                        owner_type: OwnerType::User,
                    })
                })
            });

            let project = project
                .map(|p| ContentItemProject {
                    id: p.id.clone(),
                    slug: p.slug.clone(),
                    title: p.title.clone(),
                    icon_url: p.icon_url.clone(),
                    description: Some(p.description.clone()),
                })
                .or_else(|| {
                    local.map(|m| ContentItemProject {
                        id: String::new(),
                        slug: None,
                        title: m.title.clone().unwrap_or_else(|| {
                            display_file_name(&file.file_name)
                        }),
                        icon_url: m.icon_url.clone(),
                        description: m.description.clone(),
                    })
                });

            let version = version
                .map(|v| ContentItemVersion {
                    id: v.id.clone(),
                    version_number: v.version_number.clone(),
                    file_name: file.file_name.clone(),
                    date_published: Some(v.date_published.to_rfc3339()),
                })
                .or_else(|| {
                    local.and_then(|m| {
                        m.version_number.as_ref().map(|version_number| {
                            ContentItemVersion {
                                id: file.hash.clone(),
                                version_number: version_number.clone(),
                                file_name: file.file_name.clone(),
                                date_published: None,
                            }
                        })
                    })
                });

            ContentItem {
                file_name: file.file_name.clone(),
                file_path: path.clone(),
                hash: file.hash.clone(),
                size: file.size,
                enabled: !file.file_name.ends_with(".disabled"),
                project_type: file.project_type,
                project,
                version,
                owner,
                has_update: file.update_version_id.is_some(),
                update_version_id: file.update_version_id.clone(),
                date_added: modification_times[i].clone(),
            }
        })
        .collect();

    items.sort_by(|a, b| {
        let name_a = a
            .project
            .as_ref()
            .map(|p| p.title.as_str())
            .unwrap_or(&a.file_name);
        let name_b = b
            .project
            .as_ref()
            .map(|p| p.title.as_str())
            .unwrap_or(&b.file_name);
        name_a
            .to_lowercase()
            .cmp(&name_b.to_lowercase())
            .then_with(|| a.file_name.cmp(&b.file_name))
    });

    Ok(items)
}

/// Resolve the owner of a project from pre-fetched teams and organizations.
fn resolve_owner(
    project: &Project,
    teams: &[Vec<TeamMember>],
    organizations: &[Organization],
) -> Option<ContentItemOwner> {
    if let Some(org_id) = &project.organization {
        organizations.iter().find(|o| &o.id == org_id).map(|o| {
            ContentItemOwner {
                id: o.id.clone(),
                name: o.name.clone(),
                avatar_url: o.icon_url.clone(),
                owner_type: OwnerType::Organization,
            }
        })
    } else {
        teams
            .iter()
            .find(|t| t.first().is_some_and(|m| m.team_id == project.team))
            .and_then(|t| t.iter().find(|m| m.is_owner))
            .map(|m| ContentItemOwner {
                id: m.user.id.clone(),
                name: m.user.username.clone(),
                avatar_url: m.user.avatar_url.clone(),
                owner_type: OwnerType::User,
            })
    }
}

/// Get content items that are part of the linked modpack (not user-added).
/// Returns modpack-bundled files with full on-disk metadata (file_path, enabled, etc).
/// Returns empty vec if the profile is not linked to a modpack.
pub async fn get_linked_modpack_content(
    profile: &Profile,
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
    let Some(linked_data) = &profile.linked_data else {
        return Ok(Vec::new());
    };

    let all_files = profile
        .get_projects(cache_behaviour, pool, fetch_semaphore)
        .await?;

    let modpack_hashes: HashSet<String> = match get_modpack_file_hashes(
        &linked_data.version_id,
        pool,
        fetch_semaphore,
    )
    .await
    {
        Ok(hashes) => hashes,
        Err(e) => {
            tracing::warn!("Failed to fetch modpack file hashes: {}", e);
            return Ok(Vec::new());
        }
    };

    // Inverse of get_content_items: keep only modpack-bundled files
    let modpack_files: Vec<(String, ProfileFile)> = all_files
        .into_iter()
        .filter(|(_, file)| modpack_hashes.contains(&file.hash))
        .collect();

    profile_files_to_content_items(
        &profile.path,
        &modpack_files,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await
}

/// Convert a list of dependencies into ContentItems with rich metadata.
/// Fetches project, version, and owner info for each dependency.
pub async fn dependencies_to_content_items(
    dependencies: &[Dependency],
    cache_behaviour: Option<CacheBehaviour>,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<Vec<ContentItem>> {
    let project_ids: HashSet<String> = dependencies
        .iter()
        .filter_map(|d| d.project_id.clone())
        .collect();

    if project_ids.is_empty() {
        return Ok(Vec::new());
    }

    let version_ids: HashSet<String> = dependencies
        .iter()
        .filter_map(|d| d.version_id.clone())
        .collect();

    let meta = resolve_metadata(
        &project_ids,
        &version_ids,
        cache_behaviour,
        pool,
        fetch_semaphore,
    )
    .await?;

    let mut items: Vec<ContentItem> = dependencies
        .iter()
        .filter_map(|dep| {
            let project_id = dep.project_id.as_ref()?;
            let project = meta.projects.iter().find(|p| &p.id == project_id)?;

            let version = dep
                .version_id
                .as_ref()
                .and_then(|vid| meta.versions.iter().find(|v| &v.id == vid));

            let owner =
                resolve_owner(project, &meta.teams, &meta.organizations);

            let project_type = match project.project_type.as_str() {
                "mod" => ProjectType::Mod,
                "resourcepack" => ProjectType::ResourcePack,
                "shader" => ProjectType::ShaderPack,
                "datapack" => ProjectType::DataPack,
                _ => ProjectType::Mod,
            };

            Some(ContentItem {
                file_name: version
                    .and_then(|v| v.files.first())
                    .map(|f| f.filename.clone())
                    .unwrap_or_else(|| {
                        format!(
                            "{}.jar",
                            project.slug.as_deref().unwrap_or(&project.id)
                        )
                    }),
                file_path: String::new(),
                hash: String::new(),
                size: version
                    .and_then(|v| v.files.first())
                    .map(|f| f.size as u64)
                    .unwrap_or(0),
                enabled: true,
                project_type,
                project: Some(ContentItemProject {
                    id: project.id.clone(),
                    slug: project.slug.clone(),
                    title: project.title.clone(),
                    icon_url: project.icon_url.clone(),
                    description: Some(project.description.clone()),
                }),
                version: version.map(|v| ContentItemVersion {
                    id: v.id.clone(),
                    version_number: v.version_number.clone(),
                    file_name: v
                        .files
                        .first()
                        .map(|f| f.filename.clone())
                        .unwrap_or_default(),
                    date_published: Some(v.date_published.to_rfc3339()),
                }),
                owner,
                has_update: false,
                update_version_id: None,
                date_added: None,
            })
        })
        .collect();

    items.sort_by(|a, b| {
        let name_a = a
            .project
            .as_ref()
            .map(|p| p.title.as_str())
            .unwrap_or(&a.file_name);
        let name_b = b
            .project
            .as_ref()
            .map(|p| p.title.as_str())
            .unwrap_or(&b.file_name);
        name_a
            .to_lowercase()
            .cmp(&name_b.to_lowercase())
            .then_with(|| a.file_name.cmp(&b.file_name))
    });

    Ok(items)
}

/// Gets SHA1 hashes of all files in a modpack version.
/// Checks cache first, falls back to downloading mrpack if not cached.
async fn get_modpack_file_hashes(
    version_id: &str,
    pool: &SqlitePool,
    fetch_semaphore: &FetchSemaphore,
) -> crate::Result<HashSet<String>> {
    if let Some(cached) =
        CachedEntry::get_modpack_files(version_id, pool, fetch_semaphore)
            .await?
    {
        tracing::info!(
            "Cache hit: {} modpack file hashes for version {}",
            cached.file_hashes.len(),
            version_id
        );
        return Ok(cached.file_hashes.into_iter().collect());
    }

    tracing::warn!(
        "Cache miss: modpack files not cached, downloading mrpack for version {}",
        version_id
    );

    let version =
        CachedEntry::get_version(version_id, None, pool, fetch_semaphore)
            .await?
            .ok_or_else(|| {
                crate::ErrorKind::InputError(format!(
                    "Modpack version {version_id} not found"
                ))
            })?;

    let primary_file = version
        .files
        .iter()
        .find(|f| f.primary)
        .or_else(|| version.files.first())
        .ok_or_else(|| {
            crate::ErrorKind::InputError(format!(
                "No files found for modpack version {version_id}"
            ))
        })?;

    let mrpack_bytes = fetch_mirrors(
        &[&primary_file.url],
        primary_file.hashes.get("sha1").map(|s| s.as_str()),
        fetch_semaphore,
        pool,
    )
    .await?;

    let reader = Cursor::new(&mrpack_bytes);
    let mut zip_reader =
        ZipFileReader::with_tokio(reader).await.map_err(|_| {
            crate::ErrorKind::InputError(
                "Failed to read modpack zip".to_string(),
            )
        })?;

    let manifest_idx = zip_reader
        .file()
        .entries()
        .iter()
        .position(|f| {
            matches!(f.filename().as_str(), Ok("modrinth.index.json"))
        })
        .ok_or_else(|| {
            crate::ErrorKind::InputError(
                "No modrinth.index.json found in mrpack".to_string(),
            )
        })?;

    let mut manifest = String::new();
    let mut entry_reader = zip_reader.reader_with_entry(manifest_idx).await?;
    entry_reader.read_to_string_checked(&mut manifest).await?;

    let pack: PackFormat = serde_json::from_str(&manifest)?;

    let mut hashes: Vec<String> = pack
        .files
        .iter()
        .filter_map(|f| f.hashes.get(&PackFileHash::Sha1).cloned())
        .collect();

    // Also hash files from overrides folders (these aren't in modrinth.index.json)
    let override_entries: Vec<usize> = zip_reader
        .file()
        .entries()
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            let filename = entry.filename().as_str().ok()?;
            let is_override = (filename.starts_with("overrides/")
                || filename.starts_with("client-overrides/")
                || filename.starts_with("server-overrides/"))
                && !filename.ends_with('/');
            is_override.then_some(index)
        })
        .collect();

    for index in override_entries {
        let mut file_bytes = Vec::new();
        let mut entry_reader = zip_reader.reader_with_entry(index).await?;
        entry_reader.read_to_end_checked(&mut file_bytes).await?;

        let hash = sha1_async(bytes::Bytes::from(file_bytes)).await?;
        hashes.push(hash);
    }

    CachedEntry::cache_modpack_files(version_id, hashes.clone(), pool).await?;

    Ok(hashes.into_iter().collect())
}
