# Runbook: changelog-driven update

Followed by `kalshi-maintainer` or `polymarket-maintainer` when the orchestrator's daily changelog cycle classifies a changelog entry as `critical-exchange-specific` and dispatches it to you.

The upstream changelog is the single drift signal. When an entry needs more detail than the changelog body provides (full request/response shape, exact param types, error codes), `WebFetch` the exchange's machine-readable specs as a reference — never as a drift source:

- Kalshi: `https://docs.kalshi.com/openapi.yaml` (REST), `https://docs.kalshi.com/asyncapi.yaml` (WebSocket)
- Polymarket: no machine-readable specs; the linked doc page on `https://docs.polymarket.com` is the reference

## Inputs

- The orchestrator's dispatch message, which contains:
  - The exchange (`kalshi` | `polymarket`)
  - The single changelog entry (the `<Update label="..." ...>` block) you must respond to
  - The classification reason (e.g. "Polymarket CLOB V2 cutover — mandatory breaking migration")

## Steps

### 1. Read the entry carefully

`<Update>` blocks contain:
- A `label` (date, sometimes a version)
- Tags (`["New Feature"]`, `["Breaking", "Upcoming"]`, `["Deprecation"]`, etc.)
- Body markdown describing what changed
- Often a list of affected endpoints / modules

For Kalshi entries that name an endpoint or schema, fetch the relevant section of `openapi.yaml` or `asyncapi.yaml` for the exact shape. For Polymarket entries, follow any URL the entry links to (migration guide, contracts page, doc page).

### 2. Confirm scope

Identify exactly which OpenPX files this change touches. Typical patterns:

| Entry shape | OpenPX impact |
|---|---|
| New endpoint (e.g. "added `GET /server-time`") | If the unified trait method already exists (look for `fetch_<thing>` in `engine/core/src/exchange/traits.rs`), implement it on this exchange and flip `has_<method>` to `true`. If the trait method does not exist yet, this is an `overlap-opportunity` for `core-architect`, not a `critical-exchange-specific` for you — comment on the orchestrator's daily PR with what you found and exit `status: blocked`. |
| Renamed field on an existing endpoint | Update `field_mappings` in `engine/core/src/exchange/manifests/<id>.rs`. |
| Removed field | Drop the `FieldMapping` entry. If a unified model field is no longer fillable, escalate to `core-architect`. |
| New optional field | Add to `field_mappings` if it maps to the unified Market/Order/etc.; otherwise to `maintenance/manifest-allowlists/<id>.txt` with a one-line comment. |
| Breaking signature change on existing endpoint | Update `exchange.rs` parsing + `field_mappings`. Body of the changelog entry usually lists the old and new shapes. |
| On-chain contract redeployment (Polymarket) | Funds-moving change — see "Special case: Polymarket contract redeployment" section below. |
| Auth flow change | Stop. `auth.rs` is human-only. Comment on the dispatch with what you found; the human takes it. |
| Service-level change (new service appeared, e.g. Polymarket adds a 5th service) | Stop. New service onboarding is a human decision. Comment and exit. |

If the entry doesn't fit any of the above, comment on the orchestrator's lock-refresh PR with what you found and exit with `status: blocked` — don't guess.

### 3. Apply the changes

Edit the relevant files in your scope:

- `kalshi-maintainer`: `engine/exchanges/kalshi/src/` (excluding `auth.rs`), `engine/core/src/exchange/manifests/kalshi.rs`, `maintenance/manifest-allowlists/kalshi.txt`.
- `polymarket-maintainer`: `engine/exchanges/polymarket/src/` (all of it including funds-moving files), `engine/core/src/exchange/manifests/polymarket.rs`, `maintenance/manifest-allowlists/polymarket.txt`, `maintenance/snapshots/polymarket-contracts.snapshot.json`.

Reuse existing `Transform` variants in manifest entries (`Direct`, `CentsToDollars`, `Iso8601ToDateTime`, etc.). New `Transform` variants are core-architect work — escalate, don't add one yourself.

Wrap any new HTTP call in:

```rust
timed!(
  "openpx.exchange.http_request_us",
  "exchange" => self.id(),
  "operation" => "<method-name>";
  ...
)
```

### 4. Run the local Rust gauntlet

```
cargo test -p px-exchange-<id>
cargo test -p px-core --test manifest_coverage
cargo clippy -p px-exchange-<id> -- -D warnings
```

Polymarket also runs:

```
cargo test -p px-exchange-polymarket --test contracts_test
```

All must pass.

### 5. Complete the preflight

Before `gh pr create`, complete `maintenance/runbooks/pr-preflight.md` to its conclusion: `just sync-all`, `just check-sync` clean, `python -m py_compile` + `tsc --noEmit` smoke checks, Python and Node SDK builds, smoke imports, docs check.

