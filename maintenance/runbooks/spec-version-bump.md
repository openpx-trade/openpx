# Runbook: spec-version bump

Followed by `kalshi-maintainer` or `polymarket-maintainer` when the drift report flags a Tier 1 hash change on an upstream documentation page.

## Inputs

- The drift artifact (`docs-drift.yml` JSON) — names the URL and old/new hashes
- Your dispatcher's instructions (which exchange, which page)

## Tier 1 pages by exchange

**Kalshi:**

- `https://docs.kalshi.com/openapi.yaml` — REST spec, `info.version` bumps trigger this
- `https://docs.kalshi.com/asyncapi.yaml` — WebSocket spec
- `https://docs.kalshi.com/changelog.md`
- `https://docs.kalshi.com/llms.txt`

**Polymarket:**

- `https://docs.polymarket.com/changelog.md`
- `https://docs.polymarket.com/api-reference/introduction.md`
- `https://docs.polymarket.com/api-reference/authentication.md`
- `https://docs.polymarket.com/resources/contracts.md` — see `contract-redeployment.md` instead
- `https://docs.polymarket.com/llms.txt`

## Steps

1. **Fetch the new content.** `WebFetch <url>` and read it. For YAML specs, fetch the raw YAML and find the `info.version` field at the top — confirm the bump.

2. **Diff against your last-seen state.** For YAML specs, compare against the version stored in `maintenance/scripts/exchange-docs.lock.json::specs.<file>.version`. For prose pages, compare against the body hash; you may need to fetch your previous understanding from the latest merged docs PR or from the lock file.

3. **Categorize the change** (the repo has no external users yet, so backward-compat is not a goal — choose the cleanest expression):
   - **New field:** add to `field_mappings` if it maps to the unified Market/Order/etc.; otherwise to the per-exchange allowlist with a one-line comment.
   - **Renamed field:** update the existing `FieldMapping.source_paths` to the new name. Don't keep the old name as a fallback — drop it.
   - **Removed field:** drop the `FieldMapping` entry. If a unified `Market`/`Order`/etc. field is no longer fillable from the exchange, dispatch `core-architect` to remove or restructure that unified field.
   - **New endpoint that supports a previously-`NotSupported` trait method:** dispatch `core-architect` to confirm the trait surface is right; then implement the method per `parity-gap-closure.md`.
   - **Removed endpoint:** if your exchange was implementing a unified method via this endpoint, the method now stays `NotSupported` here. Update `describe()` to flip the `has_<method>` flag back to `false`.
   - **Semantics-only (e.g. clarification of behaviour without API change):** review existing implementation against new wording; may not require code change. If implementation already matches new wording, the only change is refreshing the lock file.

4. **Apply the manifest change.** Edit `engine/core/src/exchange/manifests/<id>.rs`. Reuse existing `Transform` variants (`Direct`, `CentsToDollars`, `Iso8601ToDateTime`, `UnixSecsToDateTime`, `ParseInt`, `ParseFloat`, `JsonArrayIndex`, `NestedPath`) — propose a new one only if no existing variant fits, and even then file an issue rather than adding it yourself (transforms are core).

5. **Apply the exchange.rs change** (if needed). Read the field, parse it, slot it into the unified model. Wrap any new HTTP call in `timed!("openpx.exchange.http_request_us", "exchange" => self.id(), "operation" => "<method>"; ...)`.

6. **Refresh the lock file.** Run:
   ```
   python3 maintenance/scripts/check_docs_drift.py --update --exchange <id>
   ```
   Note: `--update` writes the entire lock — make sure your local checkout has no other unintended drift before running.

7. **Run the local test gauntlet:**
   ```
   cargo test -p px-exchange-<id>
   cargo test -p px-core --test manifest_coverage
   cargo clippy -p px-exchange-<id> -- -D warnings
   ```
   All must pass.

8. **Open the PR.** Use the maintainer's structured PR-body template. Conventional-commit title:
   - `chore(<id>): track upstream spec v<X.Y.Z>` for version-only bumps with no behaviour change
   - `feat(<id>): support new <description> field` for additive changes
   - `fix(<id>): handle renamed <old> -> <new> field` for renames

9. **Request reviewer:** `gh pr edit <PR> --add-reviewer MilindPathiyal`.

10. **Submit handoff.** Include in `Notes` which Tier 1 page changed, the version delta if any, and the categorization decision you made.

## Verification

The PR's CI must run green on:

- `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `manifest-coverage`
- `version-sync`, `sdk-sync`

If `manifest-coverage` fails, you read a new JSON key without declaring it. Add to `field_mappings` (preferred) or to `<id>_allowlist.txt` (only if not part of the unified Market schema).

If `sdk-sync` fails, your change touched a `#[derive(JsonSchema)]` type. Run `just sync-all` and commit the regenerated artifacts in the same PR.

## Single-purpose rule reminder

If the diff reveals multiple unrelated changes (a version bump *and* a renamed field on a different endpoint), stop and ask the orchestrator to split. Open one PR per concern.
