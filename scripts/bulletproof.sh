#!/usr/bin/env bash
# ZAP1 bulletproof API test suite.
# Tests every API endpoint and conformance suite.
#
# Usage:
#   API_KEY=xxx bash scripts/bulletproof.sh
#   API_KEY=xxx API_URL=http://127.0.0.1:3080 bash scripts/bulletproof.sh
#
# Exit: 0 if all pass, 1 if any fail.

set -uo pipefail

API_URL="${API_URL:-http://127.0.0.1:3080}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"

RED='\033[0;31m'
GRN='\033[0;32m'
YLW='\033[1;33m'
RST='\033[0m'

pass=0
fail=0

# Require API_KEY
if [ -z "${API_KEY:-}" ]; then
  printf "${RED}ERROR${RST}  API_KEY is not set. Usage: API_KEY=xxx bash scripts/bulletproof.sh\n"
  exit 1
fi

ok() {
  printf "${GRN}PASS${RST}  %s\n" "$1"
  pass=$((pass + 1))
}

fail_test() {
  printf "${RED}FAIL${RST}  %s  (%s)\n" "$1" "$2"
  fail=$((fail + 1))
}

skip_test() {
  printf "${YLW}SKIP${RST}  %s  (%s)\n" "$1" "$2"
}

# http_get <url> [extra curl args...]
# Returns HTTP status code, body in $BODY
http_get() {
  local url="$1"; shift
  BODY=$(curl -s -w "\n%{http_code}" "$@" "$url")
  HTTP_CODE=$(printf '%s' "$BODY" | tail -1)
  BODY=$(printf '%s' "$BODY" | head -n -1)
}

# http_post <url> <content_type> <data> [extra curl args...]
http_post() {
  local url="$1" ctype="$2" data="$3"; shift 3
  BODY=$(curl -s -w "\n%{http_code}" -X POST -H "Content-Type: $ctype" --data "$data" "$@" "$url")
  HTTP_CODE=$(printf '%s' "$BODY" | tail -1)
  BODY=$(printf '%s' "$BODY" | head -n -1)
}

json_field() {
  python3 -c "import sys,json; print(json.loads(sys.stdin.read()).get('$1',''))" 2>/dev/null
}

printf "\nZAP1 bulletproof test suite\n"
printf "API: %s\n\n" "$API_URL"

# Section 1 - read endpoints return 200
printf "-- read endpoints --\n"

for path in "/health" "/stats" "/anchor/status" "/protocol/info" "/build/info" "/cohort" "/anchor/history" "/events"; do
  http_get "${API_URL}${path}"
  if [ "$HTTP_CODE" = "200" ]; then
    ok "GET ${path}"
  else
    fail_test "GET ${path}" "HTTP $HTTP_CODE"
  fi
done

# Section 2 - /attest auth behavior
printf "\n-- /attest authentication --\n"

ATTEST_PAYLOAD='{"event_type":"HOSTING_PAYMENT","wallet_hash":"test-bulletproof-wallet","serial_number":"bp-test-001","month":4,"year":2026}'

# 2a. valid key returns 201 or 200 with leaf_hash
http_post "${API_URL}/attest" "application/json" "$ATTEST_PAYLOAD" \
  -H "Authorization: Bearer ${API_KEY}"
if [ "$HTTP_CODE" = "201" ] || [ "$HTTP_CODE" = "200" ]; then
  ATTEST_LEAF=$(printf '%s' "$BODY" | json_field "leaf_hash")
  if [ -n "$ATTEST_LEAF" ]; then
    ok "/attest valid key returns ${HTTP_CODE} with leaf_hash"
  else
    fail_test "/attest valid key" "HTTP ${HTTP_CODE} but no leaf_hash in response"
  fi
else
  ATTEST_LEAF=""
  fail_test "/attest valid key" "HTTP ${HTTP_CODE}"
fi

# 2b. no key returns 401
http_post "${API_URL}/attest" "application/json" "$ATTEST_PAYLOAD"
if [ "$HTTP_CODE" = "401" ]; then
  ok "/attest no key returns 401"
else
  fail_test "/attest no key returns 401" "HTTP ${HTTP_CODE}"
fi

# 2c. bad key returns 401
http_post "${API_URL}/attest" "application/json" "$ATTEST_PAYLOAD" \
  -H "Authorization: Bearer INVALID-KEY-xyz-00000"
if [ "$HTTP_CODE" = "401" ]; then
  ok "/attest bad key returns 401"
else
  fail_test "/attest bad key returns 401" "HTTP ${HTTP_CODE}"
fi

# Section 3 - /admin/keys
printf "\n-- /admin/keys --\n"

# 3a. superadmin returns 200
http_get "${API_URL}/admin/keys" -H "Authorization: Bearer ${API_KEY}"
if [ "$HTTP_CODE" = "200" ]; then
  ok "GET /admin/keys with superadmin returns 200"
else
  fail_test "GET /admin/keys with superadmin returns 200" "HTTP ${HTTP_CODE}"
fi

# 3b. no auth returns 401
http_get "${API_URL}/admin/keys"
if [ "$HTTP_CODE" = "401" ]; then
  ok "GET /admin/keys without auth returns 401"
else
  fail_test "GET /admin/keys without auth returns 401" "HTTP ${HTTP_CODE}"
fi

# Section 4 - verify endpoints (use leaf created in section 2)
printf "\n-- /verify/{leaf} --\n"

