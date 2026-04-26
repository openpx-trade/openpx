# Runbook: Polymarket contract redeployment

Followed by `polymarket-maintainer` when:

- The drift report flags a Tier 1 hash change on `https://docs.polymarket.com/resources/contracts.md`, OR
- `cargo test -p px-exchange-polymarket --test contracts_test` fails on a PR, OR
- `https://docs.polymarket.com/changelog` announces an upcoming or completed contract redeployment

**This runbook covers the 2026-04-28 CLOB V2 + pUSD migration.** Same procedure: verify, snapshot, edit, gate, request review.

## Why this runbook is special

Funds-moving. A wrong contract address can move user funds to a contract under someone else's control. **Verify every address against an external source.** Never paste blindly from documentation.

The agent does the typing; the human reviews and merges. CODEOWNERS forces human review on every file in `engine/exchanges/polymarket/src/{clob,ctf,relayer,swap,signer,approvals}.rs`. The `contracts_test` snapshot guards against drift between source and snapshot. Together, these mean the agent can draft the change confidently — every wrong address either fails the test or is caught by human review.

## Steps

1. **Fetch the new contracts page.** `WebFetch https://docs.polymarket.com/resources/contracts.md`. Pull every `0x...` address.

2. **Cross-verify each address on Polygonscan.** For each address you intend to record:
   - `WebFetch https://polygonscan.com/address/<address>`
   - Confirm: contract is deployed; deployer matches a known Polymarket multisig; recent activity is consistent with the contract's stated purpose.
   - Note the deployment block + the tx hash that deployed it. These go in the snapshot file as provenance, in the `purpose` field.

3. **Diff against the current snapshot.** Read `maintenance/snapshots/polymarket-contracts.snapshot.json`. Identify which constants moved.

4. **Update the snapshot.** Edit `maintenance/snapshots/polymarket-contracts.snapshot.json`:
   - Update `address` for each changed constant
   - Update `_last_verified` to today's date
   - Add provenance to the `purpose` field (e.g. "CLOB V2 deployment, 2026-04-28; Polygonscan: <url>; deployed at block <N>, tx <hash>")
   - Add new constants if the redeployment introduced any
   - Remove constants if the redeployment removed any (and follow up by removing them from source)

5. **Update the source.** Edit the corresponding `engine/exchanges/polymarket/src/*.rs` files to match the snapshot:
   - `approvals.rs` for `USDC_ADDRESS`, `CTF_ADDRESS`, `CTF_EXCHANGE`, `NEG_RISK_CTF_EXCHANGE`, `NEG_RISK_ADAPTER`
   - `swap.rs` for `NATIVE_USDC_ADDRESS`, `BRIDGED_USDC_E_ADDRESS`, `UNISWAP_V3_ROUTER`
   - Any other file the snapshot points to via the `file` field

6. **Run the local test gauntlet:**
   ```
   cargo test -p px-exchange-polymarket --test contracts_test
   cargo test -p px-exchange-polymarket
   cargo clippy -p px-exchange-polymarket -- -D warnings
   ```
   The contracts_test must pass — that's the proof you matched source and snapshot correctly.

7. **Open a single PR** containing both the snapshot update and the source edits. Conventional-commit title:
   - `feat(polymarket)!: migrate to CLOB V2 / pUSD` for major migrations (the `!` marks breaking)
   - `chore(polymarket): update contract <name> address` for single-contract redeploys

   Body uses the maintainer template. Add `requires-human-careful-review` + `area:onchain` labels. The body's `Review focus` section must list every changed address with the Polygonscan URL.

8. **Request reviewer:** `gh pr edit <PR> --add-reviewer MilindPathiyal`.

9. **Submit handoff** with status `success` (you opened a PR; the human merges). In `Notes`, list every changed address with the Polygonscan URL again — redundant with the PR body but easier for the orchestrator to log.

## What you must NEVER do

- Update the snapshot without Polygonscan verification of every changed address.
- Merge the PR yourself — `gh pr create` only.
- Bypass `contracts_test` (`#[ignore]`, `#[cfg(skip)]`, etc).
- Hand-edit a contract address inline without updating the snapshot in the same PR.
- Commit the source change without the snapshot change, or vice versa — they must land together for `contracts_test` to stay green.

## When to escalate instead

- The new addresses' deployer doesn't match a known Polymarket multisig — stop, comment on the drift issue with what you found, request human investigation.
- Polygonscan shows no recent activity on a "deployed" address — could be a typo in the docs page or a pre-launch contract; flag for human review.
- The redeployment removes a contract you can't find a replacement for in the docs — comment on the drift issue requesting clarification before editing.
