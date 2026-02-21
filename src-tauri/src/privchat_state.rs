use anyhow::{anyhow, Context, Result};
use privchat_sdk_ffi::{PrivchatClient, PrivchatConfig, ServerEndpoint, TransportProtocol};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SavedSession {
    pub user_id: u64,
    pub token: String,
    pub device_id: String,
    pub username: Option<String>,
}

pub struct PrivchatAppState {
    pub data_dir: PathBuf,
    pub auth_file: PathBuf,
    pub server_url: String,
    pub client: Mutex<Option<Arc<PrivchatClient>>>,
}

impl PrivchatAppState {
    pub fn new(data_dir: PathBuf, server_url: String) -> Self {
        let auth_file = data_dir.join("privchat_auth.json");
        Self {
            data_dir,
            auth_file,
            server_url,
            client: Mutex::new(None),
        }
    }

    pub async fn get_or_create_client(&self) -> Result<Arc<PrivchatClient>> {
        let mut guard = self.client.lock().await;
        if let Some(client) = guard.clone() {
            return Ok(client);
        }
        let endpoint = parse_server_endpoint(&self.server_url)?;
        let cfg = PrivchatConfig {
            endpoints: vec![endpoint],
            connection_timeout_secs: 10,
            data_dir: self.data_dir.to_string_lossy().to_string(),
        };
        let client = Arc::new(PrivchatClient::new(cfg).map_err(|e| anyhow!(e.to_string()))?);
        *guard = Some(client.clone());
        Ok(client)
    }

    pub async fn clear_client(&self) {
        let mut guard = self.client.lock().await;
        *guard = None;
    }

    pub fn load_session(&self) -> Result<Option<SavedSession>> {
        read_json_file(&self.auth_file)
    }

    pub fn save_session(&self, session: &SavedSession) -> Result<()> {
        if let Some(parent) = self.auth_file.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create auth parent dir {}", parent.display())
            })?;
        }
        let bytes = serde_json::to_vec_pretty(session)?;
        std::fs::write(&self.auth_file, bytes)
            .with_context(|| format!("failed to write auth file {}", self.auth_file.display()))
    }

    pub fn clear_session(&self) -> Result<()> {
        if self.auth_file.exists() {
            std::fs::remove_file(&self.auth_file)
                .with_context(|| format!("failed to remove auth file {}", self.auth_file.display()))?;
        }
        Ok(())
    }
}

fn read_json_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>> {
    if !path.exists() {
        return Ok(None);
    }
    let content =
        std::fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if content.trim().is_empty() {
        return Ok(None);
    }
    let parsed = serde_json::from_str::<T>(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;
    Ok(Some(parsed))
}

fn parse_server_endpoint(url: &str) -> Result<ServerEndpoint> {
    let (protocol, use_tls, rest) = if let Some(v) = url.strip_prefix("tcp://") {
        (TransportProtocol::Tcp, false, v)
    } else if let Some(v) = url.strip_prefix("quic://") {
        (TransportProtocol::Quic, false, v)
    } else if let Some(v) = url.strip_prefix("ws://") {
        (TransportProtocol::WebSocket, false, v)
    } else if let Some(v) = url.strip_prefix("wss://") {
        (TransportProtocol::WebSocket, true, v)
    } else {
        return Err(anyhow!("unsupported server url scheme: {}", url));
    };

    let (host_port, path) = if let Some(slash_idx) = rest.find('/') {
        (&rest[..slash_idx], Some(rest[slash_idx..].to_string()))
    } else {
        (rest, None)
    };

    let (host, port) = if let Some(colon_idx) = host_port.rfind(':') {
        let host = host_port[..colon_idx].to_string();
        let port = host_port[colon_idx + 1..]
            .parse::<u16>()
            .map_err(|e| anyhow!("invalid server port in {}: {}", url, e))?;
        (host, port)
    } else {
        (host_port.to_string(), 9001)
    };

    Ok(ServerEndpoint {
        protocol,
        host,
        port,
        path,
        use_tls,
    })
}
