---
name: e2e-testing
description: Run end-to-end tests against the Whatsim app using Playwright
user_invocable: true
---

# E2E Testing Skill

Run end-to-end tests for Whatsim using Playwright.

## Setup

If Playwright is not installed yet:
```bash
cd web && npx playwright install --with-deps chromium
```

## Running Tests

```bash
cd /Users/malazay/dev/projects/whatsim/web
npx playwright test
```

## What to Test

When asked to run e2e tests or verify the app works end-to-end:

1. **Start the backend** if not running:
   ```bash
   cargo build -p whatsim-server && cargo run -p whatsim-server &
   ```

2. **Run the full flow test:**
   - Open the app at http://127.0.0.1:3210
   - Create a new conversation via the UI
   - Send an inbound message
   - Verify the message appears in the chat
   - Check the inspector panel shows events

3. **API-level e2e test:**
   ```bash
   # Create conversation
   CONV=$(curl -s -X POST http://localhost:3210/api/conversations \
     -H 'Content-Type: application/json' \
     -d '{"fromPhone": "+5493510000001", "contactName": "E2E Test"}')
   CONV_ID=$(echo $CONV | jq -r '.id')
   
   # Send inbound
   curl -s -X POST http://localhost:3210/api/messages/inbound-text \
     -H 'Content-Type: application/json' \
     -d "{\"conversationId\": \"$CONV_ID\", \"text\": \"E2E test message\"}"
   
   # Verify messages
   curl -s http://localhost:3210/api/conversations/$CONV_ID/messages | jq .
   
   # Send mock outbound
   curl -s -X POST http://localhost:3210/api/mock-meta/messages \
     -H 'Content-Type: application/json' \
     -d '{"messaging_product": "whatsapp", "to": "+5493510000001", "type": "text", "text": {"body": "Reply from app"}}'
   
   # Verify both messages
   curl -s http://localhost:3210/api/conversations/$CONV_ID/messages | jq .
   
   # Check events
   curl -s http://localhost:3210/api/conversations/$CONV_ID/events | jq .
   ```

4. **Report results** — summarize pass/fail for each step.

## Writing New Tests

When writing Playwright tests, put them in `web/e2e/` directory. Use the Page Object pattern and test the critical user flows:
- Create conversation
- Send message
- View conversation list
- Open inspector
- Real-time message updates
