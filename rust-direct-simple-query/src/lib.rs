use std::str::FromStr;

use tokio_postgres::config::{Config, Host};
use worker::{postgres_tls::PassthroughTls, *};

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let config = Config::from_str(&env.secret("DATABASE_URL")?.to_string())
        .map_err(|e| worker::Error::RustError(format!("Failed to parse configuration: {:?}", e)))?;

    let host = match &config.get_hosts()[0] {
        Host::Tcp(host) => host,
        #[allow(unreachable_patterns)]
        _ => {
            return Err(worker::Error::RustError("Could not parse host".to_string()));
        }
    };
    let port = config.get_ports()[0];

    let socket = Socket::builder()
        .secure_transport(SecureTransport::StartTls)
        .connect(host, port)?;

    let (client, connection) = config
        .connect_raw(socket, PassthroughTls)
        .await
        .map_err(|e| worker::Error::RustError(format!("Failed to connect: {:?}", e)))?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = connection.await {
            console_log!("connection error: {:?}", error);
        }
    });

    let rows = client
        .simple_query("SELECT id, title, completed FROM todos WHERE completed = true")
        .await
        .map_err(|e| worker::Error::RustError(format!("Failed to query: {:?}", e)))?;

    Ok(Response::ok(format!("{:?}", rows))?)
}
