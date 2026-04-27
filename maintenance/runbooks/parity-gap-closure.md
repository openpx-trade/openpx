# Runbook: parity-gap closure

Followed by an exchange-maintainer when the orchestrator's daily `describe()`-flag scan dispatches you to handle a `has_<method>: false` line on your exchange. The trait method has been scaffolded (typically by `core-architect` in response to an `overlap-opportunity` changelog entry); your job is to either implement it or mark it as intentionally unsupported.

## Inputs

The orchestrator's dispatch message names:
- The exchange (`kalshi` | `polymarket`)
- The trait method (e.g., `fetch_server_time`)
- A pointer to the trait scaffolding PR that introduced it (so you can read the unified shape)

## Decision: implement or mark intentionally unsupported

First read the upstream API for your exchange. One of two outcomes:

- **An equivalent endpoint exists** → implement the method. Continue to the implementation steps.
- **No equivalent exists, and there's no near-term path to one** → mark the flag with a single-line comment. Open a small PR that just edits `engine/exchanges/<id>/src/exchange.rs::describe()` to add the marker comment above (or trailing) the existing `has_<method>: false` line:

  ```rust
  // intentionally unsupported: <one-sentence reason — e.g. "Polymarket has no server-time endpoint">
  has_<method>: false,
  ```

  PR title: `chore(<id>): mark <method> intentionally unsupported`. Body provenance: `Triggered by: daily describe()-scan dispatch (run <run-id>)`. After `gh pr create`, apply the dedup label `gh pr edit <PR> --add-label parity/<exchange>/<method>`. Complete `pr-preflight.md`. Done — handoff with `status: success`.

The marker comment is the signal to the orchestrator's next describe()-scan that this `(exchange, method)` pair is settled and should not be re-dispatched.

## Steps to implement

1. **Read the trait scaffolding PR** named in your dispatch. The PR body contains the proposal — request/response types, error mapping notes, naming rationale. Also read `engine/core/src/exchange/traits.rs` for the post-merge trait shape.

2. **Read the reference implementation** on the *other* exchange (the one that already implements the method). Pattern-match the request building, response parsing, error mapping. Don't blindly copy — exchanges have different conventions — but use the structural shape.

3. **Read the upstream API docs** for your exchange's equivalent endpoint. Confirm:
   - Endpoint exists (URL, HTTP method, path/query params)
   - Request format
   - Response shape
   - Error responses
   - Rate-limit category

4. **Implement the method:**
   - Wrap the HTTP call in `timed!("openpx.exchange.http_request_us", "exchange" => self.id(), "operation" => "<method-name>"; ...)`.
   - Use `define_exchange_error!`-defined variants for exchange-specific errors; map them via the existing `From<<Id>Error> for px_core::ExchangeError` impl.
   - If the response shape doesn't fit existing unified models, **stop and escalate** — model changes belong to `core-architect`. Write what you found to `$GITHUB_STEP_SUMMARY` and exit `status: blocked`.
   - If parsing requires reading a JSON key not in the manifest, prefer adding a `FieldMapping` entry over the allowlist (unless the field is genuinely outside the unified Market schema).

5. **Update `describe()`.** Set `has_<method>: true`. Don't lie — only flip the flag if the implementation is actually working, not just compiling.

6. **Add tests:**
   - One happy-path test using `wiremock`.
   - One error-path test covering the most likely upstream failure (404, 401, 429).
   Tests live in `engine/exchanges/<id>/tests/exchange_tests.rs`.

7. **Run the local Rust gauntlet:**
   ```
   cargo test -p px-exchange-<id>
   cargo test -p px-core --test manifest_coverage
   cargo clippy -p px-exchange-<id> -- -D warnings
   ```
   All must pass.

8. **Complete `maintenance/runbooks/pr-preflight.md` to its conclusion.** Flipping `has_<method>: false` → `true` is a schema change — the regen will produce diffs in `schema/openpx.schema.json`, `_models.py`, `models.d.ts`, and `docs/reference/types.mdx` that all land in this same PR.

9. **Open the PR.** Conventional commit: `feat(<id>): implement <method>`. Body provenance:

   ```
   Triggered by: daily describe()-scan dispatch (run <run-id>) — implements <method> on <exchange>; trait scaffolded in PR #<scaffolding-pr-N>
   ```

   Body uses the maintainer template (What changed / Why / Files / Tests / Review focus). Label `area:<id>`.

10. **Apply the dedup label and request reviewer:**
   ```
   gh pr edit <PR> --add-label parity/<exchange>/<method>
   gh pr edit <PR> --add-reviewer MilindPathiyal
   ```
   The label is the orchestrator's dedup key — without it, the next describe()-scan cycle will dispatch a duplicate.

11. **Watch CI per `runbooks/pr-ci-watch.md`.** Up to 3 fix attempts.

12. **Submit handoff.** In `Notes`, mention which exchange's reference implementation you mirrored and any deviations.

## When to abort

- The upstream API doesn't actually have the endpoint → mark intentionally unsupported (the small-PR path above) instead.
- Implementing requires changing the trait signature → write to `$GITHUB_STEP_SUMMARY`, exit `status: blocked` (this is `core-architect`'s next overlap-opportunity dispatch).
- Implementing requires changing a unified model → same; write to step summary + `status: blocked`.
