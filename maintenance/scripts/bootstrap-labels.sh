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
  # --- PR type ---
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

  # --- Issue / PR misc (some overlap with GitHub defaults; -f updates them) ---
  "enhancement|A2EEEF|New feature or improvement request"
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
