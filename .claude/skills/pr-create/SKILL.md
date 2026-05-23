---
name: pr-create
description: Create a GitHub pull request from local changes. Verifies build and tests pass first, splits changes into focused commits without using `git add .`, drafts a PR body following the project's template, and waits for user approval before pushing. Use when the user asks to open a PR, create a pull request, push changes for review, ship the current branch, or invokes /create-pr.
---

# Create PR

Open a GitHub pull request for the current branch. Stop at the first failure — never push or PR with broken builds.

## Workflow

Run every step in order. Stop and report if anything fails.

### 1. Pre-flight

- `git status` — list untracked + modified files.
- `git log --oneline origin/HEAD..HEAD` — local commits not yet pushed.
- `git diff origin/HEAD...HEAD` — full diff vs base (use this to plan commits later).
- Detect base branch from `origin/HEAD`; fall back to `main`.
- If the current branch **is** the base branch, ask the user for a new branch name and create it before continuing.

### 2. Build + test

Determine commands in this order:
1. Read project's `CLAUDE.md` / `AGENTS.md` — use whatever build/test commands are documented.
2. If absent, detect from files:
   - `Cargo.toml` → `cargo build && cargo test`
   - `package.json` → check `scripts` for `build` and `test`
   - `go.mod` → `go build ./... && go test ./...`
   - `pyproject.toml` / `setup.py` → check for `pytest`, `make test`, etc.
3. Run them. **If anything fails, stop.** Report the failure and let the user fix it.

### 3. Commit splitting

Group the diff (modified + untracked files) by logical concern. Examples:
- Schema changes (`migrations/`)
- One feature/domain (`src/routes/users/`)
- Test additions (`tests/`)
- Config/tooling (`.claude/`, `docker-compose.yml`, dep bumps)
- Docs (`README.md`, `CLAUDE.md`)

For each group:
- **`git add <explicit file paths>`** — name every file.
- **Never `git add .`, `git add -A`, or `git add -u`.**
- Match commit-message convention by inspecting recent `git log`. One-line subject + optional body for the *why*. Use a HEREDOC for multiline messages.
- After each commit, run `git status` to confirm what remains for the next group.

If a single file legitimately spans multiple concerns, use `git add -p <file>` for hunk-level staging.

### 4. PR body

- If `.github/PULL_REQUEST_TEMPLATE.md` exists, follow its sections verbatim.
- Otherwise use this default:

```
## Summary
<2-4 bullets, focused on the why>

## Test plan
- [ ] <how this was verified>
```

Fill sections from the commits just made and test results from step 2.

**Show the proposed title + body to the user and ask for explicit approval. Do not push until they confirm.** They may request edits — apply them and re-show.

### 5. Push + open PR

After approval:
1. `git push -u origin <branch>`
2. `gh pr create --title "<title>" --body "$(cat <<'EOF' ... EOF)"` using the approved body.
3. Return the PR URL to the user.

## Hard rules

- **No `git add .` / `-A` / `-u`.** Always explicit paths.
- **No pushing before build + tests pass.**
- **No PR creation without explicit user approval of the body.**
- **No `--no-verify`, no `--force` push, no amending already-pushed commits** unless the user explicitly asks.
- **No committing `.env` or other gitignored secret files** — if they appear staged, abort and tell the user.
