//! Functions for fetching information from the Internet
use super::io::{self, IOError};
use crate::ErrorKind;
use crate::LAUNCHER_USER_AGENT;
use crate::event::LoadingBarId;
use crate::event::emit::{check_loading_cancelled, emit_loading};
use crate::state::{ProxyType, Settings};
use bytes::Bytes;
use reqwest::{IntoUrl, Method};
use serde::de::DeserializeOwned;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, RwLock};
use std::time::{self, Duration};
use tokio::sync::Semaphore;
use tokio::{fs::File, io::AsyncWriteExt};
use url::Url;

#[derive(Debug)]
pub struct IoSemaphore(pub Semaphore);
#[derive(Debug)]
pub struct FetchSemaphore(pub Semaphore);

#[derive(Debug)]
pub struct ReqwestClientHandle(RwLock<reqwest::Client>);

impl ReqwestClientHandle {
    fn new() -> Self {
        Self(RwLock::new(
            build_reqwest_client(None).expect("Reqwest Client Building Failed"),
        ))
    }

    pub fn client(&self) -> reqwest::Client {
        self.0.read().expect("Reqwest client poisoned").clone()
    }

    pub fn replace(&self, client: reqwest::Client) {
        *self.0.write().expect("Reqwest client poisoned") = client;
    }

    pub fn request<U: IntoUrl>(
        &self,
        method: Method,
        url: U,
    ) -> reqwest::RequestBuilder {
        self.client().request(method, url)
    }

    pub fn get<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client().get(url)
    }

    pub fn post<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client().post(url)
    }

    pub fn put<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client().put(url)
    }

    pub fn delete<U: IntoUrl>(&self, url: U) -> reqwest::RequestBuilder {
        self.client().delete(url)
    }
}

pub static REQWEST_CLIENT: LazyLock<ReqwestClientHandle> =
    LazyLock::new(ReqwestClientHandle::new);

fn build_reqwest_client(
    proxy_url: Option<&str>,
) -> crate::Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    let header =
        reqwest::header::HeaderValue::from_str(LAUNCHER_USER_AGENT).unwrap();
    headers.insert(reqwest::header::USER_AGENT, header);

    let mut builder = reqwest::Client::builder()
        .tcp_keepalive(Some(time::Duration::from_secs(10)))
        .connect_timeout(Duration::from_secs(20))
        .read_timeout(Duration::from_secs(45))
        .default_headers(headers);

    if let Some(proxy_url) = proxy_url {
        let proxy = reqwest::Proxy::all(proxy_url).map_err(|err| {
            ErrorKind::OtherError(format!("Failed to configure proxy: {err}"))
        })?;
        builder = builder.proxy(proxy);
    }

    builder.build().map_err(|err| {
        ErrorKind::OtherError(format!("Reqwest client build failed: {err}"))
            .into()
    })
}

pub fn proxy_url_from_settings(
    settings: &Settings,
) -> crate::Result<Option<String>> {
    if !settings.proxy_enabled {
        return Ok(None);
    }

    let host = settings.proxy_host.trim();
    if host.is_empty() {
        return Err(ErrorKind::OtherError(
            "Proxy host cannot be empty while proxy is enabled".to_string(),
        )
        .into());
    }

    let scheme = match settings.proxy_type {
        ProxyType::Http => "http",
        ProxyType::Https => "https",
        ProxyType::Socks5 => "socks5h",
    };

    let mut url =
        Url::parse(&format!("{scheme}://{host}:{}", settings.proxy_port))
            .map_err(|err| {
                ErrorKind::OtherError(format!(
                    "Invalid proxy configuration: {err}"
                ))
            })?;

    if settings.proxy_auth_enabled && !settings.proxy_username.is_empty() {
        url.set_username(&settings.proxy_username).map_err(|_| {
            ErrorKind::OtherError(
                "Invalid proxy username for URL encoding".to_string(),
            )
        })?;
        if !settings.proxy_password.is_empty() {
            url.set_password(Some(&settings.proxy_password))
                .map_err(|_| {
                    ErrorKind::OtherError(
                        "Invalid proxy password for URL encoding".to_string(),
                    )
                })?;
        }
    }

    Ok(Some(url.to_string()))
}

pub fn build_reqwest_client_from_settings(
    settings: &Settings,
) -> crate::Result<reqwest::Client> {
    let proxy_url = proxy_url_from_settings(settings)?;
    build_reqwest_client(proxy_url.as_deref())
}

pub fn configure_reqwest_client_from_settings(
    settings: &Settings,
) -> crate::Result<()> {
    REQWEST_CLIENT.replace(build_reqwest_client_from_settings(settings)?);
    Ok(())
}

const FETCH_ATTEMPTS: usize = 5;

