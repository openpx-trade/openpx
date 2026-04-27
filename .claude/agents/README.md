# OpenPX agent roster

Four specialist agents maintain this repo. All run on `claude-opus-4-7` with max reasoning effort. Every PR they open requires explicit human approval — no auto-merge.

## Roster

| Agent | Owns | Triggered by |
|---|---|---|
| `orchestrator` | Daily cycle — (1) diffs the Kalshi + Polymarket changelogs against the lock and dispatches per actionable entry; (2) scans both exchanges' `describe()` for unimplemented scaffolded methods and dispatches the relevant maintainer; (3) appends one bullet per user-facing merged PR since last tick to `docs/changelog.mdx` under `## Unreleased`; (4) opens one daily PR with the lock refresh + changelog append | Daily cron 00:00 UTC, `workflow_dispatch` (incl. `just backfill <DATE>`) |
| `core-architect` | `engine/core/` — trait, manifest schema, normalizers, error hierarchy, models. Designs the unified trait shape, scaffolds it, and writes the proposal as the PR body itself (no separate proposal-issue step). | Dispatched by `orchestrator` when a changelog entry is classified as `overlap-opportunity` (or for cross-cutting refactors) |
| `kalshi-maintainer` | `engine/exchanges/kalshi/` (excluding `auth.rs`) and Kalshi entries in `engine/core/src/exchange/manifests/kalshi.rs` | Dispatched by `orchestrator` on a Kalshi changelog entry classified as `critical-exchange-specific`, or on a `(kalshi, <method>)` describe()-scan hit (per `runbooks/parity-gap-closure.md`) |
| `polymarket-maintainer` | All of `engine/exchanges/polymarket/` (including funds-moving files; CODEOWNERS forces human review on those) and Polymarket entries in manifests + the contracts snapshot | Dispatched by `orchestrator` on a Polymarket changelog entry classified as `critical-exchange-specific`, or on a `(polymarket, <method>)` describe()-scan hit (per `runbooks/parity-gap-closure.md`) |

## How dispatch works

`orchestrator` is the only agent that fans work out. The daily cycle:

1. Run `python3 maintenance/scripts/check_docs_drift.py --json` — fetches both upstream changelogs, diffs against `maintenance/scripts/exchange-docs.lock.json`, returns the unified diff per exchange.
2. For each new `<Update>` block in the diff, run the surface-area protocol in `orchestrator.md` Step 2 (mechanical `rg`-grounded checks) and classify as `overlap-opportunity` (→ dispatch `core-architect` to scaffold the trait and write the proposal in the PR body), `critical-exchange-specific` (→ dispatch the relevant maintainer to open a PR), or `no-surface-area` (→ skip with `rg`-evidence quoted in the handoff and PR body — no prose-only skips).
3. Read both exchanges' `describe()` impls. For each `has_<method>: false` line without an `// intentionally unsupported:` marker, dispatch the relevant maintainer to either implement the method or add the marker.
4. Query merged PRs since the last commit that touched `docs/changelog.mdx`. For each user-facing PR, append one bullet under `## Unreleased`. Skip pure-mechanical PRs (regen, CI, agent config).
5. Refresh the lock and open one `chore(daily): refresh changelog lock + append openpx changelog for <DATE>` PR.
6. End with the standard handoff message.

Each dispatch is its own concern → its own Task call → its own PR. Never bundle.

## Triggers

- `agent-tick.yml` — daily cron at 00:00 UTC + `workflow_dispatch`. The `workflow_dispatch` form takes optional `mode` and `since` inputs to support `just backfill <DATE>` (re-process every changelog entry dated on/after `since`).

## Invariants every agent honors

- **Single-purpose PRs.** One concern per PR. The orchestrator fans out multi-item drift; maintainers refuse to bundle.
- **Structured PR body.** What changed / Why / Files / Tests / Review focus. Reviewers use this to scan in under 60 seconds.
- **Reviewer requested explicitly.** Every PR runs `gh pr edit --add-reviewer MilindPathiyal` so the human gets a GitHub review-request email.
- **Never merge.** Agents `gh pr create` but never `gh pr merge`. Humans always merge.
- **Never bypass CI.** No `--no-verify`, no `--no-gpg-sign`, no skipping of any pre-commit or commit-msg hook.
- **Never edit human-only paths.** CODEOWNERS and `.github/REVIEW_POLICY.md` define these. Each agent's prompt also names them explicitly so a misbehaving agent fails fast at prompt level, not just at CODEOWNERS.
- **Never open a PR without completing `maintenance/runbooks/pr-preflight.md`.** Every bot PR keeps the Rust core, Python SDK, TypeScript SDK, and docs in sync, and every SDK actually builds and imports cleanly. CI gates `SDK Sync Check`, `Python SDK Build`, and `Node.js SDK Build` mechanically backstop this. If a preflight step can't run because of missing tooling, the agent stops and comments on the orchestrator's daily PR — it does not invent a justification to skip.
- **Never open a duplicate PR for an already-dispatched concern.** The orchestrator runs a `gh pr list --state open --search ...` dedup pre-flight before every dispatch (Steps 2a / 3a in `orchestrator.md`) and before opening the daily lock-refresh PR (Step 5). If the prior cycle's PR is still open, the orchestrator updates it instead of stacking a new one.

## Files

- `HANDOFF.md` — exit-message contract every agent uses
- `orchestrator.md`, `core-architect.md`, `kalshi-maintainer.md`, `polymarket-maintainer.md` — agent definitions
- `../runbooks/` — procedural checklists agents read at startup

## See also

- `.github/CODEOWNERS` — mechanical enforcement of human-only paths
- `.github/REVIEW_POLICY.md` — written review policy, label taxonomy
- `/Users/mppathiyal/.claude/plans/just-so-i-can-rustling-planet.md` — the full design
