# ShredLink – Secure Text to Link

Production-grade web app to share sensitive text via short, secure links. Text is encrypted at rest (AES-256-GCM), optional password protection (Argon2), and configurable expiration (time-based, one-time view, one-time password).

## Tech stack

- **Backend:** Rust, Actix-Web, Tokio
- **Database:** SQLite (sqlx), designed so it can be swapped to PostgreSQL later
- **Frontend:** HTML, Tailwind CSS, Vanilla JS
- **Security:** Argon2 (passwords), AES-256-GCM (text), nanoid (tokens)

## Local development

### Prerequisites

- Rust (e.g. `rustup default stable`)
- Optional: create `data` directory for SQLite file if not using default path

### Setup

1. Clone the repo and enter the project directory.

2. Copy env example and set required variables:

   ```bash
   cp .env.example .env
   ```

3. Generate a 32-byte encryption key (required):

   ```bash
   openssl rand -base64 32
   ```

   Put the output in `.env` as `ENCRYPTION_KEY=...`.

4. Ensure `DATABASE_URL` points to a SQLite path. Default:

   `sqlite:data/shredlink.db?mode=rwc`

   The app will create the `data` directory and DB file if missing.

### Run

From the **project root** (parent of `server/` and `public/`):

```bash
cargo run --release
```

- App listens on `HOST:PORT` (default `127.0.0.1:8080`).
- Open `http://127.0.0.1:8080` in a browser.
- Migrations run on startup; SQLite DB and tables are created automatically.

### Configure `.env`

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Bind host | `127.0.0.1` |
| `PORT` | Bind port | `8080` |
| `DATABASE_URL` | SQLite (or later PostgreSQL) URL | `sqlite:data/shredlink.db?mode=rwc` |
| `ENCRYPTION_KEY` | 32 bytes, base64 (required) | — |
| `CREATE_RATE_LIMIT_PER_MINUTE` | Rate limit for create endpoint per IP | `10` |
| `MAX_TEXT_SIZE_BYTES` | Max request body size for text | `100000` |
| `CLEANUP_INTERVAL_SECS` | Background cleanup interval (seconds) | `600` |
| `BASE_URL` | Public base URL for generated links | `http://127.0.0.1:8080` |

## Deploy on a Linux VPS

### 1. Build binary

On the VPS or a compatible Linux host (e.g. same glibc):

```bash
cd /path/to/ShredLink
cargo build --release --manifest-path server/Cargo.toml
```

Binary: `server/target/release/shredlink-server` (or `shredlink_server` depending on crate name).

### 2. Install and run with systemd

- Copy the release binary to a system path, e.g. `/usr/local/bin/shredlink-server`.
- Create a dedicated user and directory, e.g. `/var/lib/shredlink`. Copy into it:
  - `public/` (frontend)
  - `server/migrations/` (e.g. as `migrations/`)
  - `.env` (with `ENCRYPTION_KEY`, `DATABASE_URL`, `BASE_URL`, etc.)
- Optionally set `MIGRATIONS_DIR=/var/lib/shredlink/migrations` in `.env` if migrations are not next to the crate source.
- Run the server with **WorkingDirectory** set to the directory that contains `public/` and your DB (e.g. `data/`). The app serves files from `./public` when present, else falls back to compile-time path.

Example unit file: `/etc/systemd/system/shredlink.service`

```ini
[Unit]
Description=ShredLink secure text-to-link service
After=network.target

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/var/lib/shredlink
EnvironmentFile=/var/lib/shredlink/.env
ExecStart=/usr/local/bin/shredlink-server
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Then:

```bash
sudo systemctl daemon-reload
sudo systemctl enable shredlink
sudo systemctl start shredlink
sudo systemctl status shredlink
```

### 3. Firewall

Open the port the app listens on (e.g. 8080), or only localhost if you put Nginx in front:

```bash
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
# If app is exposed directly:
# sudo ufw allow 8080/tcp
sudo ufw enable
```

### 4. Nginx reverse proxy

Example HTTPS config (replace domain and paths):

```nginx
server {
    listen 80;
    server_name shredlink.example.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    server_name shredlink.example.com;

    ssl_certificate     /etc/letsencrypt/live/shredlink.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/shredlink.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Set `BASE_URL=https://shredlink.example.com` in `.env` so generated links use HTTPS.

### 5. Production hardening

- Use a strong, unique `ENCRYPTION_KEY`; rotate only if you re-encrypt existing data.
- Run the process as a non-root user (e.g. `User=www-data` in systemd).
- Keep the server and Rust toolchain updated.
- Prefer PostgreSQL in production if you need concurrency and scale; keep the same schema and swap `DATABASE_URL` and driver in code.
- Restrict DB file or DB user permissions; do not expose the DB port publicly.
- Rely on Nginx (or similar) for TLS and optional rate limiting / DDoS mitigation in front of the app.

## API

- **POST /api/create** – JSON body: `text`, optional `password`, `expire_minutes`, `expire_hours`, `one_time_view`, `one_time_password`. Returns `{ "token", "url" }`. Rate limited per IP.
- **GET /s/{token}** – Redirects to password page if protected, or shows decrypted text. Returns 404/410 for missing or expired/used links.
- **POST /api/unlock/{token}** – JSON body: `password`. Returns `{ "text" }` on success.

## License

MIT.
