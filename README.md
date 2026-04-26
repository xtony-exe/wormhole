# 🌀 Wormhole

> **A fast, modern TCP tunnel in Rust** — punch through NAT firewalls and expose local ports to the internet in seconds.
>
> *By **THINKING TEAM** · Authored by **XTONY***

---

## ✨ Features

- 🚀 **Instant tunnels** — expose any local port with one command
- 🔒 **HMAC-SHA256 auth** — optional shared secret to lock down your server
- ♻️ **Auto-reconnect** — `--retry` flag keeps your tunnel alive through network blips
- 🏷️ **Named tunnels** — label tunnels with `--label` for easy identification
- 📱 **QR code output** — scan the tunnel URL instantly on mobile
- 🎨 **Beautiful CLI** — colored output, info boxes, and a clean banner
- 🐳 **Tiny Docker image** — `FROM scratch` final image, ~5MB total
- ⚡ **Async & concurrent** — powered by Tokio, handles hundreds of tunnels

---

## 📦 Installation

### Option 1: Download the .exe (Recommended for Windows)

1. Go to the [Releases](https://github.com/xtony-exe/wormhole/releases) page
2. Download `wormhole.exe`
3. Place it in any folder (e.g. `C:\Tools\`)
4. Open **PowerShell** or **Command Prompt** in that folder
5. You're ready to go!

### Option 2: Build From Source (Rust required)

If you have Rust installed:

```bash
git clone https://github.com/xtony-exe/wormhole
cd wormhole
cargo build --release
```

The binary will be at `target/release/wormhole.exe` (Windows) or `target/release/wormhole` (Linux/Mac).

### Option 3: Docker

```bash
docker build -t wormhole .
```

---

## 🚀 Quick Start — Your First Tunnel in 60 Seconds

### What does this do?

Wormhole lets you make any app running on your computer accessible from the internet.
For example, if you have a website running on `localhost:8080`, wormhole gives you a public URL
that anyone in the world can open.

### Step 1: Run something locally

Start any app on your computer. For example, a simple Python web server:

```bash
python -m http.server 8080
```

Or your Node.js app:

```bash
npm start    # usually runs on port 3000
```

Or literally anything that listens on a port (bots, APIs, games, etc).

### Step 2: Open a wormhole

Open a **new terminal window** and run:

```bash
wormhole open 8080 --to your-server.com
```

> ⚠️ Replace `8080` with whatever port your app uses, and `your-server.com` with the server address.

### Step 3: See the connection info

Wormhole will print a clear info box:

```
  v1.0.0  —  by THINKING TEAM · XTONY

  ◈ Opening [wormhole] on localhost:8080

  ┌──────────────────────────────────────────┐
  │  ✔ WORMHOLE ACTIVE                      │
  ├──────────────────────────────────────────┤
  │    Label  :  wormhole                    │
  │    Server :  your-server.com             │
  │    IP     :  your-server.com             │
  │    Port   :  52341                       │
  │    Local  :  localhost:8080              │
  │    URL    :  your-server.com:52341       │
  └──────────────────────────────────────────┘

  ◉ Scan to connect: [QR CODE]

  ▶ Forwarding traffic... (Ctrl+C to stop)
```

Share the **URL** (`your-server.com:52341`) with anyone — they can access your app!

### Step 4: Stop the tunnel

Press `Ctrl+C` in the terminal to close the tunnel.

---

## 📖 Complete Usage Guide

### Basic Tunnel

```bash
# Expose port 8080 to the internet
wormhole open 8080 --to your-server.com
```

### Named Tunnel (with a label)

```bash
# Give your tunnel a friendly name
wormhole open 8080 --to your-server.com --label "my-website"
```

The label shows up in the info box so you can identify your tunnels easily.

### Auto-Reconnect (stays alive forever)

```bash
# If your WiFi drops or the connection breaks, it reconnects automatically
wormhole open 8080 --to your-server.com --retry
```

Without `--retry`, the tunnel dies on disconnect. With `--retry`, it reconnects
with smart backoff: `2s → 4s → 8s → 16s → 32s` then keeps trying every 32s.

### Password-Protected Tunnel

```bash
# Only clients with the matching secret can connect
wormhole open 8080 --to your-server.com --secret "my-password-123"
```

### Pick a Specific Port

```bash
# Request port 9000 on the remote server (instead of random)
wormhole open 8080 --to your-server.com --port 9000
```

> Note: The requested port may not be available. If so, you'll get an error.

### Forward a Different Host

```bash
# Forward traffic to a different machine on your network
wormhole open 80 --to your-server.com --local-host 192.168.1.50
```

### All Flags Combined

```bash
wormhole open 3000 \
  --to your-server.com \
  --label "my-api" \
  --secret "token123" \
  --retry \
  --port 9000 \
  --local-host localhost
```

---

## 🖥️ Hosting Your Own Wormhole Server

If you have a VPS or cloud server, you can run your own wormhole server.

### Start the Server

On your **public server** (VPS, cloud VM, etc):

```bash
# Basic (no auth, open to anyone)
wormhole host

# With password protection (recommended)
wormhole host --secret "my-server-password"

# With client limit
wormhole host --secret "my-server-password" --max-clients 50
```

The server shows a clean info box:

```
  ┌──────────────────────────────────────────┐
  │  ◈ WORMHOLE SERVER                       │
  ├──────────────────────────────────────────┤
  │    Bind IP     :  0.0.0.0                │
  │    Control Port:  7835                   │
  │    Port Range  :  1024 – 65535           │
  │    Max Clients :  100                    │
  │    Auth        :  ✔ Enabled              │
  └──────────────────────────────────────────┘

  ▶ Waiting for connections... (Ctrl+C to stop)
```

### Connect Clients

On your **local machine**:

```bash
wormhole open 8080 --to YOUR_SERVER_IP --secret "my-server-password" --retry
```

### Server Options

| Flag | Default | What it does |
|---|---|---|
| `--secret` | *(none)* | Password clients must provide to connect |
| `--min-port` | `1024` | Lowest port number allowed for tunnels |
| `--max-port` | `65535` | Highest port number allowed for tunnels |
| `--bind-addr` | `0.0.0.0` | IP address the server listens on |
| `--max-clients` | `100` | Maximum number of tunnels at the same time |

---

## 🧾 Complete Flag Reference

### `wormhole open` (Client)

| Flag | Short | Default | Description |
|---|---|---|---|
| `<local_port>` | — | *(required)* | The port number of your local app |
| `--to` | `-t` | *(required)* | Address of the wormhole server |
| `--local-host` | `-l` | `localhost` | Local host/IP to forward |
| `--port` | `-p` | `0` (auto) | Request a specific remote port |
| `--secret` | `-s` | *(none)* | Password for authentication |
| `--label` | — | *(none)* | Friendly name for this tunnel |
| `--retry` | `-r` | `false` | Auto-reconnect on disconnect |

### `wormhole host` (Server)

| Flag | Short | Default | Description |
|---|---|---|---|
| `--secret` | `-s` | *(none)* | Required password from clients |
| `--min-port` | — | `1024` | Minimum tunnel port |
| `--max-port` | — | `65535` | Maximum tunnel port |
| `--bind-addr` | — | `0.0.0.0` | Control server bind address |
| `--bind-tunnels` | — | *(same as bind-addr)* | Tunnel listener bind address |
| `--max-clients` | — | `100` | Max simultaneous clients |

---

## 🌍 Environment Variables

You can set defaults using environment variables instead of flags:

| Variable | Replaces |
|---|---|
| `WORMHOLE_SERVER` | `--to` |
| `WORMHOLE_SECRET` | `--secret` |
| `WORMHOLE_LOCAL_PORT` | `<local_port>` |
| `WORMHOLE_MIN_PORT` | `--min-port` |
| `WORMHOLE_MAX_PORT` | `--max-port` |

Example:

```bash
# Set once
set WORMHOLE_SERVER=your-server.com
set WORMHOLE_SECRET=mytoken

# Then just run
wormhole open 8080
```

---

## 💡 Real-World Examples

### 🌐 Share a website you're building

```bash
# Your React/Vue/Next.js app on port 3000
wormhole open 3000 --to your-server.com --label "frontend" --retry
```

### 🤖 Expose a Discord/Telegram bot webhook

```bash
# Your bot listens on port 5000 for webhooks
wormhole open 5000 --to your-server.com --label "bot-webhook" --retry
```

### 🔗 Share an API for testing

```bash
# Let your teammate test your API
wormhole open 4000 --to your-server.com --label "rest-api"
# Send them the URL from the info box
```

### 🎮 Host a game server temporarily

```bash
# Minecraft server on port 25565
wormhole open 25565 --to your-server.com --label "minecraft"
```

### 🔧 Remote access to SSH

```bash
# Expose SSH (port 22) — use with caution!
wormhole open 22 --to your-server.com --secret "strong-password" --label "ssh"
```

### 📱 Test mobile apps against localhost

```bash
wormhole open 8080 --to your-server.com --label "mobile-test"
# Scan the QR code with your phone!
```

---

## 🐳 Docker Usage

### Run the Server

```bash
docker run -d \
  -p 7835:7835 \
  -p 1024-65535:1024-65535 \
  wormhole host --secret "mytoken"
```

### Run the Client

```bash
docker run wormhole open 8080 --to your-server.com --secret "mytoken"
```

---

## ❓ Troubleshooting

### "could not connect to server"
- Check that the server address is correct
- Make sure port **7835** is open on the server's firewall
- If using a secret, make sure client and server secrets match

### "port already in use"
- The requested port is taken. Use `--port 0` (or omit `--port`) for auto-assignment

### "server requires authentication"
- The server has `--secret` set. Add `--secret "same-password"` to your client command

### "timed out waiting for initial message"
- Server might be down or unreachable
- Check your internet connection
- Try adding `--retry` to keep trying

### Tunnel closes when I close the terminal
- Run wormhole inside `tmux`, `screen`, or as a background service
- On Windows: use `Start-Process` or run as a service

---

## 🔐 How Authentication Works

1. Server sends a **random challenge** (UUID) to the client
2. Client computes `HMAC-SHA256(SHA256(your_secret), challenge)` and sends it back
3. Server validates the response — rejects on mismatch

**No passwords are ever sent in plaintext.** The secret is hashed before use.

---

## 🏗️ Architecture

```
[Internet User]
      │
      │  connects to public port
      ▼
[WORMHOLE SERVER]  ←── control channel (port 7835) ──→  [WORMHOLE CLIENT]
      │                                                        │
      │  notifies client of incoming connection                │
      │                                                        ▼
      └──────────── raw TCP proxy ────────────────→  [Your Local App]
```

- **Control channel** (port 7835): JSON messages over null-delimited TCP
- **Data channel**: raw bidirectional TCP splice — zero overhead
- **Heartbeat**: server pings every 500ms to detect dead connections

---

## 📄 License

MIT — see [LICENSE](LICENSE)

---

<div align="center">
  <br>
  <b>🌀 WORMHOLE v1.0.0</b>
  <br>
  <i>by <b>THINKING TEAM</b> · <b>XTONY</b></i>
  <br><br>
  <sub>A fast, modern TCP tunnel — punch through NAT firewalls in seconds.</sub>
</div>
