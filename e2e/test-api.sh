#!/bin/bash
# FlagLite E2E API Tests
# Run against live or local API

set -e

# Configuration
API_URL="${FLAGLITE_API_URL:-https://api.flaglite.dev}"
VERBOSE="${VERBOSE:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

# Test counters
PASSED=0
FAILED=0
SKIPPED=0

# Test user credentials (generated per run)
TEST_USER="test-$(date +%s)"
TEST_PASSWORD="testpass123"
API_KEY=""
AUTH_TOKEN=""

log() {
    echo -e "$1"
}

pass() {
    log "${GREEN}✓${NC} $1"
    PASSED=$((PASSED + 1))
}

fail() {
    log "${RED}✗${NC} $1"
    if [ "$VERBOSE" = "true" ] && [ -n "$2" ]; then
        log "  Response: $2"
    fi
    FAILED=$((FAILED + 1))
}

skip() {
    log "${YELLOW}○${NC} $1 (skipped)"
    SKIPPED=$((SKIPPED + 1))
}

# HTTP helper
http() {
    local method=$1
    local endpoint=$2
    local data=$3
    local auth=$4
    
    local headers=(-H "Content-Type: application/json")
    
    if [ -n "$auth" ]; then
        headers+=(-H "Authorization: Bearer $auth")
    fi
    
    if [ -n "$data" ]; then
        curl -s -X "$method" "${API_URL}${endpoint}" "${headers[@]}" -d "$data"
    else
        curl -s -X "$method" "${API_URL}${endpoint}" "${headers[@]}"
    fi
}

http_status() {
    local method=$1
    local endpoint=$2
    local data=$3
    local auth=$4
    
    local headers=(-H "Content-Type: application/json")
    
    if [ -n "$auth" ]; then
        headers+=(-H "Authorization: Bearer $auth")
    fi
    
    if [ -n "$data" ]; then
        curl -s -o /dev/null -w "%{http_code}" -X "$method" "${API_URL}${endpoint}" "${headers[@]}" -d "$data"
    else
        curl -s -o /dev/null -w "%{http_code}" -X "$method" "${API_URL}${endpoint}" "${headers[@]}"
    fi
}

# ============================================
# Health Check
# ============================================
test_health() {
    log "\n=== Health Check ==="
    
    local status=$(http_status GET "/health")
    if [ "$status" = "200" ]; then
        pass "GET /health returns 200"
    else
        fail "GET /health returns $status (expected 200)"
    fi
}

