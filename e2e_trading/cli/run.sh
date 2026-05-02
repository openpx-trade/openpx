#!/usr/bin/env bash
# e2e_trading CLI driver.
#
# Exercises every variation of the authenticated trading subcommands against
# both exchanges through the openpx CLI binary. Validates exit codes, JSON
# shape, and unified error semantics across Kalshi (BTC 15m) and Polymarket
# (BTC 5m) live markets.
#
# Run from repo root:
#   bash e2e_trading/cli/run.sh
#
# Output: writes per-test results into ../results/cli.log, prints a summary,
# exits non-zero if any case fails.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CLI="$REPO_ROOT/target/release/openpx"
LOG_DIR="$REPO_ROOT/e2e_trading/results"
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

# Run a CLI invocation, capture stdout/stderr/exit-code, write to log.
# Args: <label> <expect_ok|expect_err|expect_ok_or_auth_skip> <cmd...>
#
# expect_ok_or_auth_skip: passes either on rc=0 with valid JSON, OR on a
# Polymarket auth-derive failure (counted as SKIP, not FAIL).
run_case() {
    local label="$1" expectation="$2"; shift 2
    note "$label"
    {
        printf '$ %q' "${1}"
        for a in "${@:2}"; do printf ' %q' "$a"; done
        printf '\n'
    } | tee -a "$LOG" >/dev/null

    local out rc
    out="$("$@" 2>&1)"; rc=$?
    if [[ $rc -ne 0 ]] && grep -qE "http error|timed out|rate limit|429|connection" <<<"$out"; then
        sleep 1
        out="$("$@" 2>&1)"; rc=$?
        printf '[retry on transient]\n' | tee -a "$LOG" >/dev/null
    fi

    {
        if [[ ${#out} -gt 1500 ]]; then
            printf '%s\n... [truncated, total %d bytes]\n' "${out:0:1500}" "${#out}"
        else
            printf '%s\n' "$out"
        fi
        printf 'exit=%d\n' "$rc"
    } | tee -a "$LOG" >/dev/null

    local outcome=fail
    case "$expectation" in
        expect_ok)
            if [[ $rc -eq 0 ]] && python3 -c "import json,sys; json.loads(sys.stdin.read())" <<<"$out" 2>>"$LOG"; then
                outcome=pass
            fi
            ;;
        expect_err)
            [[ $rc -ne 0 ]] && outcome=pass
            ;;
        expect_ok_or_auth_skip)
            if [[ $rc -eq 0 ]] && python3 -c "import json,sys; json.loads(sys.stdin.read())" <<<"$out" 2>>"$LOG"; then
                outcome=pass
            elif grep -qE "Could not derive api key|Cloudflare WAF blocked|Cannot reach clob.polymarket.com|L1 EIP-712 signature|no signing method|private key required" <<<"$out"; then
                outcome=skip
            fi
            ;;
    esac

    case "$outcome" in
        pass)
            PASS=$((PASS+1)); PASS_LIST+=("$label")
            printf '  ✓ PASS\n' | tee -a "$LOG"
            ;;
        skip)
            SKIP=$((SKIP+1)); SKIP_LIST+=("$label")
            printf '  ⊘ SKIP (auth unavailable)\n' | tee -a "$LOG"
            ;;
        *)
            FAIL=$((FAIL+1)); FAIL_LIST+=("$label")
            printf '  ✗ FAIL (expected %s, got rc=%d)\n' "$expectation" "$rc" | tee -a "$LOG"
            ;;
    esac
}

# Discover the Kalshi BTC 15m active market ticker.
seed_kalshi_btc_market() {
    "$CLI" kalshi fetch-markets --series-ticker KXBTC15M --status active --limit 5 2>/dev/null \
        | python3 -c "
import json, sys
data = json.load(sys.stdin)
markets = data.get('markets', [])
if not markets:
    sys.exit(1)
m = sorted(markets, key=lambda m: m.get('close_time') or '')[0]
print(m['ticker'])
"
}

