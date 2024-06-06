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
        .map_err(|e| worker::Error::RustError(format!("Failed to connect: {:?}", e)))?;

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
