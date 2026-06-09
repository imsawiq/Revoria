//! Authentication flow interface
use crate::event::emit::{emit_loading, init_loading};
use crate::state::JavaVersion;
use crate::util::fetch::{fetch_advanced, fetch_json};
use dashmap::DashMap;
use reqwest::Method;
use serde::Deserialize;
use std::path::PathBuf;
use sysinfo::{MemoryRefreshKind, RefreshKind};

use crate::util::io;
use crate::util::jre::extract_java_version;
use crate::{
    LoadingBarType, State,
    util::jre::{self},
};

pub async fn get_java_versions() -> crate::Result<DashMap<u32, JavaVersion>> {
    let state = State::get().await?;

    JavaVersion::get_all(&state.pool).await
}

pub async fn set_java_version(java_version: JavaVersion) -> crate::Result<()> {
    let state = State::get().await?;
    java_version.upsert(&state.pool).await?;
    Ok(())
}

// Searches for jres on the system given a java version (ex: 1.8, 1.17, 1.18)
// Allow higher allows for versions higher than the given version to be returned ('at least')
pub async fn find_filtered_jres(
    java_version: Option<u32>,
) -> crate::Result<Vec<JavaVersion>> {
    let jres = jre::get_all_jre().await?;

    // Filter out JREs that are not 1.17 or higher
    Ok(if let Some(java_version) = java_version {
        jres.into_iter()
            .filter(|jre| {
                let jre_version = extract_java_version(&jre.version);
                if let Ok(jre_version) = jre_version {
                    jre_version == java_version
                } else {
                    false
                }
            })
            .collect()
    } else {
        jres
    })
}

pub async fn auto_install_java(java_version: u32) -> crate::Result<PathBuf> {
    let state = State::get().await?;

    let loading_bar = init_loading(
        LoadingBarType::JavaDownload {
            version: java_version,
        },
        100.0,
        "Downloading java version",
    )
    .await?;

    if let Some(java) = find_filtered_jres(Some(java_version)).await?.first() {
        tracing::info!(
            "Using existing Java {} at {}",
            java.parsed_version,
            java.path
        );
        return Ok(PathBuf::from(&java.path));
    }

    #[derive(Deserialize)]
    struct Package {
        pub download_url: String,
    }

    emit_loading(&loading_bar, 0.0, Some("Fetching java version"))?;
    let packages = fetch_json::<Vec<Package>>(
                Method::GET,
                &format!(
                    "https://api.azul.com/metadata/v1/zulu/packages?arch={}&java_version={}&os={}&archive_type=zip&javafx_bundled=false&java_package_type=jre&page_size=5",
                    std::env::consts::ARCH, java_version, std::env::consts::OS
                ),
                None,
                None,
                &state.fetch_semaphore,
                &state.pool,
            ).await?;
    emit_loading(&loading_bar, 10.0, Some("Downloading java version"))?;

    if packages.is_empty() {
        return Err(crate::ErrorKind::LauncherError(format!(
            "No Java Version found for Java version {}, OS {}, and Architecture {}",
            java_version,
            std::env::consts::OS,
            std::env::consts::ARCH,
        ))
        .into());
    }

    let path = state.directories.java_versions_dir();
    let mut last_error = None;

    for (index, download) in packages.iter().enumerate() {
        let progress_message = format!(
            "Downloading java version ({}/{})",
            index + 1,
            packages.len()
        );
        emit_loading(&loading_bar, 0.0, Some(&progress_message))?;

        let file = match fetch_advanced(
            Method::GET,
            &download.download_url,
            None,
            None,
            None,
            Some((&loading_bar, 80.0)),
            &state.fetch_semaphore,
            &state.pool,
        )
        .await
        {
            Ok(file) => file,
            Err(err) => {
                tracing::warn!(
                    "Failed to download Java {} candidate {} from {}: {err}",
                    java_version,
                    index + 1,
                    download.download_url
                );
                last_error = Some(err.to_string());
                continue;
            }
        };

        let mut archive = match zip::ZipArchive::new(std::io::Cursor::new(file))
        {
            Ok(archive) => archive,
            Err(err) => {
                tracing::warn!(
                    "Failed to read Java {} candidate {} zip: {err}",
                    java_version,
                    index + 1
                );
                last_error = Some(format!("Failed to read java zip: {err}"));
                continue;
            }
        };

        let Some(archive_root) = archive
            .file_names()
            .next()
            .and_then(|file| file.split('/').next())
            .filter(|dir| !dir.is_empty())
            .map(ToOwned::to_owned)
        else {
            last_error =
                Some("Java zip did not contain a root directory".to_string());
            continue;
        };

        let install_path = path.join(&archive_root);

        if install_path.exists() {
            io::remove_dir_all(&install_path).await?;
        }

        emit_loading(&loading_bar, 0.0, Some("Extracting java"))?;
        if let Err(err) = archive.extract(&path) {
            tracing::warn!(
                "Failed to extract Java {} candidate {}: {err}",
                java_version,
                index + 1
            );
            if install_path.exists() {
                let _ = io::remove_dir_all(&install_path).await;
            }
            last_error = Some(format!("Failed to extract java zip: {err}"));
            continue;
        }

        emit_loading(&loading_bar, 10.0, Some("Done extracting java"))?;
        let mut base_path = install_path;

        #[cfg(target_os = "macos")]
        {
            base_path = base_path
                .join(format!("zulu-{java_version}.jre"))
                .join("Contents")
                .join("Home")
                .join("bin")
                .join("java")
        }

        #[cfg(not(target_os = "macos"))]
        {
            base_path = base_path.join("bin").join(jre::JAVA_BIN)
        }

        return Ok(base_path);
    }

    Err(crate::ErrorKind::LauncherError(format!(
        "Failed to download Java version {java_version} from Azul after {} candidates{}",
        packages.len(),
        last_error
            .map(|err| format!("; last error: {err}"))
            .unwrap_or_default()
    ))
    .into())
}

// Validates JRE at a given at a given path
pub async fn check_jre(path: PathBuf) -> crate::Result<JavaVersion> {
    jre::check_java_at_filepath(&path).await
}

// Test JRE at a given path
pub async fn test_jre(
    path: PathBuf,
    major_version: u32,
) -> crate::Result<bool> {
    let jre = match jre::check_java_at_filepath(&path).await {
        Ok(jre) => jre,
        Err(e) => {
            tracing::warn!("Invalid Java at {}: {e}", path.display());
            return Ok(false);
        }
    };
    let version = extract_java_version(&jre.version)?;
    tracing::info!(
        "Expected Java version {major_version}, and found {version} at {}",
        path.display()
    );
    Ok(version == major_version)
}

// Gets maximum memory in KiB.
pub async fn get_max_memory() -> crate::Result<u64> {
    Ok(sysinfo::System::new_with_specifics(
        RefreshKind::nothing()
            .with_memory(MemoryRefreshKind::nothing().with_ram()),
    )
    .total_memory()
        / 1024)
}
