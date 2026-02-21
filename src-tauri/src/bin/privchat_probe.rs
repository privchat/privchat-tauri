use privchat_sdk_ffi::{PrivchatClient, PrivchatConfig, ServerEndpoint, TransportProtocol};

fn parse_server(url: &str) -> anyhow::Result<ServerEndpoint> {
    let (protocol, use_tls, rest) = if let Some(v) = url.strip_prefix("tcp://") {
        (TransportProtocol::Tcp, false, v)
    } else if let Some(v) = url.strip_prefix("quic://") {
        (TransportProtocol::Quic, false, v)
    } else if let Some(v) = url.strip_prefix("ws://") {
        (TransportProtocol::WebSocket, false, v)
    } else if let Some(v) = url.strip_prefix("wss://") {
        (TransportProtocol::WebSocket, true, v)
    } else {
        anyhow::bail!("unsupported url: {url}");
    };

    let (host_port, path) = if let Some(i) = rest.find('/') {
        (&rest[..i], Some(rest[i..].to_string()))
    } else {
        (rest, None)
    };

    let (host, port) = if let Some(i) = host_port.rfind(':') {
        let host = host_port[..i].to_string();
        let port = host_port[i + 1..].parse::<u16>()?;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = std::env::var("PRIVCHAT_SERVER_URL").unwrap_or_else(|_| "tcp://127.0.0.1:9001".to_string());
    let username = std::env::var("PRIVCHAT_USERNAME").ok();
    let password = std::env::var("PRIVCHAT_PASSWORD").ok();
    let data_dir = std::env::var("PRIVCHAT_DATA_DIR").unwrap_or_else(|_| "/tmp/privchat-probe".to_string());

    let cfg = PrivchatConfig {
        endpoints: vec![parse_server(&server)?],
        connection_timeout_secs: 10,
        data_dir,
    };

    let client = PrivchatClient::new(cfg).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    client.connect().await.map_err(|e| anyhow::anyhow!(e.to_string()))?;
    println!("[probe] connect ok: {}", server);

    if let (Some(u), Some(p)) = (username, password) {
        let device_id = uuid::Uuid::new_v4().to_string();
        let login = client
            .login(u.clone(), p, device_id.clone())
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        client
            .authenticate(login.user_id, login.token, device_id)
            .await
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        println!("[probe] login+auth ok: user={}", u);
    } else {
        println!("[probe] skipped login (set PRIVCHAT_USERNAME/PRIVCHAT_PASSWORD to test auth)");
    }

    Ok(())
}
