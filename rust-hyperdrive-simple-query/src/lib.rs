use tokio_postgres::SimpleQueryMessage;
use worker::{postgres_tls::PassthroughTls, *};

#[event(fetch)]
async fn main(_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let hyperdrive = env.hyperdrive("todo-db-hyperdrive")?;

    let config = hyperdrive
        .connection_string()
        .parse::<tokio_postgres::Config>()
        .map_err(|e| worker::Error::RustError(format!("Failed to parse configuration: {:?}", e)))?;

    let host = hyperdrive.host();
    let port = hyperdrive.port();

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

    let rows: Vec<SimpleQueryMessage> = client
        .simple_query("SELECT id, title, completed FROM todos where completed = true")
        .await
        .map_err(|e| worker::Error::RustError(format!("tokio-postgres: {:?}", e)))?;

    let mapped: Vec<_> = rows
        .into_iter()
        .flat_map(|row| match row {
            SimpleQueryMessage::Row(row) => {
                let id = row.get(0).map(str::to_string);
                let title = row.get(1).map(str::to_string);
                let completed = row.get(2).map(str::to_string);
                Some((id, title, completed))
            }
            _ => None,
        })
        .collect();

    Ok(Response::ok(format!("{mapped:?}"))?)
}
