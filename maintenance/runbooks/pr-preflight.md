# Runbook: PR preflight (mandatory for every PR-opening agent)

Followed by **every** agent before running `gh pr create`. No exceptions — `orchestrator`, `core-architect`, `kalshi-maintainer`, `polymarket-maintainer`, and any future PR-opening agent must complete this checklist to its conclusion.

The rule this runbook enforces: **every bot PR keeps the Rust core, Python SDK, TypeScript SDK, and docs in sync, AND every SDK actually builds and imports cleanly.** No partial syncs, no "regen this before merge" caveats, no "intentionally untouched" carve-outs for any one SDK or for docs.

## Why this exists

PR #26 (`feat/core-fetch-server-time`) shipped with `has_fetch_server_time` only in Rust + Python; the TS SDK was skipped with the false claim "TS `models.d.ts` is a hand-rolled stub upstream and intentionally untouched." Two things failed at once:

- The bot didn't have `just` in its sandbox and silently invented a justification to skip TS regen instead of stopping.
- The CI gate's `git diff --exit-code` against `models.d.ts` passed because the codegen had been producing a degenerate empty stub for an indeterminate amount of time — the gate was green for a problem that had been silently present.

This preflight + the CI hardening (`sdk-sync` smoke checks + new SDK build jobs) together close both holes.

## The checklist

Run every step. If any step fails for a reason other than expected regen drift, **stop and escalate** — do not open the PR.

### 1. Rust gauntlet (scoped to your edit)

```
cargo fmt --all --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p px-core --test manifest_coverage
```

Polymarket maintainers also run:

```
cargo test -p px-exchange-polymarket --test contracts_test
```

### 2. Sync regen — produce all four artifacts in one shot

```
just sync-all
```

This regenerates `schema/openpx.schema.json`, `sdks/python/python/openpx/_models.py`, `sdks/typescript/types/models.d.ts`, and `docs/reference/types.mdx` together. Every diff this produces is part of your PR — stage all four files even if only one looks like it changed.

### 3. Sync gate — assert the artifacts are coherent

```
just check-sync
```

This runs the regen and `git diff --exit-code` against the four artifact paths. After step 2 it must pass. If it doesn't, you committed mid-regen — restart from step 2.

### 4. SDK smoke checks — confirm the regen actually produces working code

The regen can produce a syntactically degenerate file that drifts undetectably (this is what hid PR #26's TS gap). These smoke checks are fast and catch it:

```
python -m py_compile sdks/python/python/openpx/_models.py
cd sdks/typescript && npx tsc --noEmit types/models.d.ts
```

Both must produce no diagnostics.

### 5. SDK builds — confirm the bindings still compile against the current Rust core

```
just python-build      # maturin develop --release
just node-build        # napi build --release
```

Each produces a native binary that imports the regenerated models. Failures here usually mean a Rust-side change broke the PyO3 / NAPI binding layer — fix the binding, do not skip.

If you have built locally before in this sandbox, `cargo`'s incremental cache makes these quick (~30s each). First-time builds are 5–10 min each; that's expected.

### 6. SDK imports — confirm the package surface is loadable end-to-end

```
cd sdks/python && python -c "import openpx; print(openpx.__version__)"
cd sdks/typescript && node -e "const o = require('./index.js'); console.log(Object.keys(o).sort())"
```

A green build with a broken import means the package metadata or entry point drifted — investigate before opening the PR.

### 7. Docs surface — confirm any user-facing concept is documented

`just docs` (run by `sync-all`) regenerates `docs/reference/types.mdx` from the Rust schema. That covers type-level changes automatically.

For changes that introduce or rename a **user-facing concept** — a new trait method, a new error class, a new model, a renamed field, a new exchange-level capability — also check whether a prose page in `docs/` (a guide, a tutorial, an exchange-specific notes page) needs an update. If yes, update it in this PR. The reference page alone is not enough for a feature; users read prose.

If you are uncertain whether a prose page needs updating, ask the orchestrator (or comment on the source issue) before opening the PR.

### 8. Provenance — every PR body starts with the source line

Per `.claude/agents/orchestrator.md`:

```
Closes #<N>                                       ← when a single source issue exists
Triggered by: weekly drift cycle (run <run-id>)
Triggered by: daily describe()-scan dispatch (run <run-id>)
Triggered by: PR-merged changelog (PR #<N>)
Triggered by: scheduled SDK + docs regen (run <run-id>)
```

Without this line the orchestrator will comment-block the PR; do not open without it.

## Hard rules

- **If `just`, `maturin`, `napi`, `tsc`, or any other required tool is missing in your sandbox, DO NOT open the PR.** Comment on the source issue with the exact tool that's missing and the step that failed, and stop. Never invent a justification to skip a step. Never write a PR body that says "please regen before merge" — the bot's job is to do the regen, not punt it to the human.
- **If `just check-sync` fails for any reason other than expected regen drift on the four tracked artifacts, STOP and escalate.** Drift on a fifth file (a copy of the schema somewhere, a hand-rolled type definition) is a sync-pipeline bug, not a license to commit the partial state.
- **Never open a PR with the artifacts in different states.** Schema changed but `_models.py` wasn't regenerated → not a PR. `_models.py` regenerated but `models.d.ts` wasn't → not a PR. Either all four are coherent or you don't open the PR.
- **Never claim a step "passed" that you didn't run.** The CI gates exist as a safety net, not as a substitute for actually running the preflight. If you skipped a step because of tooling, say so explicitly in your handoff — don't quietly omit it from the PR body's test plan.

## When this checklist applies

To **every** PR opened by an agent, including:

- Trait / model / manifest changes (`core-architect`)
- Changelog-driven exchange updates and `(exchange, method)` describe()-scan dispatches (maintainers)
- The cross-cutting `chore: regen SDK + docs` PR opened by `orchestrator` after the weekly tick
- The `chore(docs): changelog #<N>` PR opened by `orchestrator` on PR-merge
- Any other PR a future agent might open

Edits that touch only non-Rust paths (e.g., a typo in a guide) will produce a no-op `just sync-all` — that's expected and fast. Run the checklist anyway; the cost is seconds and the consistency is worth it.

## Verification (for the human reviewer reading the PR)

A correct preflight run is visible in the PR body's `## Tests` (or `## Test plan`) section. Look for these lines:

```
- cargo fmt + clippy + test + manifest_coverage: pass
- just check-sync: clean
- python -m py_compile sdks/python/python/openpx/_models.py: clean
- npx tsc --noEmit sdks/typescript/types/models.d.ts: clean
- just python-build: pass; `import openpx` smoke OK
- just node-build: pass; `require('./index.js')` smoke OK
- docs/reference/types.mdx: regenerated; prose pages reviewed (none required | <list>)
```

If any of these is missing or qualified ("not run because…"), the PR should bounce back to the bot with a comment requesting the missing step.
