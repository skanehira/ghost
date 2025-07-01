#!/bin/bash

# Run all E2E tests for ghost

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
E2E_DIR="./e2e"
SUMMARY_LOG="/tmp/ghost_e2e_summary.log"

# Test results
declare -a PASSED_TESTS=()
declare -a FAILED_TESTS=()

# Helper functions
log() {
    echo -e "${BLUE}[E2E RUNNER]${NC} $1" | tee -a "$SUMMARY_LOG"
}

success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1" | tee -a "$SUMMARY_LOG"
}

failure() {
    echo -e "${RED}[FAILURE]${NC} $1" | tee -a "$SUMMARY_LOG"
}

# Run a single test
run_test() {
    local test_script="$1"
    local test_name
    test_name=$(basename "$test_script" .sh)
    
    log "Running $test_name..."
    
    if "$test_script"; then
        success "$test_name passed"
        PASSED_TESTS+=("$test_name")
        return 0
    else
        failure "$test_name failed"
        FAILED_TESTS+=("$test_name")
        return 1
    fi
}

# Main execution
main() {
    log "=== Ghost E2E Test Suite ==="
    log "Summary log: $SUMMARY_LOG"
    echo > "$SUMMARY_LOG"  # Clear summary log
    
    # Ensure we're in the right directory
    if [[ ! -d "$E2E_DIR" ]]; then
        failure "E2E directory not found: $E2E_DIR"
        exit 1
    fi
    
    # Build release binary if needed
    if [[ ! -f "./target/release/ghost" ]]; then
        log "Building release binary..."
        cargo build --release
    fi
    
    # Find and run all test scripts (exclude helper files)
    local test_files
    test_files=($(find "$E2E_DIR" -name "test_*.sh" -type f ! -name "test_helpers.sh" | sort))
    
    if [[ ${#test_files[@]} -eq 0 ]]; then
        failure "No test files found in $E2E_DIR"
        exit 1
    fi
    
    log "Found ${#test_files[@]} test files"
    
    # Run each test
    local total_tests=${#test_files[@]}
    local current_test=0
    
    for test_file in "${test_files[@]}"; do
        current_test=$((current_test + 1))
        log "[$current_test/$total_tests] Testing $(basename "$test_file")"
        echo "----------------------------------------"
        
        # Run test (use gtimeout on macOS if available, otherwise run without timeout)
        if command -v gtimeout >/dev/null 2>&1; then
            if gtimeout 60s run_test "$test_file"; then
                echo # Add spacing
            else
                echo # Add spacing
            fi
        elif command -v timeout >/dev/null 2>&1; then
            if timeout 60s run_test "$test_file"; then
                echo # Add spacing
            else
                echo # Add spacing
            fi
        else
            if run_test "$test_file"; then
                echo # Add spacing
            else
                echo # Add spacing
            fi
        fi
    done
    
    # Print summary
    echo "========================================"
    log "Test Summary:"
    log "Total tests: $total_tests"
    log "Passed: ${#PASSED_TESTS[@]}"
    log "Failed: ${#FAILED_TESTS[@]}"
    
    if [[ ${#PASSED_TESTS[@]} -gt 0 ]]; then
        success "Passed tests:"
        for test in "${PASSED_TESTS[@]}"; do
            success "  - $test"
        done
    fi
    
    if [[ ${#FAILED_TESTS[@]} -gt 0 ]]; then
        failure "Failed tests:"
        for test in "${FAILED_TESTS[@]}"; do
            failure "  - $test"
        done
        
        log "Check individual test logs for details"
        exit 1
    else
        success "All tests passed! 🎉"
        exit 0
    fi
}

# Run with cleanup
cleanup() {
    # Kill any remaining ghost processes
    pkill -f "ghost" 2>/dev/null || true
    # Clean up any test files
    rm -f /tmp/ghost_e2e_*.log 2>/dev/null || true
}

trap cleanup EXIT

main "$@"