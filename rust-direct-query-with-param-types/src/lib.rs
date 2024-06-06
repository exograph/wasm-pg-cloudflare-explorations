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
        .map_err(|e| worker::Error::RustError(format!("tokio-postgres: {:?}", e)))?;

    wasm_bindgen_futures::spawn_local(async move {
        if let Err(error) = connection.await {
            console_log!("connection error: {:?}", error);
        }
    });

    use tokio_postgres::types::Type;

    let rows: Vec<tokio_postgres::Row> = client
        .query_with_param_types(
            "SELECT id, title, completed FROM todos where completed <> $1",
            &[(&true, Type::BOOL)],
        )
        .await
        .map_err(|e| worker::Error::RustError(format!("query_with_param_types: {:?}", e)))?;

    let mapped: Vec<_> = rows
        .into_iter()
        .map(|row| {
            let id = row.get::<_, i32>(0);
            let title = row.get::<_, &str>(1).to_string();
            let completed = row.get::<_, bool>(2);
            (id, title, completed)
        })
        .collect();

    Ok(Response::ok(format!("{mapped:?}"))?)
}
