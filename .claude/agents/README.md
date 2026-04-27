# OpenPX agent roster

Three specialist agents maintain this repo. All run on `claude-opus-4-7` with max reasoning effort. Every PR they open requires explicit human approval — no auto-merge.

## Roster

| Agent | Owns | Triggered by |
|---|---|---|
| `orchestrator` | Daily cycle — (1) diffs the Kalshi + Polymarket changelogs against the per-entry lock and emits one dispatch per actionable entry; (2) scans both exchanges' `describe()` for unimplemented scaffolded methods and emits one dispatch per `has_<method>: false` flag; (3) on drift, opens a single `chore(bot): refresh changelog lock` PR. Never edits Rust source. | Daily cron 00:00 UTC, `workflow_dispatch` (incl. `just backfill <DATE>`) |
| `core-architect` | `engine/core/` — trait, manifest schema, normalizers, error hierarchy, models. Designs the unified trait shape, scaffolds it, and writes the proposal as the PR body itself (no separate proposal-issue step). | Dispatched by `orchestrator` when a changelog entry is classified as `overlap-opportunity` (or for cross-cutting refactors) |
| `exchange-maintainer` | `engine/exchanges/<exchange>/` and the matching `engine/core/src/exchange/manifests/<exchange>.rs` entries. Operates on `kalshi` or `polymarket` per dispatch payload. Includes high-risk files — Polymarket on-chain (`clob/ctf/relayer/swap/signer/approvals.rs`) and Kalshi `auth.rs`. CODEOWNERS forces human review on every PR touching those. | Dispatched by `orchestrator` on a `critical-exchange-specific` changelog entry or a `(exchange, <method>)` describe()-scan hit (per `runbooks/parity-gap-closure.md`) |

## How dispatch works

`orchestrator` is the only agent that fans work out, and it does so by emitting a JSON dispatch list — it does NOT run specialists in-session. The workflow's `dispatch` matrix job then forks one parallel job per dispatch, each running a single specialist (`core-architect` or `exchange-maintainer`) against one concern in its own runner.

Per cycle the orchestrator:

1. Runs `python3 maintenance/scripts/check_docs_drift.py --json` — fetches both upstream changelogs, parses them into per-entry hashed blocks, and returns `{new, amended, removed}` per exchange.
2. For each new or amended `<Update>` block, runs the mechanical surface-area protocol in `orchestrator.md` Step 2 (`rg`-grounded checks against our code) and classifies as `overlap-opportunity` (→ emit a `core-architect` dispatch to scaffold the trait), `critical-exchange-specific` (→ emit an `exchange-maintainer` dispatch), or `no-surface-area` (→ skip with `0 hits` rg evidence quoted in the daily PR body).
3. Reads both exchanges' `describe()` impls. For each `has_<method>: false` line without an `// intentionally unsupported:` marker, emits an `exchange-maintainer` dispatch.
4. Runs the dedup pre-flight: for each prospective dispatch, queries `gh pr list --label cl/<exchange>/<id> --state all` (or `parity/<exchange>/<method>` for parity-gap dispatches). Open PR → comment-and-skip. Merged → silent skip. Closed-not-merged → escalate via `$GITHUB_STEP_SUMMARY`. Empty → emit dispatch.
5. Writes the surviving dispatches to `/tmp/dispatches.json` (consumed by the workflow's matrix job).
6. If any drift was actually detected, refreshes the lock and opens one `chore(bot): refresh changelog lock for <DATE>` PR with the dispatch summary table. **Quiet days (no drift, no parity gaps) exit without opening any PR** — workflow run history is the audit trail.
7. End with the standard handoff message.

Each dispatched specialist appends its own bullet to `docs/changelog.mdx::## Unreleased` in the same PR that lands the change — there is no separate orchestrator step for retroactive changelog appends.

Each dispatch is its own concern → its own matrix job → its own PR. Never bundle.

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
- **Never open a duplicate PR for an already-dispatched concern.** Every dispatched PR carries a label `cl/<exchange>/<id>` (changelog entry) or `parity/<exchange>/<method>` (parity-gap). The orchestrator's pre-dispatch query is `gh pr list --label <label> --state all`. Open → comment-and-skip; merged → silent skip; closed-not-merged → escalate. The lock-refresh PR uses a similar guard so a stale prior cycle's PR is rebased rather than duplicated.

## Files

- `HANDOFF.md` — exit-message contract every agent uses
- `orchestrator.md`, `core-architect.md`, `exchange-maintainer.md` — agent definitions
- `../runbooks/` — procedural checklists agents read at startup

## See also

- `.github/CODEOWNERS` — mechanical enforcement of human-only paths
- `.github/REVIEW_POLICY.md` — written review policy, label taxonomy
- `/Users/mppathiyal/.claude/plans/just-so-i-can-rustling-planet.md` — the full design
