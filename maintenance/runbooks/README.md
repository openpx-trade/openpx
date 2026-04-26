# Runbooks

Imperative checklists agents read at startup. Procedure-as-code lives here so prompts stay short and procedures are versioned.

| Runbook | When to read |
|---|---|
| `spec-version-bump.md` | An exchange-maintainer runs after `check_docs_drift.py` flagged an upstream spec/changelog/page change. |
| `contract-redeployment.md` | Polymarket-maintainer runs after `/resources/contracts.md` content drifted, or `contracts_test.rs` failed. |
| `parity-gap-closure.md` | A maintainer runs after a human-approved parity-analyst proposal routes a method implementation to them. |
| `issue-triage.md` | Orchestrator runs on `issues.*` events after the admin-association gate passes. |

When you are about to give an agent the same procedural instruction in a PR review for the second time, write a new runbook for it instead.
