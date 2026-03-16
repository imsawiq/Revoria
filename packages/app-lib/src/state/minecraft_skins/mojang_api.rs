use std::{error::Error, sync::Arc, time::Instant, str::FromStr};

use bytes::Bytes;
use futures::TryStream;
use reqwest::{Body, multipart::Part};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use super::MinecraftSkinVariant;
use crate::{
    ErrorKind,
    data::Credentials,
    state::{MinecraftProfile, PROFILE_CACHE, ProfileCacheEntry},
    util::fetch::REQWEST_CLIENT,
};

/// Provides operations for interacting with capes on a Minecraft player profile.
pub struct MinecraftCapeOperation;

impl MinecraftCapeOperation {
    pub async fn equip(
        credentials: &Credentials,
        cape_id: Uuid,
    ) -> crate::Result<()> {
        update_profile_cache_from_response(
            REQWEST_CLIENT
                .put("https://api.minecraftservices.com/minecraft/profile/capes/active")
                .header("Content-Type", "application/json; charset=utf-8")
                .header("Accept", "application/json")
                .bearer_auth(&credentials.access_token)
                .json(&json!({
                    "capeId": cape_id.hyphenated(),
                }))
                .send()
                .await
                .and_then(|response| response.error_for_status())?
        )
        .await;

        Ok(())
    }

    pub async fn unequip_any(credentials: &Credentials) -> crate::Result<()> {
        update_profile_cache_from_response(
            REQWEST_CLIENT
                .delete("https://api.minecraftservices.com/minecraft/profile/capes/active")
                .header("Accept", "application/json")
                .bearer_auth(&credentials.access_token)
                .send()
                .await
                .and_then(|response| response.error_for_status())?
        )
        .await;

        Ok(())
    }
}

/// Provides operations for interacting with skins on a Minecraft player profile.
pub struct MinecraftSkinOperation;

impl MinecraftSkinOperation {
    pub async fn equip<TextureStream>(
        credentials: &Credentials,
        texture: TextureStream,
        variant: MinecraftSkinVariant,
    ) -> crate::Result<()>
    where
        TextureStream: TryStream + Send + 'static,
        TextureStream::Error: Into<Box<dyn Error + Send + Sync>>,
        Bytes: From<TextureStream::Ok>,
    {
        let form = reqwest::multipart::Form::new()
            .text(
                "variant",
                match variant {
                    MinecraftSkinVariant::Slim => "slim",
                    MinecraftSkinVariant::Classic => "classic",
                    _ => {
                        return Err(ErrorKind::OtherError(
                            "Cannot equip skin of unknown model variant".into(),
                        )
                        .into());
                    }
                },
            )
            .part(
                "file",
                Part::stream(Body::wrap_stream(texture))
                    .mime_str("image/png")?
                    .file_name("skin.png"),
            );

        update_profile_cache_from_response(
            REQWEST_CLIENT
                .post(
                    "https://api.minecraftservices.com/minecraft/profile/skins",
                )
                .header("Accept", "application/json")
                .bearer_auth(&credentials.access_token)
                .multipart(form)
                .send()
                .await
                .and_then(|response| response.error_for_status())?,
        )
        .await;

        Ok(())
    }

    pub async fn unequip_any(credentials: &Credentials) -> crate::Result<()> {
        update_profile_cache_from_response(
            REQWEST_CLIENT
                .delete("https://api.minecraftservices.com/minecraft/profile/skins/active")
                .header("Accept", "application/json")
                .bearer_auth(&credentials.access_token)
                .send()
                .await
                .and_then(|response| response.error_for_status())?
        )
        .await;

        Ok(())
    }
}

async fn update_profile_cache_from_response(response: reqwest::Response) {
    let Some(mut profile) = response.json::<MinecraftProfile>().await.ok()
    else {
        tracing::warn!(
            "Failed to parse player profile from skin or cape operation response, not updating profile cache"
        );
        return;
    };

    profile.fetch_time = Some(Instant::now());

    PROFILE_CACHE
        .lock()
        .await
        .insert(profile.id, ProfileCacheEntry::Hit(Arc::new(profile)));
}

#[derive(Debug, Clone, Deserialize)]
pub struct SkinHistoryEntry {
    pub id: Uuid,
    pub url: String,
    pub variant: String,
    pub alias: Option<String>,
}

