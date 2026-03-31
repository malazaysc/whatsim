---
name: implement
description: Main implementation loop for building Whatsim features
user_invocable: true
---

# Implement Skill

The main implementation workflow for Whatsim features and fixes.

## When Invoked

Follow this implementation loop:

### 1. Understand the Task
- Read the relevant issue or user request
- Read CLAUDE.md for project context
- Identify which crates/files are affected
- Check for related existing code

### 2. Plan the Changes
- List the files that need to change
- Identify the order of changes (types first, then storage, then engine, then server, then frontend)
- Consider if tests need updating

### 3. Implement (Crate Order)

Always work bottom-up through the dependency chain:

1. **whatsim-core** — New types, enums, error variants
2. **whatsim-provider-meta** — Payload type changes, generation updates
3. **whatsim-storage** — Storage method additions/changes
4. **whatsim-simulator** — Engine logic changes
5. **whatsim-server** — Route handlers, API changes
6. **web/** — Frontend components, API client, types

### 4. Verify

After each change:
```bash
cargo check                    # Type-check Rust
cargo test                     # Run Rust tests
cd web && npm run build        # Build frontend
```

### 5. Test the Full Flow

```bash
# Start the server
cargo run -p whatsim-server &

# Create a conversation
curl -s -X POST http://localhost:3210/api/conversations \
  -H 'Content-Type: application/json' \
  -d '{"fromPhone": "+5493510000001", "contactName": "Test"}'

# Send a message
curl -s -X POST http://localhost:3210/api/messages/inbound-text \
  -H 'Content-Type: application/json' \
  -d '{"conversationId": "...", "text": "Test message"}'

# Check the UI at http://localhost:3210
```

## Code Conventions

- **Rust:** snake_case, `#[serde(rename_all = "camelCase")]` on API-facing structs
- **TypeScript:** camelCase, strict mode, functional components
- **API JSON:** camelCase field names
- **Tests:** in-file `#[cfg(test)] mod tests` for Rust, co-located for frontend
- **Errors:** use `WhatsimError` variants, never panic in handlers
- **State:** `InMemoryStore` is the single source of truth, accessed via `SimulationEngine`
- **Real-time:** broadcast via `tokio::sync::broadcast` channel, consumed as SSE

## Common Patterns

### Adding a new API endpoint:
1. Add handler function in `crates/whatsim-server/src/routes/`
2. Register route in `main.rs`
3. Add API client function in `web/src/api.ts`
4. Wire into frontend components

### Adding a new message type:
1. Add variant to `MessageKind` in `whatsim-core`
2. Update payload generation in `whatsim-provider-meta`
3. Update `SimulationEngine` in `whatsim-simulator`
4. Add endpoint/modify endpoint in `whatsim-server`
5. Update chat bubble rendering in frontend

### Adding a new event type:
1. Add variant to `EventType` in `whatsim-core`
2. Emit events in `whatsim-simulator`
3. Event will auto-appear in inspector via existing `list_events` endpoint