if [ -n "$ATTEST_LEAF" ]; then
  # 4a. /verify/{leaf}/check returns valid=true
  http_get "${API_URL}/verify/${ATTEST_LEAF}/check"
  if [ "$HTTP_CODE" = "200" ]; then
    VALID_FIELD=$(printf '%s' "$BODY" | json_field "valid")
    if [ "$VALID_FIELD" = "True" ] || [ "$VALID_FIELD" = "true" ]; then
      ok "/verify/${ATTEST_LEAF}/check returns valid=true"
    else
      fail_test "/verify/${ATTEST_LEAF}/check" "valid=${VALID_FIELD} (expected true)"
    fi
  else
    fail_test "/verify/${ATTEST_LEAF}/check" "HTTP ${HTTP_CODE}"
  fi

  # 4b. /verify/{leaf}/proof.json returns a valid proof bundle
  http_get "${API_URL}/verify/${ATTEST_LEAF}/proof.json"
  if [ "$HTTP_CODE" = "200" ]; then
    PROOF_LEAF=$(printf '%s' "$BODY" | json_field "leaf_hash")
    PROOF_ROOT=$(printf '%s' "$BODY" | json_field "root")
    if [ -n "$PROOF_LEAF" ] && [ -n "$PROOF_ROOT" ]; then
      ok "/verify/${ATTEST_LEAF}/proof.json returns valid bundle"
    else
      fail_test "/verify/${ATTEST_LEAF}/proof.json" "missing leaf_hash or root in bundle"
    fi
  else
    fail_test "/verify/${ATTEST_LEAF}/proof.json" "HTTP ${HTTP_CODE}"
  fi
else
  skip_test "/verify/{leaf}/check" "no leaf_hash from /attest (skipping)"
  skip_test "/verify/{leaf}/proof.json" "no leaf_hash from /attest (skipping)"
fi

# Section 5 - /webhook/cipherpay
printf "\n-- /webhook/cipherpay --\n"

CIPHER_PAYLOAD='{"txid":"bp-test-txid-001","amount_zat":62500000,"merchant_id":"bp-merchant-001","timestamp":1743465600,"status":"completed"}'
http_post "${API_URL}/webhook/cipherpay" "application/json" "$CIPHER_PAYLOAD"
if [ "$HTTP_CODE" = "200" ] || [ "$HTTP_CODE" = "201" ]; then
  ok "/webhook/cipherpay accepts test payload (HTTP ${HTTP_CODE})"
else
  fail_test "/webhook/cipherpay" "HTTP ${HTTP_CODE}"
fi

# Section 6 - /agent/{id}/bond and /agent/{id}/policy/verify
printf "\n-- agent endpoints --\n"

TEST_AGENT_ID="bp-test-agent-001"

http_get "${API_URL}/agent/${TEST_AGENT_ID}/bond"
if [ "$HTTP_CODE" = "200" ]; then
  BOND_AGENT=$(printf '%s' "$BODY" | json_field "agent_id")
  if [ -n "$BOND_AGENT" ]; then
    ok "/agent/${TEST_AGENT_ID}/bond returns JSON"
  else
    fail_test "/agent/${TEST_AGENT_ID}/bond" "HTTP 200 but no agent_id in response"
  fi
else
  fail_test "/agent/${TEST_AGENT_ID}/bond" "HTTP ${HTTP_CODE}"
fi

http_get "${API_URL}/agent/${TEST_AGENT_ID}/policy/verify"
if [ "$HTTP_CODE" = "200" ]; then
  POLICY_AGENT=$(printf '%s' "$BODY" | json_field "agent_id")
  if [ -n "$POLICY_AGENT" ]; then
    ok "/agent/${TEST_AGENT_ID}/policy/verify returns JSON"
  else
    fail_test "/agent/${TEST_AGENT_ID}/policy/verify" "HTTP 200 but no agent_id in response"
  fi
else
  fail_test "/agent/${TEST_AGENT_ID}/policy/verify" "HTTP ${HTTP_CODE}"
fi

# Section 7 - conformance suite
printf "\n-- ZIP 1243 conformance suite --\n"

CONFORMANCE_SCRIPT="${REPO_ROOT}/conformance/zip1243_conformance.py"
if [ ! -f "$CONFORMANCE_SCRIPT" ]; then
  fail_test "conformance/zip1243_conformance.py" "script not found at ${CONFORMANCE_SCRIPT}"
elif ! command -v python3 > /dev/null 2>&1; then
  fail_test "conformance/zip1243_conformance.py" "python3 not found"
else
  CONFORM_OUT=$(python3 "$CONFORMANCE_SCRIPT" 2>&1)
  CONFORM_EXIT=$?
  if [ "$CONFORM_EXIT" = "0" ]; then
    CONFORM_SUMMARY=$(printf '%s' "$CONFORM_OUT" | tail -1)
    ok "zip1243_conformance.py passes (${CONFORM_SUMMARY})"
  else
    CONFORM_SUMMARY=$(printf '%s' "$CONFORM_OUT" | tail -3 | tr '\n' ' ')
    fail_test "zip1243_conformance.py" "$CONFORM_SUMMARY"
  fi
fi

# Summary
printf "\n=============================\n"
total=$((pass + fail))
printf "%d/%d passed, %d failed\n" "$pass" "$total" "$fail"

if [ "$fail" -gt 0 ]; then
  exit 1
fi