pub async fn get_skin_history(
    credentials: &Credentials,
) -> crate::Result<Vec<SkinHistoryEntry>> {
    // First, get the current profile to get the username and UUID
    let profile_response = REQWEST_CLIENT
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Accept", "application/json")
        .bearer_auth(&credentials.access_token)
        .send()
        .await
        .and_then(|response| response.error_for_status())?;

    #[derive(Debug, Deserialize)]
    struct ProfileResponse {
        name: String,
        id: String,
    }

    let profile: ProfileResponse = profile_response.json().await?;
    let username = profile.name;
    let uuid = profile.id;

    tracing::info!("Fetching skin history for user: {} ({})", username, uuid);

    // Use ashcon.app API to get skin history (free, no auth required)
    // This API returns full skin history for a Minecraft user
    let ashcon_url = format!(
        "https://api.ashcon.app/mojang/v2/skins/{}",
        uuid.replace("-", "")
    );
    tracing::info!("Fetching ashcon API: {}", ashcon_url);

    let response = REQWEST_CLIENT
        .get(&ashcon_url)
        .header("Accept", "application/json")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            tracing::info!("Ashcon API response status: {}", status);

            if status.is_success() {
                if let Ok(history_response) =
                    resp.json::<AshconSkinResponse>().await
                {
                    tracing::info!(
                        "Successfully fetched {} skins from ashcon API",
                        history_response.skins.len()
                    );

                    // Convert ashcon response to our SkinHistoryEntry format
                    let skins = history_response
                        .skins
                        .into_iter()
                        .filter_map(|skin| {
                            // Parse the skin UUID from the texture ID
                            let id = uuid::Uuid::from_str(&skin.texture_id)
                                .unwrap_or_else(|_| {
                                    generate_uuid_from_url(&skin.url)
                                });

                            Some(SkinHistoryEntry {
                                id,
                                url: skin.url,
                                variant: skin
                                    .model
                                    .unwrap_or_else(|| "classic".to_string()),
                                alias: None,
                            })
                        })
                        .collect::<Vec<_>>();

                    if !skins.is_empty() {
                        return Ok(skins);
                    }
                }
            } else {
                tracing::warn!(
                    "Ashcon API returned non-success status: {}",
                    status
                );
                if let Ok(text) = resp.text().await {
                    tracing::warn!("Ashcon API response body: {}", text);
                }
            }
        }
        Err(e) => {
            tracing::warn!("Failed to fetch ashcon API: {:?}", e);
        }
    }

    // Try mineatar.io API as fallback
    let mineatar_url =
        format!("https://api.mineatar.io/skins/{}", uuid.replace("-", ""));
    tracing::info!("Fetching mineatar API: {}", mineatar_url);

    let response = REQWEST_CLIENT
        .get(&mineatar_url)
        .header("Accept", "application/json")
        .send()
        .await;

    if let Ok(resp) = response {
        if resp.status().is_success() {
            if let Ok(history_response) =
                resp.json::<MineatarSkinResponse>().await
            {
                tracing::info!(
                    "Successfully fetched {} skins from mineatar API",
                    history_response.skins.len()
                );

                let skins = history_response
                    .skins
                    .into_iter()
                    .filter_map(|skin| {
                        let id = uuid::Uuid::from_str(&skin.id).unwrap_or_else(
                            |_| generate_uuid_from_url(&skin.url),
                        );

                        Some(SkinHistoryEntry {
                            id,
                            url: skin.url,
                            variant: skin
                                .variant
                                .unwrap_or_else(|| "classic".to_string()),
                            alias: None,
                        })
                    })
                    .collect::<Vec<_>>();

                if !skins.is_empty() {
                    return Ok(skins);
                }
            }
        }
    }

    // Fallback to Mojang API (returns all skins associated with account)
    tracing::info!("Falling back to Mojang API for skin history");
    let response = REQWEST_CLIENT
        .get("https://api.minecraftservices.com/minecraft/profile/skins")
        .header("Accept", "application/json")
        .bearer_auth(&credentials.access_token)
        .send()
        .await
        .and_then(|response| response.error_for_status())?;

    #[derive(Debug, Deserialize)]
    struct MojangSkinHistoryResponse {
        skins: Vec<SkinHistoryEntry>,
    }

    let history_response: MojangSkinHistoryResponse = response.json().await?;
    tracing::info!(
        "Successfully fetched {} skins from Mojang",
        history_response.skins.len()
    );
    Ok(history_response.skins)
}

/// Response from ashcon.app skin API
#[derive(Debug, Clone, Deserialize, Serialize)]
struct AshconSkinResponse {
    skins: Vec<AshconSkin>,
}

/// Single skin entry from ashcon.app API
#[derive(Debug, Clone, Deserialize, Serialize)]
struct AshconSkin {
    url: String,
    #[serde(rename = "textureId")]
    texture_id: String,
    #[serde(rename = "model")]
    model: Option<String>,
}

/// Response from mineatar.io skin API
#[derive(Debug, Clone, Deserialize, Serialize)]
struct MineatarSkinResponse {
    skins: Vec<MineatarSkin>,
}

/// Single skin entry from mineatar.io API
#[derive(Debug, Clone, Deserialize, Serialize)]
struct MineatarSkin {
    id: String,
    url: String,
    variant: Option<String>,
}

/// Generates a UUID from a URL string for identification
fn generate_uuid_from_url(url: &str) -> Uuid {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();

    // Create a UUID from the hash
    Uuid::from_u64_pair(hash, hash ^ 0x1234567890ABCDEF)
}
