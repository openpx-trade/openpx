# Runbook: parity-gap closure

Followed by an exchange-maintainer when a human approves a parity-analyst proposal and routes implementation to them. Typical case: an `Exchange` trait method that's currently `NotSupported` on one exchange but implemented on the other.

## Inputs

- The proposal issue (parity-analyst-authored) with the human's approval comment
- Your dispatcher's instructions (which method, which exchange)

## Steps

1. **Confirm the gap is real.** Read your exchange's `Exchange` impl in `engine/exchanges/<id>/src/exchange.rs`. The target method should currently:
   - Use the default `NotSupported` impl (no override), OR
   - Have an explicit override that returns `Err(ExchangeError::NotSupported(...))`
   - Have `has_<method>: false` in the `describe()` impl

2. **Read the reference implementation** on the other exchange (the one that already has it). Pattern-match the request building, response parsing, error mapping. Do not blindly copy — exchanges have different conventions — but use the structural shape.

3. **Read the upstream API docs** for your exchange's equivalent endpoint. Confirm:
   - The endpoint exists (URL, HTTP method, path/query params)
   - Request format
   - Response shape
   - Error responses
   - Rate limit category

4. **Implement the method.** Follow these conventions:
   - Wrap the HTTP call in `timed!("openpx.exchange.http_request_us", "exchange" => self.id(), "operation" => "<method-name>"; ...)`.
   - Use `define_exchange_error!`-defined variants for exchange-specific errors; map them via the existing `From<<Id>Error> for px_core::ExchangeError` impl.
   - If the response shape doesn't fit existing unified models, **stop and escalate** — model changes are human-only.
   - If parsing requires reading a JSON key not in the manifest, prefer adding a `FieldMapping` entry over the allowlist (unless the field is genuinely outside the unified Market schema).

5. **Update `describe()`.** Set the corresponding `has_<method>` flag to `true` in the `ExchangeInfo` returned. Don't lie — only flip the flag if the implementation is actually working, not just compiling.

6. **Add tests.** At minimum:
   - One happy-path test using `wiremock` (look at existing tests in `engine/exchanges/<id>/tests/` for the pattern).
   - One error-path test covering the most likely upstream failure (404, 401, 429).
   Tests live in `engine/exchanges/<id>/tests/exchange_tests.rs` or a new file if the existing one is getting too large.

7. **Run the gauntlet:**
   ```
   cargo test -p px-exchange-<id>
   cargo test -p px-core --test manifest_coverage
   cargo clippy -p px-exchange-<id> -- -D warnings
   ```
   All must pass.

7a. **Complete `maintenance/runbooks/pr-preflight.md` to its conclusion.** Flipping `has_<method>: false` → `true` in `describe()` is a schema change — the regen will produce diffs in `schema/openpx.schema.json`, `_models.py`, `models.d.ts`, and `docs/reference/types.mdx` that all must land in this same PR. SDK builds + smoke imports are also part of the preflight; do not skip.

8. **Open the PR.** Conventional commit: `feat(<id>): implement <method>`. Body uses the maintainer template. Label `parity-fill` + `area:<id>`. Reference the proposal issue: `Closes #<proposal-issue-N>`.

9. **Request reviewer:** `gh pr edit <PR> --add-reviewer MilindPathiyal`.

10. **Submit handoff.** In `Notes`, mention which existing exchange's pattern you mirrored and any deviations.

## Verification

CI green on `manifest-coverage`, `clippy`, `test`. After merge:

- `parity-analyst` will pick up the new `has_<method>: true` flag on its next weekly run and remove the gap from `docs/parity/STATUS.md`.
- The proposal issue closes automatically via the PR's `Closes #<N>`.

## When to abort instead of finishing

- The upstream API doesn't actually have the endpoint the proposal assumed — comment on the proposal issue with what you found, set status `blocked`.
- Implementing requires changing the trait signature — that's `core-architect` (deferred); escalate to human, set status `blocked`.
- Implementing requires changing a unified model — same; human-only, set status `blocked`.
