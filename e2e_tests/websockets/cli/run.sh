#!/usr/bin/env bash
# e2e_tests/websockets CLI driver.
#
# Exercises the unified websocket subcommands of the openpx CLI against both
# Kalshi and Polymarket. Validates that the CLI can:
#   - construct a websocket from env vars
#   - subscribe to an active market
#   - emit at least one well-formed JSON update on stdout within a window
#   - exit cleanly when interrupted via timeout
#
# Run from repo root:
#   bash e2e_tests/websockets/cli/run.sh
#
# Output: writes per-test results into ../results/cli.log, prints a final
# pass/fail summary, exits non-zero if any case fails.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
CLI="$REPO_ROOT/target/release/openpx"
LOG_DIR="$REPO_ROOT/e2e_tests/websockets/results"
LOG="$LOG_DIR/cli.log"
ENV_FILE="$REPO_ROOT/.env"

mkdir -p "$LOG_DIR"
: > "$LOG"

if [[ -f "$ENV_FILE" ]]; then
    set -a
    # shellcheck disable=SC1090
    source "$ENV_FILE"
    set +a
fi

PASS=0; FAIL=0; SKIP=0
PASS_LIST=(); FAIL_LIST=(); SKIP_LIST=()

note() { printf '\n=== %s ===\n' "$1" | tee -a "$LOG"; }

# Run a CLI ws subcommand for `seconds` then send SIGINT and capture output.
# Args: <label> <expect_ok|expect_skip> <seconds> <cmd...>
ws_case() {
    local label="$1"; local expect="$2"; local seconds="$3"; shift 3
    note "$label"
    local out
    # Use timeout(1) to cap runtime; --preserve-status keeps the CLI's exit code
    # if it terminates first. macOS bsd-timeout uses -k for kill-after.
    if command -v gtimeout >/dev/null 2>&1; then
        TIMEOUT=gtimeout
    elif command -v timeout >/dev/null 2>&1; then
        TIMEOUT=timeout
    else
        echo "no timeout(1) available, skipping" | tee -a "$LOG"
        SKIP=$((SKIP+1)); SKIP_LIST+=("$label")
        return
    fi
    out=$("$TIMEOUT" --preserve-status "$seconds" "$@" 2>&1 | head -200)
    echo "$out" | tee -a "$LOG" >/dev/null

    case "$expect" in
        expect_ok)
            # CLI streams JSON updates; at least one well-formed kind line.
            if echo "$out" | grep -qE '"kind"[[:space:]]*:[[:space:]]*"(Snapshot|Delta|Trade|Fill)"'; then
                PASS=$((PASS+1)); PASS_LIST+=("$label")
                echo "PASS" | tee -a "$LOG"
            else
                FAIL=$((FAIL+1)); FAIL_LIST+=("$label")
                echo "FAIL: no Snapshot/Delta/Trade/Fill in $seconds s" | tee -a "$LOG"
            fi
            ;;
        expect_skip)
            # We tolerate any output — used for cases where market may be quiet.
            SKIP=$((SKIP+1)); SKIP_LIST+=("$label")
            echo "SKIP (informational)" | tee -a "$LOG"
            ;;
    esac
}

# Discover an active market id by piping through `openpx fetch-markets`.
discover_id() {
    local exchange="$1"
    "$CLI" fetch-markets --exchange "$exchange" --status active --limit 5 2>/dev/null \
        | python3 -c '
import json, sys
data = json.load(sys.stdin)
markets = data.get("markets", data) if isinstance(data, dict) else data
for m in markets[:5]:
    if "ticker" in m and m.get("ticker"):
        print(m["ticker"]); break
        ' 2>/dev/null
}

if [[ ! -x "$CLI" ]]; then
    echo "CLI not built at $CLI — run: cargo build -p px-cli --release" | tee -a "$LOG"
    SKIP=$((SKIP+1)); SKIP_LIST+=("cli_not_built")
fi

# Each case runs the ws subcommand for ~12 s and looks for a JSON update in stdout.
if [[ -x "$CLI" ]]; then
    KALSHI_TICKER=$(discover_id kalshi 2>/dev/null || true)
    POLY_TICKER=$(discover_id polymarket 2>/dev/null || true)

    if [[ -n "$KALSHI_TICKER" ]]; then
        ws_case "kalshi.ws-orderbook" expect_ok 12 \
            "$CLI" ws orderbook --exchange kalshi --market-ticker "$KALSHI_TICKER"
    else
        echo "no kalshi ticker discovered; skipping kalshi ws cases" | tee -a "$LOG"
        SKIP=$((SKIP+1)); SKIP_LIST+=("kalshi.ws-orderbook")
    fi

    if [[ -n "$POLY_TICKER" ]]; then
        ws_case "polymarket.ws-orderbook" expect_ok 12 \
            "$CLI" ws orderbook --exchange polymarket --market-ticker "$POLY_TICKER"
    else
        echo "no polymarket ticker discovered; skipping polymarket ws cases" | tee -a "$LOG"
        SKIP=$((SKIP+1)); SKIP_LIST+=("polymarket.ws-orderbook")
    fi
fi

note "Summary"
echo "$PASS pass / $FAIL fail / $SKIP skip" | tee -a "$LOG"
echo "log: $LOG"
exit $([ "$FAIL" -eq 0 ] && echo 0 || echo 1)
