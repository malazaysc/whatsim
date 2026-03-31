# Whatsim

**Local WhatsApp sandbox for developers and coding agents.**

Whatsim lets you simulate WhatsApp Business conversations locally — send fake inbound messages, forward them as real Meta-style webhooks to your app, intercept outbound replies, and see everything in a WhatsApp Web-like UI. No real phone. No Meta account. No Docker.

## Features

- WhatsApp Web-style chat UI
- Simulates Meta Cloud API inbound webhooks
- Mock outbound provider endpoint (replaces `graph.facebook.com`)
- Real-time updates via SSE
- Agent-first HTTP API for automated testing
- Raw payload inspector for debugging
- Organization/tenant scoping
- Single binary — backend + frontend embedded
- Runs fully in-memory by default

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (stable)
- [Node.js](https://nodejs.org/) (v18+)

### Build & Run

```bash
# Clone
git clone https://github.com/malazaysc/whatsim.git
cd whatsim

# Build frontend
cd web && npm install && npm run build && cd ..

# Run (dev mode)
WHATSIM_WEBHOOK_TARGET=http://localhost:3000/api/webhooks/whatsapp \
  cargo run -p whatsim-server

# Open http://127.0.0.1:3210
```

### Development Mode

Run frontend and backend separately for hot-reload:

```bash
# Terminal 1: Vite dev server with proxy
cd web && npm run dev

# Terminal 2: Rust backend
cargo run -p whatsim-server

# Open http://localhost:5173 (Vite proxies API calls to :3210)
```

### Production Build

```bash
cd web && npm install && npm run build && cd ..
cargo build --release
./target/release/whatsim
```

## Configuration

All configuration via environment variables:

| Variable | Default | Description |
|---|---|---|
| `WHATSIM_HOST` | `127.0.0.1` | Bind address |
| `WHATSIM_PORT` | `3210` | Server port |
| `WHATSIM_LOG_LEVEL` | `info` | Log level (trace/debug/info/warn/error) |
| `WHATSIM_WEBHOOK_TARGET` | — | URL to forward inbound webhook payloads |
| `WHATSIM_ENABLE_PERSISTENCE` | `false` | Enable SQLite persistence |
| `WHATSIM_DB_PATH` | — | Path to SQLite database file |
| `WHATSIM_DEFAULT_ORGANIZATION_ID` | — | Default org ID for new conversations |
| `WHATSIM_PUBLIC_BASE_URL` | `http://127.0.0.1:3210` | Public URL for the server |

Copy `.env.example` to `.env` to get started.

## How It Works

Whatsim creates a closed-loop simulation:

```
┌─────────────────────────────────────────────────────────────┐
│                        Whatsim                              │
│                                                             │
│  ┌──────────┐    POST webhook    ┌──────────────────────┐   │
│  │  Chat UI │ ──────────────────>│   Your App           │   │
│  │          │                    │   (webhook handler)  │   │
│  │  inbound │    Meta-style      │                      │   │
│  │  messages│    payload         │                      │   │
│  │          │                    └──────────┬───────────┘   │
│  │          │                               │               │
│  │  outbound│    POST to mock    ┌──────────▼───────────┐   │
│  │  messages│ <──────────────────│  /api/mock-meta/     │   │
│  │          │                    │  messages             │   │
│  └──────────┘                    └──────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

1. **You send a message** in the Whatsim UI (or via API)
2. **Whatsim generates** a realistic Meta Cloud API webhook payload
3. **Whatsim POSTs** that payload to your app's webhook URL
4. **Your app processes** it and sends an outbound reply to Whatsim's mock endpoint
5. **Whatsim captures** the reply and displays it in the chat

## API Reference

### Conversations

```bash
# Create a conversation
curl -X POST http://localhost:3210/api/conversations \
  -H 'Content-Type: application/json' \
  -d '{"fromPhone": "+5493510000001", "contactName": "Test User"}'

# List conversations
curl http://localhost:3210/api/conversations

# Get a conversation
curl http://localhost:3210/api/conversations/{id}
```

### Messages

```bash
# Send an inbound text message (simulates customer sending a WhatsApp message)
curl -X POST http://localhost:3210/api/messages/inbound-text \
  -H 'Content-Type: application/json' \
  -d '{"conversationId": "...", "text": "Hola, quiero info"}'

# List messages in a conversation
curl http://localhost:3210/api/conversations/{id}/messages
```

### Mock Meta Outbound (point your app here instead of graph.facebook.com)

```bash
# Your app sends this to Whatsim instead of Meta
curl -X POST http://localhost:3210/api/mock-meta/messages \
  -H 'Content-Type: application/json' \
  -d '{
    "messaging_product": "whatsapp",
    "to": "+5493510000001",
    "type": "text",
    "text": {"body": "Thanks for reaching out!"}
  }'
```

### Events & Debugging

```bash
# List events for a conversation
curl http://localhost:3210/api/conversations/{id}/events

# Health check
curl http://localhost:3210/health
```

### Real-time Updates (SSE)

```bash
# Subscribe to live events
curl -N http://localhost:3210/api/stream
```

## Agent Integration Example

For coding agents or automated tests:

```bash
# 1. Create a conversation
CONV=$(curl -s -X POST http://localhost:3210/api/conversations \
  -H 'Content-Type: application/json' \
  -d '{"fromPhone": "+5493510000001", "contactName": "Bot Test"}')
CONV_ID=$(echo $CONV | jq -r '.id')

# 2. Send an inbound message
curl -s -X POST http://localhost:3210/api/messages/inbound-text \
  -H 'Content-Type: application/json' \
  -d "{\"conversationId\": \"$CONV_ID\", \"text\": \"Hello!\"}"

# 3. Wait a moment for your app to process and respond...
sleep 2

# 4. Check for outbound messages
curl -s http://localhost:3210/api/conversations/$CONV_ID/messages | jq '.[] | select(.direction == "outbound")'
```

## Project Structure

```
whatsim/
├── crates/
│   ├── whatsim-core/          # Domain types, config, errors
│   ├── whatsim-storage/       # In-memory (+ future SQLite) store
│   ├── whatsim-provider-meta/ # Meta webhook payload types & generation
│   ├── whatsim-simulator/     # Simulation engine orchestration
│   └── whatsim-server/        # Axum server, routes, SSE, static assets
├── web/                       # React + Vite frontend
├── .claude/                   # Claude Code skills & config
├── Cargo.toml                 # Workspace root
├── CLAUDE.md                  # Claude Code project guide
└── README.md
```

## Next Steps

Features planned for future versions:

- [ ] **Media messages** — image, document, audio, video placeholders
- [ ] **Interactive messages** — buttons, lists, quick replies
- [ ] **Template messages** — outbound template support
- [ ] **SQLite persistence** — persist conversations across restarts
- [ ] **Webhook retry logic** — configurable retry with backoff
- [ ] **Multiple provider support** — Twilio, 360dialog adapters
- [ ] **Scenario runner** — script multi-step conversation flows
- [ ] **Playwright E2E tests** — browser-based integration testing
- [ ] **Status callbacks** — delivery status webhook simulation
- [ ] **Export/import** — save and replay conversation snapshots
- [ ] **Dark mode** — because of course

## Contributing

Contributions welcome! Please open an issue first to discuss what you'd like to change.

## License

MIT
