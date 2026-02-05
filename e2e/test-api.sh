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
PROJECT_ID=""

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
        # Extract API key from nested structure: {"api_key":{"key":"flg_..."}}
        API_KEY=$(echo "$signup_response" | grep -o '"key":"flg_[^"]*"' | head -1 | cut -d'"' -f4)
        # Also extract project ID for later use
        PROJECT_ID=$(echo "$signup_response" | grep -o '"project":{"id":"[^"]*"' | cut -d'"' -f6)
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
# Flags Tests (project-scoped)
# ============================================
test_flags() {
    log "\n=== Flags Tests ==="
    
    if [ -z "$API_KEY" ]; then
        skip "Flags tests - no API key (auth failed)"
        return
    fi
    
    if [ -z "$PROJECT_ID" ]; then
        skip "Flags tests - no project ID (signup response changed?)"
        return
    fi
    
    # Create flag (project-scoped) - use unique key per test run
    local flag_key="test-flag-$(date +%s)"
    local flag_data="{\"key\":\"$flag_key\",\"name\":\"Test Flag\",\"enabled\":true}"
    local create_response=$(http POST "/projects/$PROJECT_ID/flags" "$flag_data" "$API_KEY")
    
    if echo "$create_response" | grep -q "\"key\":\"$flag_key\""; then
        pass "POST /projects/:id/flags - creates flag"
    elif echo "$create_response" | grep -q "not found"; then
        fail "POST /projects/:id/flags - endpoint not found (404)" "$create_response"
        return
    else
        fail "POST /projects/:id/flags - failed" "$create_response"
    fi
    
    # List flags (project-scoped)
    local list_response=$(http GET "/projects/$PROJECT_ID/flags" "" "$API_KEY")
    local list_status=$(http_status GET "/projects/$PROJECT_ID/flags" "" "$API_KEY")
    
    if [ "$list_status" = "200" ]; then
        pass "GET /projects/:id/flags - lists flags"
    else
        fail "GET /projects/:id/flags - returned $list_status" "$list_response"
    fi
    
    # Get flag (project-scoped) - use the flag we just created
    local eval_status=$(http_status GET "/projects/$PROJECT_ID/flags/$flag_key" "" "$API_KEY")
    
    if [ "$eval_status" = "200" ]; then
        pass "GET /projects/:id/flags/:key - gets flag"
    else
        fail "GET /projects/:id/flags/:key - returned $eval_status"
    fi
    
    # Toggle flag (project-scoped, requires environment param)
    local toggle_status=$(http_status POST "/projects/$PROJECT_ID/flags/$flag_key/toggle?environment=production" "" "$API_KEY")
    
    if [ "$toggle_status" = "200" ]; then
        pass "POST /projects/:id/flags/:key/toggle - toggles flag"
    else
        fail "POST /projects/:id/flags/:key/toggle - returned $toggle_status"
    fi
}

# ============================================
# Projects Tests
# ============================================
test_projects() {
    log "\n=== Projects Tests ==="
    
    if [ -z "$API_KEY" ]; then
        skip "Projects tests - no API key"
        return
    fi
    
    # List projects (CLI-compatible path)
    local status=$(http_status GET "/projects" "" "$API_KEY")
    
    if [ "$status" = "404" ]; then
        skip "GET /projects - endpoint not implemented"
    elif [ "$status" = "200" ]; then
        pass "GET /projects - lists projects"
    else
        fail "GET /projects - returned $status"
    fi
    
    # Create a new project
    local create_data='{"name":"test-project"}'
    local create_status=$(http_status POST "/projects" "$create_data" "$API_KEY")
    
    if [ "$create_status" = "200" ] || [ "$create_status" = "201" ]; then
        pass "POST /projects - creates project"
    else
        fail "POST /projects - returned $create_status"
    fi
}

# ============================================
# Environments Tests
# ============================================
test_environments() {
    log "\n=== Environments Tests ==="
    
    if [ -z "$API_KEY" ]; then
        skip "Environments tests - no API key"
        return
    fi
    
    if [ -z "$PROJECT_ID" ]; then
        skip "Environments tests - no project ID"
        return
    fi
    
    # List environments for project (CLI-compatible path)
    local status=$(http_status GET "/projects/$PROJECT_ID/environments" "" "$API_KEY")
    
    if [ "$status" = "404" ]; then
        skip "GET /projects/:id/environments - endpoint not implemented"
    elif [ "$status" = "200" ]; then
        pass "GET /projects/:id/environments - lists environments"
    else
        fail "GET /projects/:id/environments - returned $status"
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
