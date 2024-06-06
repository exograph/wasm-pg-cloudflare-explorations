Code accompanying the blog post "Latency at the Edge with Rust/WebAssembly and Postgres" [Part 1](https://exograph.dev/blog/wasm-pg-explorations-1) and [Part 2](https://exograph.dev/blog/wasm-pg-explorations-2).

## Common setup

Visit [Neon](https://neon.tech) and create a database, say, `todo-db`, with the following table:

```sql
CREATE TABLE todos (
  id SERIAL PRIMARY KEY,
  title TEXT NOT NULL,
  completed BOOLEAN NOT NULL
);
```

## Creating a Hyperdrive

You need to create a Hyperdrive instance only once but update `wrangler.toml` for each project.

Replace the connection string with the one from the Neon dashboard.

```sh
npx wrangler hyperdrive create todo-db-hyperdrive --caching-disabled --connection-string "postgres://alex:AbC123dEf@ep-cool-darkness-123456.us-east-2.aws.neon.tech/dbname"
```

Add the following to `wrangler.toml` in each `rust-hyperdrive-*` project:

```toml
[[hyperdrive]]
binding = "todo-db-hyperdrive"
id = "<your-hyperdrive-id>"
```

## Setting the DATABASE_URL secret

For all `rust-direct-*` projects, set the following environment variables. Alternatively, you can use the "Integration" tab in the Cloudflare Worker's dashboard to add the Neon integration, which does the same thing.

```sh
npx wrangler secret put DATABASE_URL
```

This will ask for the connection URL, which you can obtain from the Neon dashboard.

## Deploying

```sh
npx wrangler deploy
```

## Testing performance

Install [oha](https://github.com/hatoo/oha) using `cargo install oha`.

```sh
oha -c 1 -n 100 <your-worker-url>
```

Of course, you can adjust the concurrency and number of requests as needed.
