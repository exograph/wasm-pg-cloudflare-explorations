use worker::*;

#[event(fetch)]
async fn main(_req: Request, _env: Env, _ctx: Context) -> Result<Response> {
    Ok(Response::ok("Hello, Cloudflare!")?)
}
