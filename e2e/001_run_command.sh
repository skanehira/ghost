#!/bin/bash

# E2E test for ghost run command
# Test that ghost status and ps command show consistent results

# Load test helpers
source "$(dirname "$0")/test_helpers.sh"

# Test-specific configuration
LOG_FILE="/tmp/ghost_e2e_test.log"

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    # Kill any remaining test processes
    if [[ -n "${TASK_ID:-}" ]]; then
        $GHOST_BIN stop "$TASK_ID" 2>/dev/null || true
    fi
    # Clean up log file
    rm -f "$LOG_FILE"
}

# Set up cleanup trap
trap cleanup EXIT

# Test-specific helper functions

# Extract PID from ghost status output
extract_pid_from_ghost_status() {
    local task_id="$1"
    local status_output
    status_output=$($GHOST_BIN status "$task_id" 2>/dev/null)
    echo "$status_output" | grep "^PID:" | awk '{print $2}'
}

# Check if process exists using ps
check_process_with_ps() {
    local pid="$1"
    ps -p "$pid" >/dev/null 2>&1
}

# Main test function
test_run_command() {
    log "Starting run command E2E test"
    
    # Step 1: Run a long-running command
    log "Step 1: Running background command..."
    local run_output
    run_output=$($GHOST_BIN run "$TEST_SCRIPT" 2>&1)
    
    # Extract task ID from output
    TASK_ID=$(echo "$run_output" | grep "Task ID:" | awk '{print $3}')
    if [[ -z "$TASK_ID" ]]; then
        error "Failed to extract task ID from run output: $run_output"
    fi
    log "Started task with ID: $TASK_ID"
    
    # Step 2: Get PID from ghost status
    log "Step 2: Getting PID from ghost status..."
    local ghost_pid
    ghost_pid=$(extract_pid_from_ghost_status "$TASK_ID")
    if [[ -z "$ghost_pid" ]]; then
        error "Failed to extract PID from ghost status"
    fi
    log "Ghost reports PID: $ghost_pid"
    
    # Step 3: Verify process exists with ps
    log "Step 3: Verifying process exists with ps..."
    if check_process_with_ps "$ghost_pid"; then
        log "✓ Process $ghost_pid exists in ps output"
    else
        error "✗ Process $ghost_pid not found in ps output"
    fi
    
    # Step 4: Get detailed process info from ps
    log "Step 4: Getting detailed process info..."
    local ps_output
    ps_output=$(ps -p "$ghost_pid" -o pid,ppid,pgid,command 2>/dev/null)
    log "Process details from ps:"
    echo "$ps_output" | tee -a "$LOG_FILE"
    
    # Step 5: Verify ghost status shows running
    log "Step 5: Verifying ghost status shows 'running'..."
    local ghost_status_output
    ghost_status_output=$($GHOST_BIN status "$TASK_ID" 2>/dev/null)
    local status_line
    status_line=$(echo "$ghost_status_output" | grep "^Status:")
    if echo "$status_line" | grep -q "running"; then
        log "✓ Ghost status shows 'running'"
    else
        error "✗ Ghost status does not show 'running': $status_line"
    fi
    
    # Step 6: Stop the process and verify it's gone
    log "Step 6: Stopping the process..."
    $GHOST_BIN stop "$TASK_ID" >/dev/null 2>&1 || true
    
    # Wait a moment for the process to die
    sleep 1
    
    if check_process_with_ps "$ghost_pid"; then
        warn "Process $ghost_pid still exists after stop command"
    else
        log "✓ Process $ghost_pid successfully stopped"
    fi
    
    # Step 7: Verify ghost status reflects the change
    log "Step 7: Verifying ghost status after stop..."
    ghost_status_output=$($GHOST_BIN status "$TASK_ID" 2>/dev/null)
    status_line=$(echo "$ghost_status_output" | grep "^Status:")
    if echo "$status_line" | grep -q "exited"; then
        log "✓ Ghost status shows 'exited' after stop"
    else
        warn "Ghost status after stop: $status_line"
    fi
    
    log "Run command E2E test completed successfully!"
}

# Main execution
main() {
    log "=== Ghost Run Command E2E Test ==="
    log "Log file: $LOG_FILE"
    
    test_prerequisites
    test_run_command
    
    log "=== All tests passed! ==="
}

# Run the test
main "$@"