# ============================================
# Auth Tests
# ============================================
test_auth() {
    log "\n=== Auth Tests ==="
    
    # Signup
    local signup_data="{\"username\":\"$TEST_USER\",\"password\":\"$TEST_PASSWORD\"}"
    local signup_response=$(http POST "/v1/auth/signup" "$signup_data")
    
    if echo "$signup_response" | grep -q "api_key"; then
        pass "POST /v1/auth/signup - creates user"
        API_KEY=$(echo "$signup_response" | grep -o '"api_key":"[^"]*"' | cut -d'"' -f4)
    else
        fail "POST /v1/auth/signup - failed" "$signup_response"
        return 1
    fi
    
    # Login
    local login_data="{\"username\":\"$TEST_USER\",\"password\":\"$TEST_PASSWORD\"}"
    local login_response=$(http POST "/v1/auth/login" "$login_data")
    
    if echo "$login_response" | grep -q "token"; then
        pass "POST /v1/auth/login - returns token"
        AUTH_TOKEN=$(echo "$login_response" | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
    else
        fail "POST /v1/auth/login - failed" "$login_response"
    fi
    
    # Me (with API key)
    local me_response=$(http GET "/v1/auth/me" "" "$API_KEY")
    
    if echo "$me_response" | grep -q "$TEST_USER"; then
        pass "GET /v1/auth/me (API key) - returns user info"
    else
        fail "GET /v1/auth/me (API key) - failed" "$me_response"
    fi
    
    # Signup with duplicate username
    local dup_status=$(http_status POST "/v1/auth/signup" "$signup_data")
    if [ "$dup_status" = "409" ] || [ "$dup_status" = "400" ]; then
        pass "POST /v1/auth/signup (duplicate) - rejects duplicate"
    else
        fail "POST /v1/auth/signup (duplicate) - returned $dup_status (expected 409 or 400)"
    fi
    
    # Invalid login
    local bad_login_data="{\"username\":\"$TEST_USER\",\"password\":\"wrongpassword\"}"
    local bad_login_status=$(http_status POST "/v1/auth/login" "$bad_login_data")
    if [ "$bad_login_status" = "401" ] || [ "$bad_login_status" = "400" ]; then
        pass "POST /v1/auth/login (wrong password) - rejects"
    else
        fail "POST /v1/auth/login (wrong password) - returned $bad_login_status (expected 401)"
    fi
}

# ============================================
# Flags Tests
# ============================================
test_flags() {
    log "\n=== Flags Tests ==="
    
    if [ -z "$API_KEY" ]; then
        skip "Flags tests - no API key (auth failed)"
        return
    fi
    
    # Create flag
    local flag_data='{"key":"test-flag","name":"Test Flag","enabled":true}'
    local create_response=$(http POST "/v1/flags" "$flag_data" "$API_KEY")
    local create_status=$(http_status POST "/v1/flags" "$flag_data" "$API_KEY")
    
    if [ "$create_status" = "201" ] || [ "$create_status" = "200" ]; then
        pass "POST /v1/flags - creates flag"
    elif [ "$create_status" = "404" ]; then
        fail "POST /v1/flags - endpoint not found (404)"
        return
    else
        fail "POST /v1/flags - returned $create_status" "$create_response"
    fi
    
    # List flags
    local list_response=$(http GET "/v1/flags" "" "$API_KEY")
    local list_status=$(http_status GET "/v1/flags" "" "$API_KEY")
    
    if [ "$list_status" = "200" ]; then
        pass "GET /v1/flags - lists flags"
    else
        fail "GET /v1/flags - returned $list_status" "$list_response"
    fi
    
    # Evaluate flag
    local eval_status=$(http_status GET "/v1/flags/test-flag" "" "$API_KEY")
    
    if [ "$eval_status" = "200" ]; then
        pass "GET /v1/flags/:key - evaluates flag"
    else
        fail "GET /v1/flags/:key - returned $eval_status"
    fi
    
    # Toggle flag
    local toggle_status=$(http_status POST "/v1/flags/test-flag/toggle" "" "$API_KEY")
    
    if [ "$toggle_status" = "200" ]; then
        pass "POST /v1/flags/:key/toggle - toggles flag"
    else
        fail "POST /v1/flags/:key/toggle - returned $toggle_status"
    fi
}

# ============================================
# Projects Tests (if implemented)
# ============================================
test_projects() {
    log "\n=== Projects Tests ==="
    
    if [ -z "$API_KEY" ]; then
        skip "Projects tests - no API key"
        return
    fi
    
    local status=$(http_status GET "/v1/projects" "" "$API_KEY")
    
    if [ "$status" = "404" ]; then
        skip "GET /v1/projects - endpoint not implemented"
    elif [ "$status" = "200" ]; then
        pass "GET /v1/projects - lists projects"
    else
        fail "GET /v1/projects - returned $status"
    fi
}

# ============================================
# Environments Tests (if implemented)
# ============================================
test_environments() {
    log "\n=== Environments Tests ==="
    
    if [ -z "$API_KEY" ]; then
        skip "Environments tests - no API key"
        return
    fi
    
    local status=$(http_status GET "/v1/environments" "" "$API_KEY")
    
    if [ "$status" = "404" ]; then
        skip "GET /v1/environments - endpoint not implemented"
    elif [ "$status" = "200" ]; then
        pass "GET /v1/environments - lists environments"
    else
        fail "GET /v1/environments - returned $status"
    fi
}

# ============================================
# Main
# ============================================
main() {
    log "FlagLite E2E API Tests"
    log "======================"
    log "API URL: $API_URL"
    log "Test user: $TEST_USER"
    
    test_health
    test_auth
    test_flags
    test_projects
    test_environments
    
    log "\n=== Summary ==="
    log "${GREEN}Passed: $PASSED${NC}"
    log "${RED}Failed: $FAILED${NC}"
    log "${YELLOW}Skipped: $SKIPPED${NC}"
    
    if [ $FAILED -gt 0 ]; then
        exit 1
    fi
}

main "$@"
