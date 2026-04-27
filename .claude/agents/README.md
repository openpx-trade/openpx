# OpenPX agent roster

Five specialist agents maintain this repo. All run on `claude-opus-4-7` with max reasoning effort. Every PR they open requires explicit human approval — no auto-merge.

## Roster

| Agent | Owns | Triggered by |
|---|---|---|
| `orchestrator` | Daily changelog cycle — diffs upstream Kalshi + Polymarket changelogs against the lock, classifies new entries, dispatches one maintainer/architect call per concern | Daily cron 00:00 UTC, `workflow_dispatch` (incl. `just backfill <DATE>`) |
| `kalshi-maintainer` | `engine/exchanges/kalshi/` (excluding `auth.rs`) and Kalshi entries in `engine/core/src/exchange/manifests/kalshi.rs` | Dispatched by `orchestrator` on a Kalshi changelog entry classified as `critical-exchange-specific` |
| `polymarket-maintainer` | All of `engine/exchanges/polymarket/` (including funds-moving files; CODEOWNERS forces human review on those) and Polymarket entries in manifests + the contracts snapshot | Dispatched by `orchestrator` on a Polymarket changelog entry classified as `critical-exchange-specific` |
| `core-architect` | `engine/core/` — trait, manifest schema, normalizers, error hierarchy, models. Implements approved parity proposals (i.e. parity-analyst proposals that a human approved, then routed back through a future cycle). | Dispatched by `orchestrator` only when a `parity-fill-approved` issue exists at cycle-start time |
| `parity-analyst` | Writes a unified-trait proposal as a new `parity-gap` issue when the orchestrator classifies a changelog entry as `overlap-opportunity` (the same/similar capability has appeared on the other exchange). Never opens PRs. | Dispatched by `orchestrator` per `overlap-opportunity` entry |

## How dispatch works

`orchestrator` is the only agent that fans work out. The daily cycle:

1. Run `python3 maintenance/scripts/check_docs_drift.py --json` — fetches both upstream changelogs, diffs against `maintenance/scripts/exchange-docs.lock.json`, returns the unified diff per exchange.
2. For each new `<Update>` block in the diff, classify as `overlap-opportunity` (→ dispatch `parity-analyst` to file a proposal issue), `critical-exchange-specific` (→ dispatch the relevant maintainer to open a PR), or `operational-only` (→ skip).
3. Each dispatch is its own concern → its own Task call → its own PR or issue. Never bundle.
4. After dispatches settle, refresh the lock and open one `chore(drift): refresh changelog lock` PR.
5. End with the standard handoff message.

## Triggers

- `agent-tick.yml` — daily cron at 00:00 UTC + `workflow_dispatch`. The `workflow_dispatch` form takes optional `mode` and `since` inputs to support `just backfill <DATE>` (re-process every changelog entry dated on/after `since`).

## Invariants every agent honors

- **Single-purpose PRs.** One concern per PR. The orchestrator fans out multi-item drift; maintainers refuse to bundle.
- **Structured PR body.** What changed / Why / Files / Tests / Review focus. Reviewers use this to scan in under 60 seconds.
- **Reviewer requested explicitly.** Every PR runs `gh pr edit --add-reviewer MilindPathiyal` so the human gets a GitHub review-request email.
- **Never merge.** Agents `gh pr create` but never `gh pr merge`. Humans always merge.
- **Never bypass CI.** No `--no-verify`, no `--no-gpg-sign`, no skipping of any pre-commit or commit-msg hook.
- **Never edit human-only paths.** CODEOWNERS and `.github/REVIEW_POLICY.md` define these. Each agent's prompt also names them explicitly so a misbehaving agent fails fast at prompt level, not just at CODEOWNERS.
- **Never open a PR without completing `maintenance/runbooks/pr-preflight.md`.** Every bot PR keeps the Rust core, Python SDK, TypeScript SDK, and docs in sync, and every SDK actually builds and imports cleanly. CI gates `SDK Sync Check`, `Python SDK Build`, and `Node.js SDK Build` mechanically backstop this. If a preflight step can't run because of missing tooling, the agent stops and comments on the source issue — it does not invent a justification to skip.

## Files

- `HANDOFF.md` — exit-message contract every agent uses
- `orchestrator.md`, `kalshi-maintainer.md`, `polymarket-maintainer.md`, `parity-analyst.md` — agent definitions
- `../runbooks/` — procedural checklists agents read at startup

## See also

- `.github/CODEOWNERS` — mechanical enforcement of human-only paths
- `.github/REVIEW_POLICY.md` — written review policy, label taxonomy
- `/Users/mppathiyal/.claude/plans/just-so-i-can-rustling-planet.md` — the full design
