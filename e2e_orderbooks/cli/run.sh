#!/usr/bin/env bash
# e2e_orderbooks CLI driver.
#
# Exercises every variation of the orderbook subcommands against both
# exchanges through the openpx CLI. Validates the JSON-on-stdout contract
# (every command emits a single well-formed JSON document, error class is
# stable, exit codes signal success/failure correctly).
#
# Run from repo root:
#   bash e2e_orderbooks/cli/run.sh
#
# Output: writes per-test results into ../results/cli.log, prints a final
# pass/fail summary, exits non-zero if any case fails.

set -uo pipefail

# Resolve repo root — script can be invoked from anywhere.
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CLI="$REPO_ROOT/target/release/openpx"
LOG_DIR="$REPO_ROOT/e2e_orderbooks/results"
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

PASS=0; FAIL=0
PASS_LIST=(); FAIL_LIST=()

note() { printf '\n=== %s ===\n' "$1" | tee -a "$LOG"; }

# Run a CLI invocation, capture stdout/stderr/exit-code, write to log.
# Args: <label> <expect_ok|expect_err|expect_err_or_empty_book> <cmd...>
#
# expect_err_or_empty_book: accepts either non-zero exit, or a zero exit with
# an Orderbook JSON whose bids and asks are both empty. Kalshi returns the
# latter for nonexistent tickers (no 404), Polymarket the former — both are
# acceptable per the unified contract.
run_case() {
    local label="$1" expectation="$2"; shift 2
    note "$label"
    {
        printf '$ %q' "${1}"
        for a in "${@:2}"; do printf ' %q' "$a"; done
        printf '\n'
    } | tee -a "$LOG" >/dev/null

    local out
    local rc
    out="$("$@" 2>&1)"; rc=$?
    # Single retry on transient network errors — same flakiness window the
    # Rust suite gracefully skips. Don't retry hard errors or successes.
    if [[ $rc -ne 0 ]] && grep -qE "http error|timed out|rate limit|429|connection" <<<"$out"; then
        sleep 1
        out="$("$@" 2>&1)"; rc=$?
        printf '[retry on transient]\n' | tee -a "$LOG" >/dev/null
    fi

    # Truncate huge bodies in the log (orderbooks can be large)
    {
        if [[ ${#out} -gt 1000 ]]; then
            printf '%s\n... [truncated, total %d bytes]\n' "${out:0:1000}" "${#out}"
        else
            printf '%s\n' "$out"
        fi
        printf 'exit=%d\n' "$rc"
    } | tee -a "$LOG" >/dev/null

    local ok=0
    case "$expectation" in
        expect_ok)
            if [[ $rc -eq 0 ]] && python3 -c "import json,sys; json.loads(sys.stdin.read())" <<<"$out" 2>>"$LOG"; then
                ok=1
            fi
            ;;
        expect_err)
            [[ $rc -ne 0 ]] && ok=1
            ;;
        expect_err_or_empty_book)
            if [[ $rc -ne 0 ]]; then
                ok=1
            elif python3 -c "
import json, sys
b = json.loads(sys.stdin.read())
sys.exit(0 if (not b.get('bids') and not b.get('asks')) else 1)
" <<<"$out" 2>>"$LOG"; then
                ok=1
            fi
            ;;
    esac
    if [[ $ok -eq 1 ]]; then
        PASS=$((PASS+1)); PASS_LIST+=("$label")
        printf '  ✓ PASS\n' | tee -a "$LOG"
    else
        FAIL=$((FAIL+1)); FAIL_LIST+=("$label")
        printf '  ✗ FAIL (expected %s, got rc=%d)\n' "$expectation" "$rc" | tee -a "$LOG"
    fi
}

# Pull a real asset_id from `fetch-markets` output for a given exchange.
# Kalshi: market ticker. Polymarket: first outcome's token_id.
seed_asset_id() {
    local ex="$1"
    "$CLI" "$ex" fetch-markets --status active --limit 30 2>/dev/null \
        | python3 -c "
import json, sys
data = json.load(sys.stdin)
for m in data.get('markets', []):
    outs = m.get('outcomes') or []
    if outs:
        tok = outs[0].get('token_id')
        if tok:
            print(tok); sys.exit(0)
    if m.get('ticker'):
        print(m['ticker']); sys.exit(0)
sys.exit(1)
"
}

# Pull up to N asset_ids from fetch-markets.
seed_asset_ids() {
    local ex="$1" n="$2"
    "$CLI" "$ex" fetch-markets --status active --limit 30 2>/dev/null \
        | python3 -c "
import json, sys
n = int('$n')
data = json.load(sys.stdin)
out = []
for m in data.get('markets', []):
    if len(out) >= n: break
    outs = m.get('outcomes') or []
    if outs and outs[0].get('token_id'):
        out.append(outs[0]['token_id'])
    elif m.get('ticker'):
        out.append(m['ticker'])
print(','.join(out))
"
}

