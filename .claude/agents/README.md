# OpenPX agent roster

Five specialist agents maintain this repo. All run on `claude-opus-4-7` with max reasoning effort. Every PR they open requires explicit human approval — no auto-merge.

## Roster

| Agent | Owns | Triggered by |
|---|---|---|
| `orchestrator` | Top-level dispatch + cross-cutting (`just sync-all`) + per-PR changelog appends | Weekly cron Mon 06:00 UTC, admin-associated issue events, `pull_request.closed` (merged), `just maintain` |
| `kalshi-maintainer` | `engine/exchanges/kalshi/` (excluding `auth.rs`) and Kalshi entries in `engine/core/src/exchange/manifests/kalshi.rs` | Dispatched by `orchestrator` on Kalshi drift or Kalshi-tagged issues |
| `polymarket-maintainer` | All of `engine/exchanges/polymarket/` (including funds-moving files; CODEOWNERS forces human review on those) and Polymarket entries in manifests + the contracts snapshot | Dispatched by `orchestrator` on Polymarket drift or Polymarket-tagged issues |
| `core-architect` | `engine/core/` — trait, manifest schema, normalizers, error hierarchy, models. Implements approved parity proposals. | Dispatched by `orchestrator` when a `parity-fill-approved` issue lands, or when a maintainer requests a cross-cutting refactor |
| `parity-analyst` | Cross-exchange parity report at `docs/parity/STATUS.md`; UX-improvement proposals; schema-naming review on maintainer PRs | Dispatched by `orchestrator` after maintainer dispatches settle |

## How dispatch works

`orchestrator` is the only agent that fans work out. When the weekly tick runs `maintenance/scripts/check_docs_drift.py --json` and the report shows multiple unrelated drift items, the orchestrator dispatches **multiple maintainer runs**, each scoped to one concern, each producing **one PR**. Maintainer prompts forbid bundling unrelated changes.

After maintainers settle, the orchestrator dispatches the parity analyst, which (a) regenerates `docs/parity/STATUS.md` from `Exchange::describe()` flags, (b) prospects for UX gaps by comparing both exchanges' `llms.txt` against the unified trait, and (c) reviews any new schema-mapping field names introduced by maintainers in this cycle for clarity and convention.

The orchestrator then handles cross-cutting work: `just sync-all` if any merged PR touched `engine/core/src/models/**`; `just docs` if any Rust docstring changed.

## Triggers

- `agent-tick.yml` — weekly cron + `workflow_dispatch` + admin-gated issue events + `pull_request.closed` merged events. Single workflow, single entry point. Orchestrator's prompt branches on the trigger.
- `docs-drift.yml` — hourly cron, runs `maintenance/scripts/check_docs_drift.py --json`, uploads artifact only. Does not dispatch agents itself.

## Invariants every agent honors

- **Single-purpose PRs.** One concern per PR. The orchestrator fans out multi-item drift; maintainers refuse to bundle.
- **Structured PR body.** What changed / Why / Files / Tests / Review focus. Reviewers use this to scan in under 60 seconds.
- **Reviewer requested explicitly.** Every PR runs `gh pr edit --add-reviewer MilindPathiyal` so the human gets a GitHub review-request email.
- **Never merge.** Agents `gh pr create` but never `gh pr merge`. Humans always merge.
- **Never bypass CI.** No `--no-verify`, no `--no-gpg-sign`, no skipping of any pre-commit or commit-msg hook.
- **Never edit human-only paths.** CODEOWNERS and `.github/REVIEW_POLICY.md` define these. Each agent's prompt also names them explicitly so a misbehaving agent fails fast at prompt level, not just at CODEOWNERS.

## Files

- `HANDOFF.md` — exit-message contract every agent uses
- `orchestrator.md`, `kalshi-maintainer.md`, `polymarket-maintainer.md`, `parity-analyst.md` — agent definitions
- `../runbooks/` — procedural checklists agents read at startup

## See also

- `.github/CODEOWNERS` — mechanical enforcement of human-only paths
- `.github/REVIEW_POLICY.md` — written review policy, label taxonomy
- `/Users/mppathiyal/.claude/plans/just-so-i-can-rustling-planet.md` — the full design
