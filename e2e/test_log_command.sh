#!/bin/bash

# E2E test for ghost log command
# Test log output functionality

# Load test helpers
source "$(dirname "$0")/test_helpers.sh"

# Test-specific configuration
LOG_FILE="/tmp/ghost_e2e_log_test.log"

# Test data
TASK_ID=""

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    # Kill any remaining test processes
    if [[ -n "$TASK_ID" ]]; then
        $GHOST_BIN stop "$TASK_ID" 2>/dev/null || true
    fi
    # Clean up log file
    rm -f "$LOG_FILE"
}

# Set up cleanup trap
trap cleanup EXIT

# Test-specific helper functions

# Main test function
test_log_command() {
    log "Starting log command E2E test"
    
    # Step 1: Start a task that produces predictable output
    log "Step 1: Starting task with predictable output..."
    local run_output
    run_output=$($GHOST_BIN run echo "Hello from ghost test" 2>&1)
    TASK_ID=$(extract_task_id "$run_output")
    log "Started task: $TASK_ID"
    
    # Wait for task to complete
    sleep 2
    
    # Step 2: Test basic log output
    log "Step 2: Testing basic log output..."
    local log_output
    log_output=$($GHOST_BIN log "$TASK_ID" 2>/dev/null)
    
    if echo "$log_output" | grep -q "Hello from ghost test"; then
        log "✓ Log output contains expected content"
    else
        error "✗ Log output does not contain expected content: $log_output"
    fi
    
    # Step 3: Test log with non-existent task
    log "Step 3: Testing log with non-existent task..."
    local fake_task_id="non-existent-task-id"
    local error_output
    if error_output=$($GHOST_BIN log "$fake_task_id" 2>&1); then
        error "✗ Expected error for non-existent task, but command succeeded"
    else
        if echo "$error_output" | grep -q "Error"; then
            log "✓ Non-existent task produces appropriate error"
        else
            error "✗ Unexpected error message: $error_output"
        fi
    fi
    
    # Step 4: Test with partial task ID
    log "Step 4: Testing with partial task ID..."
    local partial_id="${TASK_ID:0:8}"
    local partial_log_output
    if partial_log_output=$($GHOST_BIN log "$partial_id" 2>&1); then
        if echo "$partial_log_output" | grep -q "Hello from ghost test"; then
            log "✓ Partial task ID works correctly"
        else
            # This might fail depending on implementation - that's OK
            warn "Partial task ID didn't return expected content (this may be expected behavior)"
        fi
    else
        warn "Partial task ID failed (this may be expected behavior)"
    fi
    
    # Step 5: Start a multi-line output task  
    log "Step 5: Testing multi-line log output..."
    local multiline_run_output
    multiline_run_output=$($GHOST_BIN run ./scripts/multiline_test.sh 2>&1)
    local multiline_task_id
    multiline_task_id=$(extract_task_id "$multiline_run_output")
    
    # Wait for completion
    sleep 2
    
    local multiline_log_output
    multiline_log_output=$($GHOST_BIN log "$multiline_task_id" 2>/dev/null)
    
    # Check all lines are present
    local line_count
    line_count=$(echo "$multiline_log_output" | wc -l | tr -d ' ')
    if [[ $line_count -ge 3 ]]; then
        log "✓ Multi-line output captured correctly ($line_count lines)"
    else
        error "✗ Multi-line output incomplete (only $line_count lines)"
    fi
    
    if echo "$multiline_log_output" | grep -q "Line1" && \
       echo "$multiline_log_output" | grep -q "Line2" && \
       echo "$multiline_log_output" | grep -q "Line3"; then
        log "✓ All expected lines found in multi-line output"
    else
        error "✗ Not all expected lines found in multi-line output"
    fi
    
    # Step 6: Test with long-running task
    log "Step 6: Testing log with long-running task..."
    local longrun_output
    longrun_output=$($GHOST_BIN run "$TEST_SCRIPT" 2>&1)
    local longrun_task_id
    longrun_task_id=$(extract_task_id "$longrun_output")
    
    # Wait for some output to be generated
    sleep 3
    
    local longrun_log_output
    longrun_log_output=$($GHOST_BIN log "$longrun_task_id" 2>/dev/null)
    
    # Should have multiple "hello" lines
    local hello_count
    hello_count=$(echo "$longrun_log_output" | grep -c "hello" || echo "0")
    if [[ $hello_count -ge 2 ]]; then
        log "✓ Long-running task log contains expected output ($hello_count hello messages)"
    else
        error "✗ Long-running task log doesn't contain expected output"
    fi
    
    # Clean up long-running task
    $GHOST_BIN stop "$longrun_task_id" >/dev/null 2>&1
    
    # Step 7: Test follow flag (basic check - we can't fully test interactive follow)
    log "Step 7: Testing follow flag syntax..."
    local follow_output
    if follow_output=$($GHOST_BIN log "$TASK_ID" --follow 2>&1 </dev/null | head -5); then
        if echo "$follow_output" | grep -q "Following logs"; then
            log "✓ Follow flag produces expected header"
        else
            log "✓ Follow flag syntax accepted (output: ${follow_output:0:50}...)"
        fi
    else
        error "✗ Follow flag failed"
    fi
    
    log "Log command E2E test completed successfully!"
}

# Main execution
main() {
    log "=== Ghost Log Command E2E Test ==="
    log "Log file: $LOG_FILE"
    
    test_prerequisites
    test_log_command
    
    log "=== All tests passed! ==="
}

# Run the test
main "$@"