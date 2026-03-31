---
name: pm
description: Project management — status, features, tickets, roadmap for Whatsim
user_invocable: true
---

# PM Skill

Help manage the Whatsim project — understand status, discuss features, create/manage GitHub issues.

## When Invoked

1. **Check current project status:**
   - Run `git log --oneline -20` to see recent activity
   - Run `git diff --stat` to see uncommitted changes
   - Check open issues: `gh issue list --repo malazaysc/whatsim`
   - Check open PRs: `gh pr list --repo malazaysc/whatsim`
   - Read CLAUDE.md for project context

2. **Present a summary:**
   - What's been built so far
   - What's in progress (uncommitted changes, open PRs)
   - What's planned (open issues, README "Next Steps")
   - Any blockers or decisions needed

3. **For feature discussions:**
   - Reference the project's design principles from the README
   - Consider scope — v0 vs future versions
   - Think about agent-first API design
   - Keep it local-first, no cloud dependencies

4. **Creating issues:**
   Use `gh issue create` with clear titles and descriptions:
   ```bash
   gh issue create --repo malazaysc/whatsim \
     --title "Add support for image messages" \
     --body "## Description
   Add image message type support to the simulation engine.
   
   ## Tasks
   - [ ] Add image variant to MessageKind
   - [ ] Update Meta webhook payload generation
   - [ ] Update mock outbound endpoint
   - [ ] Update frontend chat bubble rendering
   
   ## Notes
   Start with URL-based images only, no upload handling yet."
   ```

5. **For labeling and organizing:**
   - Use labels: `enhancement`, `bug`, `documentation`, `good first issue`
   - Reference the architecture (which crate is affected)
   - Link related issues

## Project Scope Reminders

**In scope for v0:** Text messages, conversation management, webhook forwarding, mock outbound, inspector panel, SSE streaming, in-memory storage.

**Out of scope for v0:** Auth, billing, teams, permissions, CRM, analytics, media uploads, template management, real Meta auth, cloud deployment, plugins, Twilio/360dialog, mobile app, Electron/Tauri.