If any preflight step fails because of missing tooling in your sandbox, **do NOT open the PR** — comment on the orchestrator's lock-refresh PR with the exact failure and exit `status: blocked`.

### 6. Open the PR

Conventional-commit title:

- `feat(<id>): support <new endpoint or field>` for additive changes
- `fix(<id>): handle renamed <old> -> <new> field` for renames
- `chore(<id>): drop removed <field>` for removals
- `feat(<id>)!: <breaking change description>` (with the `!`) for breaking changes — and label `breaking-change`

PR body MUST start with:

```
Triggered by: daily changelog cycle (run <run-id>) — <exchange> changelog entry "<label>"
```

The rest of the body uses the maintainer template (What changed / Why / Files / Tests / Review focus). The "Why" must link to the upstream changelog URL and quote the relevant `<Update>` block.

### 7. Request reviewer

```
gh pr edit <PR> --add-reviewer MilindPathiyal
```

### 8. Watch CI

Per `runbooks/pr-ci-watch.md`. Up to 3 fix attempts. Submit `status: success` when CI is green or `status: blocked` with detailed Notes if you can't unstick it.

### 9. Submit handoff

In `Notes`:
- The changelog entry's label and date.
- The categorization decision you made.
- The CI status (`green on attempt N`).

## When to abort instead of finishing

- The entry implies a trait change → escalate to `core-architect`. Comment on the orchestrator's lock-refresh PR with the proposal context; exit `status: blocked`.
- The entry implies a unified-model change → same; escalate.
- The entry touches `auth.rs` → human-only. Comment and exit.
- The entry implies a new service or new exchange → human decision. Comment and exit.
- Any preflight step fails because of missing tooling → comment with the exact failure; do NOT open the PR.

## Special case: Polymarket contract redeployment

A wrong contract address can move user funds to a contract under someone else's control. Verify every address against an external source — never paste blindly from documentation. CODEOWNERS forces human review on `engine/exchanges/polymarket/src/{clob,ctf,relayer,swap,signer,approvals}.rs` and the `contracts_test` snapshot guards source-vs-snapshot drift; the agent drafts confidently because both gates catch a mistake.

When the changelog entry mentions a contract redeployment (CTF Exchange, NegRisk Adapter, USDC.e proxy, CLOB V2 cutover, etc.):

1. **Pull every changed address.** `WebFetch https://docs.polymarket.com/resources/contracts.md` and extract every `0x...` that the entry says changed.

2. **Cross-verify each on Polygonscan.** For each address:
   - `WebFetch https://polygonscan.com/address/<address>`
   - Confirm: contract is deployed; deployer matches a known Polymarket multisig; recent activity matches the contract's stated purpose.
   - Note the deployment block + the tx hash that deployed it — these go in the snapshot's `purpose` field as provenance.

3. **Update `maintenance/snapshots/polymarket-contracts.snapshot.json`:**
   - Update `address` for each changed constant.
   - Update `_last_verified` to today's date.
   - Add provenance to `purpose` (e.g. `"CLOB V2 deployment, 2026-04-28; Polygonscan: <url>; deployed at block <N>, tx <hash>"`).
   - Add new constants if the redeployment introduced any; remove constants that the redeployment removed.

4. **Update the source.** Edit the corresponding `engine/exchanges/polymarket/src/*.rs` files to match the snapshot:
   - `approvals.rs` for `USDC_ADDRESS`, `CTF_ADDRESS`, `CTF_EXCHANGE`, `NEG_RISK_CTF_EXCHANGE`, `NEG_RISK_ADAPTER`
   - `swap.rs` for `NATIVE_USDC_ADDRESS`, `BRIDGED_USDC_E_ADDRESS`, `UNISWAP_V3_ROUTER`
   - Any other file the snapshot's `file` field points to

5. **Source + snapshot land in the SAME PR.** Splitting them guarantees `contracts_test` fails. `cargo test -p px-exchange-polymarket --test contracts_test` must pass before you `gh pr create`.

6. **PR labels:** add `requires-human-careful-review` + `area:onchain`. Title: `feat(polymarket)!: migrate to <name>` for major migrations (the `!` marks breaking) or `chore(polymarket): update contract <name> address` for single-contract redeploys.

7. **PR body's `## Review focus` lists every changed address with its Polygonscan URL.** Reviewers eyeball each address against Polygonscan before merging.

**NEVER:** bypass `contracts_test` (`#[ignore]`, `#[cfg(skip)]`), edit a contract address without the matching snapshot change, or merge yourself.

**Escalate** (comment on the orchestrator's daily PR + exit `status: blocked`) if:
- The new addresses' deployer doesn't match a known Polymarket multisig.
- A "deployed" address shows no recent Polygonscan activity (could be docs typo or pre-launch contract).
- The redeployment removes a contract you can't find a replacement for in the docs.
