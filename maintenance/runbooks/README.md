# Runbooks

Imperative checklists agents read at startup. Procedure-as-code lives here so prompts stay short and procedures are versioned.

| Runbook | When to read |
|---|---|
| `changelog-driven-update.md` | An exchange-maintainer runs when the orchestrator dispatches a `critical-exchange-specific` changelog entry to them. |
| `contract-redeployment.md` | Polymarket-maintainer runs when a changelog entry mentions a contract redeployment (CTF, NegRisk, etc.). |
| `parity-gap-closure.md` | A maintainer runs when the orchestrator's daily `describe()`-scan dispatches a `(exchange, method)` pair with `has_<method>: false` to them — they either implement the method or mark it intentionally unsupported. |
| `trait-evolution.md` | core-architect runs to extend the unified trait/manifest/models in response to an `overlap-opportunity` changelog dispatch from the orchestrator. |
| `pr-preflight.md` | Every PR-opening agent runs **before** `gh pr create` — sync regen + SDK builds + smoke imports + docs check. The CI side (`SDK Sync Check`, `Python SDK Build`, `Node.js SDK Build`) backstops it. |
| `pr-ci-watch.md` | Every PR-opening agent runs after `gh pr create` to watch CI and fix failures until green. |

When you are about to give an agent the same procedural instruction in a PR review for the second time, write a new runbook for it instead.
