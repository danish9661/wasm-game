# Deploying Starfall

Starfall is a static WebGPU web game plus an optional authoritative co-op
server. Both ship from this repo.

## 1. Build the client (`pkg/`)

```bash
./build.sh
```

`pkg/` now contains the wasm bundle + static shell. Serve it from any static
host (the game itself runs entirely client-side; no server is required for
single-player).

## 2. (Optional) Run the co-op server

```bash
SEED=1337 BIND=0.0.0.0:8081 ./target/debug/starfall-server
# release build:
cargo build --release -p starfall-server
SEED=1337 BIND=0.0.0.0:8081 ./target/release/starfall-server
```

The server is authoritative: it simulates the world at 30 Hz and streams
JSON snapshots to every connected client over WebSocket. Joining a game is
purely a URL query on the client:

```
index.html?mp=ws://your-host:8081&name=Alias&token=my-account
```

- `mp` — WebSocket URL of the co-op server (omit for single-player).
- `name` — display alias for allies (defaults to "Wanderer").
- `token` — optional account id; when present the server persists your
  inventory/position to `saves/<token>.json` on disconnect and restores it on
  reconnect (cross-device save).

## 3. Production hosting notes

- **TLS required.** Browsers only expose WebGPU and secure WebSocket
  (`wss://`) on HTTPS pages. Put both the static files and the WS server
  behind an HTTPS reverse proxy.
- **Proxy the socket.** Map a path (e.g. `/ws`) on your domain to the server's
  `:8081`, so clients use `?mp=wss://your-domain/ws`. Example (Caddy):

  ```
  your-domain {
      root * /path/to/pkg
      file_server
      reverse_proxy /ws localhost:8081
  }
  ```

- **SEO placeholders.** `index.html`, `robots.txt`, and `sitemap.xml` currently
  use `https://example.com/` as a placeholder. Replace it with your real domain
  before going live:

  ```bash
  DOMAIN=https://your-domain.com
  sed -i "s#https://example.com#$DOMAIN#g" pkg/robots.txt pkg/sitemap.xml pkg/index.html
  ```

  (Re-run after each `build.sh`, or edit the sources under `web/static/` first.)

## 4. Verify

Serve `pkg/` and open it in a WebGPU-capable browser (Chrome/Edge 113+). For
co-op, open the `?mp=...` URL in two tabs/windows — both players share one
simulated world, each tinted a distinct color, with interpolated movement.
