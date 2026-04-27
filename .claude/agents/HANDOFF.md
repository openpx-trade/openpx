# Agent handoff contract

Every OpenPX subagent ends its run with this exact structure as its final user-visible output. The orchestrator (and humans reviewing the run) parse it to decide what happens next.

```
## Result
- status: success | partial | blocked
- branch: <branch-name> or null
- pr: <https://github.com/.../pull/N> or null
- next: <suggested-next-agent or "none">

## Files touched
- <relative path>: <±lines>
- <relative path>: <±lines>

## Verification
- cargo test --workspace: pass | fail | not-run
- cargo clippy --workspace --all-targets -- -D warnings: pass | fail | not-run
- just check-sync: pass | fail | not-run
- <other relevant cargo/just/python commands>: pass | fail | not-run

## Notes
<freeform — what was inspected, what was skipped, anything the next agent or
human reviewer should know>
```

## Status semantics

- **success** — work completed, PR opened (if applicable), CI green or pending. Next agent / human can proceed.
- **partial** — some progress made, but more work needed. Document exactly what's left in `Notes`.
- **blocked** — could not proceed. State precisely what's needed: human decision, missing data, conflicting drift, etc.

## When status must be `blocked`, never `success`

- Trait change required (`engine/core/src/exchange/traits.rs`)
- Funds-moving Polymarket file edit required
- Kalshi `auth.rs` edit required
- Release config edit required
- New crate added to workspace
- Auto-detected duplicate PR within 24h (per `orchestrator.md` Step 2a / 3a dedup pre-flight)
- `cargo test` or `cargo clippy` failed and fix is non-trivial
- `manifest_coverage` test fails and the new key isn't obviously order/fill/position parsing (escalate so a human picks the right manifest entry vs allowlist)

## Reviewer assignment

Every PR opened in `success` status MUST have run `gh pr edit <PR> --add-reviewer MilindPathiyal`. Confirm in `Notes` that this was done.
