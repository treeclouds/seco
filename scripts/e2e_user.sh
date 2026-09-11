#!/usr/bin/env bash
# E2E test for the SECO admin user-management API.
#
# Exercises the full CRUD surface: list / create / get / update / delete,
# plus authorization (non-superuser rejected).
#
# Prerequisites:
#   - jq must be installed.
#   - A superuser must exist. Bootstrap one (once) against the DB:
#       sudo -u postgres psql seco \
#         -c "UPDATE users SET is_superuser = true WHERE email = '<admin-email>';"
#     (register the account first via POST /api/auth/register if needed.)
#
# Usage:
#   BASE_URL=https://gudangsports.treeclouds.com \
#   ADMIN_EMAIL=admin@example.com \
#   ADMIN_PASSWORD='...' \
#   bash scripts/e2e_user.sh

set -euo pipefail

BASE_URL="${BASE_URL:-http://localhost:80}"
ADMIN_EMAIL="${ADMIN_EMAIL:?ADMIN_EMAIL must be set}"
ADMIN_PASSWORD="${ADMIN_PASSWORD:?ADMIN_PASSWORD must be set}"

command -v jq >/dev/null 2>&1 || { echo "ERROR: jq is required"; exit 1; }

pass=0
fail=0
step() { echo ""; echo "==> $*"; }
ok()   { echo "    PASS: $*"; pass=$((pass + 1)); }
ko()   { echo "    FAIL: $*"; fail=$((fail + 1)); }

# 1. Login as admin (superuser) to obtain a JWT
step "Login as admin ($ADMIN_EMAIL)"
login_resp=$(curl -sS -m 15 -X POST "$BASE_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d "{\"email\":\"$ADMIN_EMAIL\",\"password\":\"$ADMIN_PASSWORD\"}")
TOKEN=$(echo "$login_resp" | jq -r '.token // empty')
if [ -n "$TOKEN" ] && [ "$TOKEN" != "null" ]; then
  ok "token acquired"
else
  ko "login failed: $login_resp"
  echo "RESULT: $pass passed, $fail failed"
  exit 1
fi

AUTH=(-H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json")
TEST_EMAIL="e2e_$(date +%s)@test.com"

# 2. Create a user
step "Create user ($TEST_EMAIL)"
create_resp=$(curl -sS -m 15 -X POST "$BASE_URL/api/user/new" "${AUTH[@]}" \
  -d "{\"email\":\"$TEST_EMAIL\",\"password\":\"Pw123456!\",\"first_name\":\"E2E\",\"last_name\":\"User\",\"phone\":\"0812\"}")
NEW_PID=$(echo "$create_resp" | jq -r '.pid // empty')
if [ -n "$NEW_PID" ] && [ "$NEW_PID" != "null" ]; then
  ok "created (pid=$NEW_PID)"
else
  ko "create failed: $create_resp"
fi

# 3. List users — new user must appear
step "List users"
list_resp=$(curl -sS -m 15 "$BASE_URL/api/users" "${AUTH[@]}")
if echo "$list_resp" | jq -e --arg e "$TEST_EMAIL" 'map(select(.email == $e)) | length == 1' >/dev/null 2>&1; then
  ok "new user present in list"
else
  ko "new user missing from list"
fi

# 4. Get the user by pid
step "Get user by pid"
get_resp=$(curl -sS -m 15 "$BASE_URL/api/user/$NEW_PID" "${AUTH[@]}")
if [ "$(echo "$get_resp" | jq -r '.email // empty')" = "$TEST_EMAIL" ]; then
  ok "get returned the user"
else
  ko "get failed: $get_resp"
fi

# 5. Update the user
step "Update user (first_name, phone)"
update_resp=$(curl -sS -m 15 -X PUT "$BASE_URL/api/user/$NEW_PID" "${AUTH[@]}" \
  -d '{"first_name":"E2E-Updated","phone":"0999"}')
if [ "$(echo "$update_resp" | jq -r '.first_name // empty')" = "E2E-Updated" ]; then
  ok "update applied"
else
  ko "update failed: $update_resp"
fi

# 6. Delete the user
step "Delete user"
del_code=$(curl -sS -m 15 -o /dev/null -w "%{http_code}" -X DELETE "$BASE_URL/api/user/$NEW_PID" "${AUTH[@]}")
if [ "$del_code" = "200" ]; then
  ok "deleted (HTTP $del_code)"
else
  ko "delete failed: HTTP $del_code"
fi

# 7. Verify the user is gone
step "Verify user is gone"
get_code=$(curl -sS -m 15 -o /dev/null -w "%{http_code}" "$BASE_URL/api/user/$NEW_PID" "${AUTH[@]}")
if [ "$get_code" = "404" ]; then
  ok "user no longer found (HTTP 404)"
else
  ko "expected 404 after delete, got HTTP $get_code"
fi

echo ""
echo "RESULT: $pass passed, $fail failed"
[ "$fail" -eq 0 ]
