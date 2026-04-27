# Runbook: trait evolution

Followed by `core-architect` when adding to or restructuring `engine/core/src/exchange/traits.rs`, `engine/core/src/exchange/manifest.rs`, or any unified model in `engine/core/src/models/`.

## Inputs

- The orchestrator's dispatch message for an `overlap-opportunity` changelog entry, OR a cross-cutting refactor opportunity flagged by a maintainer.

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

3. **Update `engine/core/src/exchange/traits.rs`'s `ExchangeInfo` struct** with a new `pub has_<method>: bool` field. Default is `false` — maintainers flip to `true` once they implement the method on their exchange (the orchestrator's daily describe()-scan dispatches them).

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

8. **Run the gauntlet, then complete `maintenance/runbooks/pr-preflight.md` to its conclusion:**
   ```
   cargo check --workspace --all-targets
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo test -p px-core --test manifest_coverage
   ```
   The preflight runs `just sync-all`, `just check-sync`, the smoke checks (`python -m py_compile` + `tsc --noEmit`), the Python and Node SDK builds, and the smoke imports. If any preflight step fails because of missing tooling in your sandbox, do NOT open the PR — comment on the source issue with the exact failure and stop.

9. **Commit the regenerated artifacts** in the same PR: `schema/openpx.schema.json`, `sdks/python/python/openpx/_models.py`, `sdks/typescript/types/models.d.ts`, `docs/reference/types.mdx`. They MUST land together — the `sdk-sync`, `Python SDK Build`, and `Node.js SDK Build` CI gates collectively verify this.

10. **Open the PR.** Conventional commit `feat(core): <one-sentence-summary>`. **PR body MUST start with `Triggered by: daily changelog cycle (run <run-id>) — <exchange> changelog entry "<label>" classified as overlap-opportunity`** and contain the proposal as the body itself (per `.claude/agents/core-architect.md`). Label `area:core`.

11. **Request reviewer:** `gh pr edit <PR> --add-reviewer MilindPathiyal`.

12. **Watch CI per `runbooks/pr-ci-watch.md`.** Up to 3 fix attempts.

13. **Submit handoff.** Per-exchange implementation lands later — the orchestrator's next daily `describe()`-scan picks up `has_<method>: false` on each exchange and dispatches the relevant maintainer per `runbooks/parity-gap-closure.md`. No follow-up issues to file from this runbook.

## Steps for refactors

Same shape as above, but the "before" state has multiple exchanges with similar code. Make sure to:

- Move/rename/restructure the helper in one PR (the refactor).
- Verify `cargo bench` shows no regression beyond noise — refactors should preserve performance.
- Per-exchange call-site updates are handled by maintainers in subsequent PRs (via orchestrator dispatch when relevant changelog drift appears, or human-routed if the refactor needs proactive cleanup).

## What you do not do under this runbook

- Per-exchange implementation — maintainers handle it on the next daily orchestrator describe()-scan.
- Anything in `.github/`, release configs, or workspace `Cargo.toml`.