# Discover the Polymarket BTC 5m active market — try the current 5m boundary
# first, then ±5min in case the active window has shifted.
seed_polymarket_btc_market_and_token() {
    local server_unix
    server_unix=$("$CLI" polymarket fetch-server-time 2>/dev/null \
        | python3 -c "import json,sys; print(json.load(sys.stdin)['unix_seconds'])")
    if [[ -z "$server_unix" ]]; then
        return 1
    fi
    local bucket=$(( (server_unix / 300) * 300 ))
    for offset in 0 300 -300; do
        local b=$(( bucket + offset ))
        local et="btc-updown-5m-${b}"
        local out
        out="$("$CLI" polymarket fetch-markets --event-ticker "$et" --status active --limit 5 2>/dev/null || true)"
        local result
        result=$(python3 -c "
import json, sys
data = json.loads(sys.stdin.read() or '{}')
markets = data.get('markets', [])
if not markets:
    sys.exit(1)
m = markets[0]
ticker = m.get('ticker', '')
outs = m.get('outcomes') or []
tok = (outs[0] or {}).get('token_id') if outs else ''
if not (ticker and tok):
    sys.exit(1)
print(f'{ticker}|{tok}')
" <<<"$out" 2>/dev/null) || continue
        if [[ -n "$result" ]]; then
            printf '%s\n' "$result"
            return 0
        fi
    done
    return 1
}

# ---------------------------------------------------------------------------
# Per-exchange test suite
# ---------------------------------------------------------------------------

# --- Kalshi -----------------------------------------------------------------
note "[kalshi] seeding BTC 15m market"
KALSHI_MARKET="$(seed_kalshi_btc_market || true)"
if [[ -z "$KALSHI_MARKET" ]]; then
    printf '  WARN: no active KXBTC15M market — skipping Kalshi suite\n' | tee -a "$LOG"
else
    printf 'kalshi market=%s\n' "$KALSHI_MARKET" | tee -a "$LOG"

    run_case "[kalshi] fetch-server-time"           expect_ok  "$CLI" kalshi fetch-server-time
    run_case "[kalshi] fetch-balance"               expect_ok  "$CLI" kalshi fetch-balance
    run_case "[kalshi] fetch-positions unfiltered"  expect_ok  "$CLI" kalshi fetch-positions
    run_case "[kalshi] fetch-positions filtered"    expect_ok  "$CLI" kalshi fetch-positions --market-ticker "$KALSHI_MARKET"
    run_case "[kalshi] fetch-open-orders unfilt."   expect_ok  "$CLI" kalshi fetch-open-orders
    run_case "[kalshi] fetch-open-orders filtered"  expect_ok  "$CLI" kalshi fetch-open-orders --asset-id "$KALSHI_MARKET"
    run_case "[kalshi] fetch-fills unfiltered"      expect_ok  "$CLI" kalshi fetch-fills --limit 10
    run_case "[kalshi] fetch-fills filtered"        expect_ok  "$CLI" kalshi fetch-fills --market-ticker "$KALSHI_MARKET" --limit 5
    run_case "[kalshi] fetch-trades basic"          expect_ok  "$CLI" kalshi fetch-trades "$KALSHI_MARKET" --limit 20
    run_case "[kalshi] fetch-trades with window"    expect_ok  "$CLI" kalshi fetch-trades "$KALSHI_MARKET" --start-ts 0 --limit 5
    run_case "[kalshi] fetch-order unknown"         expect_err "$CLI" kalshi fetch-order 00000000-0000-0000-0000-000000000000
    run_case "[kalshi] cancel-order unknown"        expect_err "$CLI" kalshi cancel-order 00000000-0000-0000-0000-000000000000

    # Place a resting BUY at $0.05 on Yes — well below the BTC 15m mid (~$0.50).
    # Capture the order_id, fetch it, then cancel it.
    note "[kalshi] order lifecycle: create → fetch → cancel"
    CREATE_OUT="$("$CLI" kalshi create-order "$KALSHI_MARKET" --outcome yes --side buy --price 0.05 --size 1 --order-type gtc 2>&1)"
    rc=$?
    printf '%s\nexit=%d\n' "$CREATE_OUT" "$rc" | tee -a "$LOG" >/dev/null
    if [[ $rc -eq 0 ]]; then
        ORDER_ID="$(python3 -c "import json,sys; print(json.loads(sys.stdin.read())['id'])" <<<"$CREATE_OUT" 2>/dev/null)"
        if [[ -n "$ORDER_ID" ]]; then
            PASS=$((PASS+1)); PASS_LIST+=("[kalshi] create-order valid")
            printf '  ✓ PASS create-order id=%s\n' "$ORDER_ID" | tee -a "$LOG"
            run_case "[kalshi] fetch-order placed"     expect_ok  "$CLI" kalshi fetch-order "$ORDER_ID"
            run_case "[kalshi] cancel-order placed"    expect_ok  "$CLI" kalshi cancel-order "$ORDER_ID"
        else
            FAIL=$((FAIL+1)); FAIL_LIST+=("[kalshi] create-order valid (no id)")
            printf '  ✗ FAIL create-order (no id parsed)\n' | tee -a "$LOG"
        fi
    else
        FAIL=$((FAIL+1)); FAIL_LIST+=("[kalshi] create-order valid")
        printf '  ✗ FAIL create-order rc=%d\n' "$rc" | tee -a "$LOG"
    fi

    run_case "[kalshi] create-order zero price"     expect_err "$CLI" kalshi create-order "$KALSHI_MARKET" --outcome yes --side buy --price 0.0 --size 1
    run_case "[kalshi] create-order one price"      expect_err "$CLI" kalshi create-order "$KALSHI_MARKET" --outcome yes --side buy --price 1.0 --size 1
    run_case "[kalshi] create-order negative size"  expect_err "$CLI" kalshi create-order "$KALSHI_MARKET" --outcome yes --side buy --price 0.05 --size -1
    run_case "[kalshi] cancel-all-orders unfilt."   expect_ok  "$CLI" kalshi cancel-all-orders
    run_case "[kalshi] cancel-all-orders filtered"  expect_ok  "$CLI" kalshi cancel-all-orders --asset-id "$KALSHI_MARKET"
fi

# --- Polymarket -------------------------------------------------------------
note "[polymarket] seeding BTC 5m market"
PM_DATA="$(seed_polymarket_btc_market_and_token || true)"
if [[ -z "$PM_DATA" ]]; then
    printf '  WARN: no active BTC 5m market — skipping Polymarket suite\n' | tee -a "$LOG"
else
    PM_MARKET="${PM_DATA%%|*}"
    PM_TOKEN="${PM_DATA##*|}"
    printf 'polymarket market=%s token=%s\n' "$PM_MARKET" "$PM_TOKEN" | tee -a "$LOG"

    run_case "[polymarket] fetch-server-time"               expect_ok               "$CLI" polymarket fetch-server-time
    run_case "[polymarket] fetch-balance"                   expect_ok_or_auth_skip  "$CLI" polymarket fetch-balance
    run_case "[polymarket] refresh-balance"                 expect_ok_or_auth_skip  "$CLI" polymarket refresh-balance
    run_case "[polymarket] fetch-positions unfiltered"      expect_ok               "$CLI" polymarket fetch-positions
    run_case "[polymarket] fetch-positions filtered"        expect_ok               "$CLI" polymarket fetch-positions --market-ticker "$PM_MARKET"
    run_case "[polymarket] fetch-open-orders unfilt."       expect_ok_or_auth_skip  "$CLI" polymarket fetch-open-orders
    run_case "[polymarket] fetch-open-orders filtered"      expect_ok_or_auth_skip  "$CLI" polymarket fetch-open-orders --asset-id "$PM_TOKEN"
    run_case "[polymarket] fetch-fills unfiltered"          expect_ok               "$CLI" polymarket fetch-fills --limit 10
    run_case "[polymarket] fetch-fills filtered"            expect_ok               "$CLI" polymarket fetch-fills --market-ticker "$PM_MARKET" --limit 5
    run_case "[polymarket] fetch-trades basic"              expect_ok               "$CLI" polymarket fetch-trades "$PM_MARKET" --limit 20
    run_case "[polymarket] fetch-order unknown"             expect_err              "$CLI" polymarket fetch-order 0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddead
    run_case "[polymarket] cancel-order unknown"            expect_err              "$CLI" polymarket cancel-order 0xdeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddeaddead

    note "[polymarket] order lifecycle: create → fetch → cancel"
    CREATE_OUT="$("$CLI" polymarket create-order "$PM_TOKEN" --outcome yes --side buy --price 0.05 --size 110 --order-type gtc 2>&1)"
    rc=$?
    printf '%s\nexit=%d\n' "$CREATE_OUT" "$rc" | tee -a "$LOG" >/dev/null
    if [[ $rc -eq 0 ]]; then
        ORDER_ID="$(python3 -c "import json,sys; print(json.loads(sys.stdin.read())['id'])" <<<"$CREATE_OUT" 2>/dev/null)"
        if [[ -n "$ORDER_ID" ]]; then
            PASS=$((PASS+1)); PASS_LIST+=("[polymarket] create-order valid")
            printf '  ✓ PASS create-order id=%s\n' "$ORDER_ID" | tee -a "$LOG"
            run_case "[polymarket] fetch-order placed"     expect_ok               "$CLI" polymarket fetch-order "$ORDER_ID"
            run_case "[polymarket] cancel-order placed"    expect_ok               "$CLI" polymarket cancel-order "$ORDER_ID"
        else
            FAIL=$((FAIL+1)); FAIL_LIST+=("[polymarket] create-order valid (no id)")
            printf '  ✗ FAIL create-order (no id parsed)\n' | tee -a "$LOG"
        fi
    elif grep -qE "Could not derive api key|Cloudflare WAF blocked|Cannot reach clob.polymarket.com|L1 EIP-712 signature|no signing method|private key required" <<<"$CREATE_OUT"; then
        SKIP=$((SKIP+1)); SKIP_LIST+=("[polymarket] create-order valid")
        printf '  ⊘ SKIP create-order (auth unavailable)\n' | tee -a "$LOG"
    else
        FAIL=$((FAIL+1)); FAIL_LIST+=("[polymarket] create-order valid")
        printf '  ✗ FAIL create-order rc=%d\n' "$rc" | tee -a "$LOG"
    fi

    run_case "[polymarket] create-order zero price"     expect_err              "$CLI" polymarket create-order "$PM_TOKEN" --outcome yes --side buy --price 0.0 --size 110
    run_case "[polymarket] create-order one price"      expect_err              "$CLI" polymarket create-order "$PM_TOKEN" --outcome yes --side buy --price 1.0 --size 110
    run_case "[polymarket] create-order negative size"  expect_err              "$CLI" polymarket create-order "$PM_TOKEN" --outcome yes --side buy --price 0.05 --size -10
    run_case "[polymarket] cancel-all-orders unfilt."   expect_ok_or_auth_skip  "$CLI" polymarket cancel-all-orders
    run_case "[polymarket] cancel-all-orders filtered"  expect_ok_or_auth_skip  "$CLI" polymarket cancel-all-orders --asset-id "$PM_TOKEN"
fi

note "summary"
printf 'PASS %d\nSKIP %d\nFAIL %d\n' "$PASS" "$SKIP" "$FAIL" | tee -a "$LOG"
if (( FAIL > 0 )); then
    printf '\nFailing cases:\n' | tee -a "$LOG"
    printf '  - %s\n' "${FAIL_LIST[@]}" | tee -a "$LOG"
    exit 1
fi
exit 0