#[tracing::instrument(skip(semaphore))]
pub async fn fetch(
    url: &str,
    sha1: Option<&str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<Bytes> {
    fetch_advanced(Method::GET, url, sha1, None, None, None, semaphore, exec)
        .await
}

#[tracing::instrument(skip(json_body, semaphore))]
pub async fn fetch_json<T>(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let result = fetch_advanced(
        method, url, sha1, json_body, None, None, semaphore, exec,
    )
    .await?;
    let value = serde_json::from_slice(&result)?;
    Ok(value)
}

/// Downloads a file with retry and checksum functionality
#[tracing::instrument(skip(json_body, semaphore))]
#[allow(clippy::too_many_arguments)]
pub async fn fetch_advanced(
    method: Method,
    url: &str,
    sha1: Option<&str>,
    json_body: Option<serde_json::Value>,
    header: Option<(&str, &str)>,
    loading_bar: Option<(&LoadingBarId, f64)>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<Bytes> {
    let _permit = semaphore.0.acquire().await?;

    let creds = if header
        .as_ref()
        .is_none_or(|x| &*x.0.to_lowercase() != "authorization")
        && (url.starts_with("https://cdn.modrinth.com")
            || url.starts_with(env!("MODRINTH_API_URL"))
            || url.starts_with(env!("MODRINTH_API_URL_V3")))
    {
        crate::state::ModrinthCredentials::get_active(exec).await?
    } else {
        None
    };

    let total_attempts = FETCH_ATTEMPTS + 1;
    let mut last_error = None;

    for attempt in 1..=total_attempts {
        let mut req = REQWEST_CLIENT.request(method.clone(), url);

        if let Some(body) = json_body.clone() {
            req = req.json(&body);
        }

        if let Some(header) = header {
            req = req.header(header.0, header.1);
        }

        if let Some(ref creds) = creds {
            req = req.header("Authorization", &creds.session);
        }

        let result = req.send().await;
        match result {
            Ok(resp) => {
                if resp.status().is_server_error() && attempt <= FETCH_ATTEMPTS
                {
                    last_error =
                        Some(format!("server returned {}", resp.status()));
                    wait_before_retry(
                        url,
                        attempt,
                        total_attempts,
                        last_error.as_deref(),
                    )
                    .await;
                    continue;
                }
                if resp.status().is_client_error()
                    || resp.status().is_server_error()
                {
                    let backup_error = resp.error_for_status_ref().unwrap_err();
                    if let Ok(error) = resp.json().await {
                        return Err(ErrorKind::LabrinthError(error).into());
                    }
                    return Err(backup_error.into());
                }

                let bytes = if let Some((bar, total)) = &loading_bar {
                    let length = resp.content_length();
                    if let Some(total_size) = length {
                        use futures::StreamExt;
                        let mut stream = resp.bytes_stream();
                        let mut bytes = Vec::new();
                        while let Some(item) = stream.next().await {
                            check_loading_cancelled(bar)?;
                            let chunk = item?;
                            bytes.append(&mut chunk.to_vec());
                            emit_loading(
                                bar,
                                (chunk.len() as f64 / total_size as f64)
                                    * total,
                                None,
                            )?;
                        }

                        Ok(bytes::Bytes::from(bytes))
                    } else {
                        resp.bytes().await
                    }
                } else {
                    resp.bytes().await
                };

                if let Ok(bytes) = bytes {
                    if let Some(sha1) = sha1 {
                        let hash = sha1_async(bytes.clone()).await?;
                        if &*hash != sha1 {
                            if attempt <= FETCH_ATTEMPTS {
                                last_error = Some(format!(
                                    "sha1 mismatch: expected {sha1}, got {hash}"
                                ));
                                wait_before_retry(
                                    url,
                                    attempt,
                                    total_attempts,
                                    last_error.as_deref(),
                                )
                                .await;
                                continue;
                            } else {
                                return Err(ErrorKind::HashError(
                                    sha1.to_string(),
                                    hash,
                                )
                                .into());
                            }
                        }
                    }

                    tracing::trace!("Done downloading URL {url}");
                    return Ok(bytes);
                } else if attempt <= FETCH_ATTEMPTS {
                    if let Err(err) = bytes {
                        last_error = Some(err.to_string());
                        wait_before_retry(
                            url,
                            attempt,
                            total_attempts,
                            last_error.as_deref(),
                        )
                        .await;
                    }
                    continue;
                } else if let Err(err) = bytes {
                    return Err(err.into());
                }
            }
            Err(err) if attempt <= FETCH_ATTEMPTS => {
                last_error = Some(err.to_string());
                wait_before_retry(
                    url,
                    attempt,
                    total_attempts,
                    last_error.as_deref(),
                )
                .await;
                continue;
            }
            Err(err) => {
                return Err(err.into());
            }
        }
    }

    Err(ErrorKind::NoValueFor(format!(
        "fetch bytes from {url} after {total_attempts} attempts{}",
        last_error
            .map(|err| format!("; last error: {err}"))
            .unwrap_or_default()
    ))
    .into())
}

async fn wait_before_retry(
    url: &str,
    attempt: usize,
    total_attempts: usize,
    reason: Option<&str>,
) {
    tracing::warn!(
        "Fetch attempt {attempt}/{total_attempts} failed for {url}: {}",
        reason.unwrap_or("unknown error")
    );

    tokio::time::sleep(Duration::from_millis(250 * attempt as u64)).await;
}

/// Downloads a file from specified mirrors
#[tracing::instrument(skip(semaphore))]
pub async fn fetch_mirrors(
    mirrors: &[&str],
    sha1: Option<&str>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<Bytes> {
    fetch_mirrors_with_loading_bar(mirrors, sha1, None, semaphore, exec).await
}

#[tracing::instrument(skip(semaphore, loading_bar))]
pub async fn fetch_mirrors_with_loading_bar(
    mirrors: &[&str],
    sha1: Option<&str>,
    loading_bar: Option<&LoadingBarId>,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite> + Copy,
) -> crate::Result<Bytes> {
    if mirrors.is_empty() {
        return Err(
            ErrorKind::InputError("No mirrors provided!".to_string()).into()
        );
    }

    for (index, mirror) in mirrors.iter().enumerate() {
        if let Some(loading_bar) = loading_bar {
            check_loading_cancelled(loading_bar)?;
        }

        let result = fetch_advanced(
            Method::GET,
            mirror,
            sha1,
            None,
            None,
            loading_bar.map(|bar| (bar, 0.0)),
            semaphore,
            exec,
        )
        .await;

        if result.is_ok() || (result.is_err() && index == (mirrors.len() - 1)) {
            return result;
        }
    }

    unreachable!()
}

/// Posts a JSON to a URL
#[tracing::instrument(skip(json_body, semaphore))]
pub async fn post_json<T>(
    url: &str,
    json_body: serde_json::Value,
    semaphore: &FetchSemaphore,
    exec: impl sqlx::Executor<'_, Database = sqlx::Sqlite>,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let _permit = semaphore.0.acquire().await?;

    let mut req = REQWEST_CLIENT.post(url).json(&json_body);

    if let Some(creds) =
        crate::state::ModrinthCredentials::get_active(exec).await?
    {
        req = req.header("Authorization", &creds.session);
    }

    let result = req.send().await?.error_for_status()?;

    let value = result.json().await?;
    Ok(value)
}

pub async fn read_json<T>(
    path: &Path,
    semaphore: &IoSemaphore,
) -> crate::Result<T>
where
    T: DeserializeOwned,
{
    let _permit = semaphore.0.acquire().await?;

    let json = io::read(path).await?;
    let json = serde_json::from_slice::<T>(&json)?;

    Ok(json)
}

#[tracing::instrument(skip(bytes, semaphore))]
pub async fn write(
    path: &Path,
    bytes: &[u8],
    semaphore: &IoSemaphore,
) -> crate::Result<()> {
    let _permit = semaphore.0.acquire().await?;

    if let Some(parent) = path.parent() {
        io::create_dir_all(parent).await?;
    }

    let mut file = File::create(path)
        .await
        .map_err(|e| IOError::with_path(e, path))?;
    file.write_all(bytes)
        .await
        .map_err(|e| IOError::with_path(e, path))?;
    tracing::trace!("Done writing file {}", path.display());
    Ok(())
}

pub async fn copy(
    src: impl AsRef<Path>,
    dest: impl AsRef<Path>,
    semaphore: &IoSemaphore,
) -> crate::Result<()> {
    let src: &Path = src.as_ref();
    let dest = dest.as_ref();

    let _permit = semaphore.0.acquire().await?;

    if let Some(parent) = dest.parent() {
        io::create_dir_all(parent).await?;
    }

    io::copy(src, dest).await?;
    tracing::trace!(
        "Done copying file {} to {}",
        src.display(),
        dest.display()
    );
    Ok(())
}

// Writes a icon to the cache and returns the absolute path of the icon within the cache directory
#[tracing::instrument(skip(bytes, semaphore))]
pub async fn write_cached_icon(
    icon_path: &str,
    cache_dir: &Path,
    bytes: Bytes,
    semaphore: &IoSemaphore,
) -> crate::Result<PathBuf> {
    let extension = Path::new(&icon_path).extension().and_then(OsStr::to_str);
    let hash = sha1_async(bytes.clone()).await?;
    let path = cache_dir.join("icons").join(if let Some(ext) = extension {
        format!("{hash}.{ext}")
    } else {
        hash
    });

    write(&path, &bytes, semaphore).await?;

    let path = io::canonicalize(path)?;
    Ok(path)
}

pub async fn sha1_async(bytes: Bytes) -> crate::Result<String> {
    let hash = tokio::task::spawn_blocking(move || {
        sha1_smol::Sha1::from(bytes).hexdigest()
    })
    .await?;

    Ok(hash)
}
