# Runbook: trait evolution

Followed by `core-architect` when adding to or restructuring `engine/core/src/exchange/traits.rs`, `engine/core/src/exchange/manifest.rs`, or any unified model in `engine/core/src/models/`.

## Inputs

- An approved parity-analyst proposal issue, OR a refactor opportunity flagged by a maintainer in a PR review.

## Decision tree

The repo is pre-users; backward compatibility is not a goal. Lean is. Treat all changes the same way — implement them directly. CODEOWNERS forces human review of every `engine/core/` PR; that's the safety net.

| Class | Examples |
|---|---|
| **New surface** | New trait method; new struct type; new error variant; new `Transform` variant; new field on an existing struct |
| **Refactor** | Hoist a helper from two exchanges into `normalizers.rs`; rename a field; collapse two redundant types; remove unused fields |
| **Restructure** | Reshape a method signature; change a `Transform`'s semantics; redesign a model type for a better UX |

## Steps

1. **Edit `traits.rs`** if adding or changing a method. For new methods, use the existing optional-method pattern (lines 64–145):
   ```rust
   /// Description of what this does and when to use it.
   /// Example response shape, gotchas, etc.
   async fn fetch_<thing>(&self, req: <Thing>Request) -> Result<<Thing>Response, OpenPxError> {
       Err(OpenPxError::Exchange(ExchangeError::NotSupported(
           "fetch_<thing> not supported by this exchange".into(),
       )))
   }
   ```
   For renames or signature changes: edit freely. The compiler + tests + your reviewer catch problems.

2. **Add the request/response types** in `engine/core/src/exchange/traits.rs` (or a sibling module if the file is getting long). `#[derive(Debug, Clone, Serialize, Deserialize)]` plus `#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]` so they appear in the auto-generated docs.

3. **Update `engine/core/src/exchange/traits.rs`'s `ExchangeInfo` struct** with a new `pub has_<method>: bool` field. Default is `false` — maintainers will flip to `true` in their parity-fill PRs once they implement the method.

4. **Update `engine/sdk/src/lib.rs`:**
   - Add a `dispatch!` arm for the new method (the macro takes the method name).
   - Add a method shim on `ExchangeInner`:
     ```rust
     pub async fn fetch_<thing>(&self, req: <Thing>Request) -> Result<<Thing>Response, OpenPxError> {
         dispatch!(self, fetch_<thing>, req)
     }
     ```
   The compiler will tell you if any other dispatch arm is missing.

5. **Each exchange's `describe()`** in `engine/exchanges/<id>/src/exchange.rs`: add the new `has_<method>: false` field. The maintainers will flip these to `true` in subsequent PRs as they implement.

6. **Add or change a model field?** Edit the relevant file in `engine/core/src/models/`. Use `Option<T>` only when the field is genuinely optional in the data — not for "compat". Rename freely if a better name exists. `#[serde(default)]` if you want absent JSON to be `None`.

7. **Add a manifest `Transform` variant?** Edit `engine/core/src/exchange/manifest.rs`. Update `engine/core/src/exchange/normalizers.rs::apply_transform` to handle it.

8. **Run the gauntlet:**
   ```
   cargo check --workspace --all-targets
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo test -p px-core --test manifest_coverage
   just sync-all
   ```

9. **Commit the regenerated artifacts** in the same PR: `schema/openpx.schema.json`, `sdks/python/python/openpx/_models.py`, `sdks/typescript/types/models.d.ts`, `docs/reference/types.mdx`. They MUST land together — the `sdk-sync` CI gate verifies this.

10. **Open the PR.** Conventional commit `feat(core): <one-sentence-summary>`. **PR body MUST start with `Closes #<proposal-N>`** so the originating proposal auto-closes on merge. Label `area:core` + the `parity-fill` label if this closes a parity proposal.

11. **Request reviewer:** `gh pr edit <PR> --add-reviewer MilindPathiyal`.

12. **File follow-up parity-fill issues** — one per exchange whose `describe()` flag you set to `false`. Use the explicit "this is impl follow-up, not a duplicate proposal" template:

    ```
    Title: [parity-fill] {exchange}: implement {method} (proposal #{N}, scaffolding PR #{M})

    Body:
    Implementation task for the `{method}` unified trait method.

    - Original proposal: #{N}
    - Trait scaffolding: PR #{M} (closes #{N} on merge)
    - Runbook: `maintenance/runbooks/parity-gap-closure.md`

    When the assignee picks this up, change `has_{method}: false` to `true` in
    `engine/exchanges/{exchange}/src/exchange.rs::describe()` and replace the
    default `NotSupported` impl with a real one that hits the upstream endpoint.

    cc @{exchange}-maintainer
    ```

    Labels: `parity-fill`, `area:{exchange}`, `enhancement`. Assignee: `openpx-bot` (every `gh issue create` MUST include `--assignee openpx-bot`). Run dedup pre-flight (`gh issue list --search` for the same `{method}` and `{exchange}`) before creating each one.

13. **Comment on the proposal issue** noting where the impl is tracked. Format:

    ```
    Trait scaffolding ready in PR #{M}. Per-exchange implementation:
    - kalshi: #{kalshi-followup}
    - polymarket: #{polymarket-followup}
    ```

## Steps for refactors

Same shape as above, but the "before" state has multiple exchanges with similar code. Make sure to:

- Move/rename/restructure the helper in one PR (the refactor).
- Update each exchange's call sites in subsequent PRs (one per maintainer; route via the orchestrator).
- Verify `cargo bench` shows no regression beyond noise — refactors should preserve performance.

## What you do not do under this runbook

- Per-exchange implementation. The maintainers do that as parity-fills.
- Anything in `.github/`, release configs, or workspace `Cargo.toml`.
