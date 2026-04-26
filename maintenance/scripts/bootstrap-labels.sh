#!/usr/bin/env bash
# Create the GitHub label taxonomy used by the autonomous-maintenance agents.
# Run once after repo setup. Idempotent — re-running updates colors/descriptions.
#
# Usage:
#     bash maintenance/scripts/bootstrap-labels.sh
#     # or via just:
#     just labels

set -euo pipefail

# Format: name|color|description
LABELS=(
  # --- Maintenance type (exactly one per agent-opened PR) ---
  "autonomous-doc-sync|0E8A16|PR tracks an upstream spec or changelog change"
  "parity-fill|1D76DB|PR closes a cross-exchange parity gap"
  "regen|C5DEF5|PR contains only generated artifacts (schema, models, docs)"
  "breaking-change|D93F0B|PR changes public API surface — semver major"
  "requires-human-careful-review|B60205|On-chain or auth — must be reviewed line-by-line by human"
  "docs-only|0075CA|Markdown / MDX changes only — no Rust source touched"

  # --- Area (zero or more per PR) ---
  "area:core|EDEDED|engine/core/ — trait, manifest schema, models, error hierarchy"
  "area:sdk|EDEDED|engine/sdk/ — ExchangeInner enum + dispatch macros"
  "area:kalshi|EDEDED|engine/exchanges/kalshi/"
  "area:polymarket|EDEDED|engine/exchanges/polymarket/"
  "area:bindings|EDEDED|sdks/python/ or sdks/typescript/"
  "area:onchain|FBCA04|Polymarket on-chain code (clob/ctf/relayer/swap/signer/approvals)"
  "area:docs|EDEDED|docs/ Mintlify site"
  "area:ci|EDEDED|.github/workflows/ or CI tooling"

  # --- Control / kill switches ---
  "pause-bots|D93F0B|KILL SWITCH — every scheduled agent workflow checks for this label and skips"
  "triage-ready|FBCA04|Admin marked a public-user issue as ready for orchestrator triage"
  "parity-fill-approved|0E8A16|Human approved a parity-analyst proposal — core-architect picks it up"

  # --- Issue types (some overlap with GitHub defaults; -f updates them) ---
  "enhancement|A2EEEF|New feature or improvement request"
  "parity-gap|FBCA04|Method or feature one exchange has but the other doesn't"
  "good-first-implementation|7057FF|Suitable for a maintainer to implement against existing patterns"
  "incident-revert|D93F0B|Reverts an auto-merged change that broke downstream"
  "bench-regression|FBCA04|cargo bench shows a >5% regression on a tracked metric"
  "live-test-failure|D93F0B|Daily live-tests workflow surfaced a regression"
)

REPO="${OPENPX_REPO:-openpx-trade/openpx}"

echo "Creating labels in $REPO ..."
for line in "${LABELS[@]}"; do
  IFS='|' read -r name color description <<< "$line"
  printf "  %-32s ... " "$name"
  if gh label create "$name" --color "$color" --description "$description" --repo "$REPO" --force >/dev/null 2>&1; then
    echo "ok"
  else
    echo "FAILED"
  fi
done

echo
echo "Done. Verify: gh label list --repo $REPO"
