# Whatsim — Claude Code Project Guide

## What is Whatsim?

Whatsim is a **local WhatsApp sandbox for developers and coding agents**. It simulates inbound WhatsApp Business webhook conversations locally, intercepts outbound messages, and displays everything in a WhatsApp Web-style UI — all without a real phone or Meta account.

## Architecture

Single self-contained binary: Rust backend + embedded React frontend.

### Rust workspace (crates/)

| Crate | Purpose |
|---|---|
| `whatsim-core` | Domain types, config, errors, normalized events |
| `whatsim-storage` | In-memory store (thread-safe with `Arc<RwLock>`) |
| `whatsim-provider-meta` | Meta Cloud API payload types and generation |
| `whatsim-simulator` | Orchestrates inbound/outbound simulation flows |
| `whatsim-server` | Axum server, API routes, SSE streaming, static asset serving |

### Frontend (web/)

React + Vite + TypeScript + Tailwind CSS. Builds to `web/dist/` which is embedded into the Rust binary via `rust-embed`.

## Build & Run

```bash
# Dev mode (frontend + backend separate)
cd web && npm run dev          # Terminal 1: Vite dev server on :5173
cargo run -p whatsim-server    # Terminal 2: Rust server on :3210

# Production build
cd web && npm run build        # Build frontend to web/dist/
cargo build --release          # Embeds web/dist/ into binary
./target/release/whatsim       # Single binary serves everything
```

## Key Commands

```bash
cargo check                    # Type-check all crates
cargo test                     # Run all tests
cargo test -p whatsim-core     # Test specific crate
cargo clippy                   # Lint
cd web && npm run build        # Build frontend
cd web && npm run dev          # Dev frontend with HMR
```

## API Endpoints

| Method | Path | Description |
|---|---|---|
| GET | `/health` | Health check |
| GET | `/api/config` | App configuration |
| GET | `/api/conversations` | List conversations (?organizationId=) |
| POST | `/api/conversations` | Create conversation |
| GET | `/api/conversations/:id` | Get conversation |
| GET | `/api/conversations/:id/messages` | List messages |
| GET | `/api/conversations/:id/events` | List events |
| POST | `/api/messages/inbound-text` | Send simulated inbound text |
| POST | `/api/mock-meta/messages` | Mock Meta send API (outbound) |
| GET | `/api/stream` | SSE event stream |

## Environment Variables

See `.env.example`. Key ones:
- `WHATSIM_WEBHOOK_TARGET` — where to forward inbound webhook payloads
- `WHATSIM_PORT` — server port (default 3210)
- `WHATSIM_LOG_LEVEL` — tracing level (default info)

## Conventions

- Rust: snake_case, idiomatic error handling with `thiserror`
- JSON API: camelCase field names via `#[serde(rename_all = "camelCase")]`
- Frontend: TypeScript strict mode, functional components, Tailwind utility classes
- No unnecessary abstractions — keep it simple and explicit
- Tests go in `#[cfg(test)] mod tests` blocks within the same file (Rust) or co-located `.test.ts` files (frontend)

## Domain Model

Core types in `whatsim-core/src/types.rs`:
- `Conversation` — a simulated WhatsApp conversation thread
- `Message` — inbound or outbound message with direction, kind, provider
- `Event` — audit log entry (inbound, outbound, webhook delivery, etc.)
- `PayloadSnapshot` — raw webhook/API payload stored for inspection

## Simulation Flow

**Inbound (user -> app):**
1. UI/API sends text → SimulationEngine
2. Engine generates Meta webhook payload
3. Stores payload snapshot + message + event
4. POSTs webhook to configured target URL
5. Records delivery result
6. Broadcasts via SSE

**Outbound (app -> user):**
1. Target app POSTs to `/api/mock-meta/messages`
2. Engine matches conversation by phone number
3. Stores outbound message + payload snapshot + event
4. Broadcasts via SSE
5. Returns Meta-style response