# Pull a non-empty book asset_id (some markets have empty books). Returns
# the first one we successfully fetch with at least one bid or ask.
seed_book_asset_id() {
    local ex="$1"
    "$CLI" "$ex" fetch-markets --status active --limit 30 2>/dev/null \
        | python3 -c "
import json, sys
data = json.load(sys.stdin)
for m in data.get('markets', []):
    outs = m.get('outcomes') or []
    if outs and outs[0].get('token_id'):
        print(outs[0]['token_id'])
    elif m.get('ticker'):
        print(m['ticker'])
" \
        | while IFS= read -r aid; do
            book="$("$CLI" "$ex" fetch-orderbook "$aid" 2>/dev/null || true)"
            if [[ -n "$book" ]] && python3 -c "
import json,sys
b = json.loads(sys.stdin.read())
sys.exit(0 if (b.get('bids') or b.get('asks')) else 1)
" <<<"$book" 2>/dev/null; then
                printf '%s\n' "$aid"
                return 0
            fi
        done
    return 1
}

# ---------------------------------------------------------------------------
# Per-exchange test suite
# ---------------------------------------------------------------------------

for EX in kalshi polymarket; do
    note "[$EX] seeding asset_ids"
    AID="$(seed_book_asset_id "$EX" || true)"
    if [[ -z "$AID" ]]; then
        printf '  WARN: no non-empty book found for %s — skipping CLI suite\n' "$EX" | tee -a "$LOG"
        continue
    fi
    AID2="$(seed_asset_ids "$EX" 3)"
    printf 'asset_id=%s\nasset_ids=%s\n' "$AID" "$AID2" | tee -a "$LOG"

    run_case "[$EX] fetch-orderbook valid"          expect_ok                  "$CLI" "$EX" fetch-orderbook "$AID"
    # Kalshi: 200 + empty book; Polymarket: non-zero (404). Both pass the
    # unified contract — caller never sees a populated book for a fake id.
    run_case "[$EX] fetch-orderbook nonexistent"    expect_err_or_empty_book   "$CLI" "$EX" fetch-orderbook "OPENPX-CLI-NOPE-0"
    run_case "[$EX] fetch-orderbook malformed"      expect_err_or_empty_book   "$CLI" "$EX" fetch-orderbook '!@#$'

    # Batch — empty list is rejected by clap (num_args=1..) so we test only
    # the populated path. The empty-list contract is exercised by the Rust
    # suite which calls the SDK directly.
    run_case "[$EX] fetch-orderbooks-batch multi"   expect_ok  "$CLI" "$EX" fetch-orderbooks-batch --asset-ids "$AID2"

    if [[ "$EX" == "kalshi" ]]; then
        # Build a 101-element fake list so we hit the InvalidOrder cap.
        OVER="$(python3 -c "print(','.join(f'OPENPX-CAP-{i}' for i in range(101)))")"
        run_case "[$EX] fetch-orderbooks-batch above-cap" expect_err \
            "$CLI" "$EX" fetch-orderbooks-batch --asset-ids "$OVER"
    fi

    run_case "[$EX] fetch-orderbook-stats valid"           expect_ok  "$CLI" "$EX" fetch-orderbook-stats "$AID"
    run_case "[$EX] fetch-orderbook-impact small"          expect_ok  "$CLI" "$EX" fetch-orderbook-impact "$AID" 1.0
    run_case "[$EX] fetch-orderbook-impact large"          expect_ok  "$CLI" "$EX" fetch-orderbook-impact "$AID" 10000000.0
    run_case "[$EX] fetch-orderbook-impact zero"           expect_err "$CLI" "$EX" fetch-orderbook-impact "$AID" 0.0
    run_case "[$EX] fetch-orderbook-impact negative"       expect_err "$CLI" "$EX" fetch-orderbook-impact "$AID" -1.0
    run_case "[$EX] fetch-orderbook-microstructure valid"  expect_ok  "$CLI" "$EX" fetch-orderbook-microstructure "$AID"
done

note "summary"
printf 'PASS %d\nFAIL %d\n' "$PASS" "$FAIL" | tee -a "$LOG"
if (( FAIL > 0 )); then
    printf '\nFailing cases:\n' | tee -a "$LOG"
    printf '  - %s\n' "${FAIL_LIST[@]}" | tee -a "$LOG"
    exit 1
fi
exit 0
