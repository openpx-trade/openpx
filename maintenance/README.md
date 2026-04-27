# `maintenance/` — autonomous-maintenance content

Everything related to OpenPX's autonomous-maintenance system lives under this single tree. The unified API code (`engine/`, `sdks/`, `docs/`, `schema/`) stays unaffected.

## Layout

```
maintenance/
├── README.md                          # this file
├── runbooks/                          # imperative checklists agents read at startup
│   ├── README.md
│   ├── changelog-driven-update.md     # maintainer response to a critical-exchange-specific changelog dispatch (incl. on-chain redeployment special case)
│   ├── parity-gap-closure.md          # maintainer response to a describe()-scan dispatch (implement or mark)
│   ├── trait-evolution.md             # core-architect's playbook for trait/model changes
│   ├── pr-preflight.md                # mandatory checklist before any gh pr create
│   └── pr-ci-watch.md                 # post-PR-create CI watch + fix protocol
├── scripts/                           # Python scripts the workflows + just recipes call
│   ├── check_docs_drift.py            # diff Kalshi + Polymarket changelogs vs the lock
│   ├── exchange-docs.lock.json        # per-exchange last-seen changelog body + hash
│   └── generate_mintlify_docs.py      # auto-generate docs/reference/types.mdx from schema
├── policy/                            # written policies (referenced by CODEOWNERS at .github/CODEOWNERS)
│   ├── REVIEW_POLICY.md               # PR review and label policy
│   └── branch-protection.yml          # source-of-truth doc for branch protection rules
├── manifest-allowlists/               # JSON keys read in exchange.rs that aren't in field_mappings
│   ├── kalshi.txt
│   └── polymarket.txt
├── snapshots/                         # vendored snapshots used by tests
│   └── polymarket-contracts.snapshot.json
└── tests/                             # maintenance assertions wired into Cargo via [[test]] entries
    ├── manifest_coverage.rs           # gates engine/core/src/exchange/manifests/ vs exchange.rs reads
    └── contracts_test.rs              # gates polymarket on-chain addresses vs the snapshot
```

## Where the agent definitions live

The agent prompt files MUST live at `.claude/agents/` (Claude Code's required path). They reference everything in this `maintenance/` tree. See `.claude/agents/README.md` for the roster.

## Tools that live elsewhere by mandate

These can't move because their tooling expects specific paths:

- `.github/CODEOWNERS` — GitHub mandates this path
- `.github/workflows/agent-tick.yml`, `ci.yml` — GitHub mandates `.github/workflows/`
- `.claude/agents/*.md`, `.claude/Claude.md`, `.claude/settings.json` — Claude Code mandates `.claude/`
- `justfile` — `just` reads from repo root

The maintenance tests at `maintenance/tests/` are wired into Cargo via explicit
`[[test]]` `path` entries in `engine/core/Cargo.toml` and
`engine/exchanges/polymarket/Cargo.toml`. They run via `cargo test -p px-core --test manifest_coverage`
and `cargo test -p px-exchange-polymarket --test contracts_test` exactly as if
they lived under each crate's `tests/` directory.

## Useful commands

```bash
# Run changelog drift detection (fetches both upstream changelogs)
just drift-check

# Refresh the lock file with the current live changelogs
just drift-update

# Regenerate Mintlify reference docs from schema
just docs

# Manually trigger the agent-tick workflow (the same workflow the daily cron runs)
just maintain

# Backfill: re-process every changelog entry dated on/after SINCE
just backfill 2026-02-01

# Run the manifest-coverage test directly
cargo test -p px-core --test manifest_coverage

# Run the contract-snapshot test directly
cargo test -p px-exchange-polymarket --test contracts_test
```

## What this system protects

- **Unified schema mapping correctness.** `manifest_coverage` test forbids reading any JSON key in `exchange.rs` that isn't either declared in the manifest or allowlisted with a justification.
- **Polymarket contract address correctness.** `contracts_test` snapshot forbids any source-code address change without a paired snapshot update (and the runbook requires Polygonscan verification).
- **Public API contract.** CODEOWNERS forces human review on every change to `engine/core/`, `engine/sdk/`, funds-moving Polymarket files, `engine/exchanges/kalshi/src/auth.rs`, `.github/`, release configs, and credentials.
- **Mintlify documentation 1-1 with code.** `just docs` regenerates `docs/reference/types.mdx` from the JSON schema (which is auto-built from Rust `#[derive(JsonSchema)]` annotations); `check-sync` CI fails if generated docs drift.

## When to add a new runbook

The second time you give an agent the same procedural instruction in a PR review, write a runbook for it instead. Put it in `runbooks/`, add an entry to `runbooks/README.md`, and reference it from the relevant agent's "always read at startup" list.
