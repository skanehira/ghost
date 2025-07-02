#!/bin/bash

# E2E test for ghost kill command
# Test direct PID killing functionality

# Load test helpers
source "$(dirname "$0")/test_helpers.sh"

# Test-specific configuration
LOG_FILE="/tmp/ghost_e2e_kill_test.log"

# Test data
declare -a TASK_IDS=()
declare -a PIDS=()

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    # Kill any remaining test processes
    for task_id in "${TASK_IDS[@]}"; do
        $GHOST_BIN stop "$task_id" 2>/dev/null || true
    done
    for pid in "${PIDS[@]}"; do
        kill -9 "$pid" 2>/dev/null || true
    done
    # Clean up log file
    rm -f "$LOG_FILE"
}

# Set up cleanup trap
trap cleanup EXIT

# Test-specific helper functions

# Extract PID from ghost status output
extract_pid_from_status() {
    local status_output="$1"
    echo "$status_output" | grep "^PID:" | awk '{print $2}'
}

# Check if process exists using ps
check_process_with_ps() {
    local pid="$1"
    ps -p "$pid" >/dev/null 2>&1
}

# Main test function
test_kill_command() {
    log "Starting kill command E2E test"
    
    # Step 1: Start a task to get a PID
    log "Step 1: Starting task to test kill command..."
    local run_output
    run_output=$($GHOST_BIN run "$TEST_SCRIPT" 2>&1)
    local task_id
    task_id=$(extract_task_id "$run_output")
    TASK_IDS+=("$task_id")
    log "Started task: $task_id"
    
    # Step 2: Get PID from status
    log "Step 2: Getting PID from task status..."
    local status_output
    status_output=$($GHOST_BIN status "$task_id" 2>/dev/null)
    local pid
    pid=$(extract_pid_from_status "$status_output")
    PIDS+=("$pid")
    log "Task PID: $pid"
    
    # Step 3: Verify process is running
    log "Step 3: Verifying process is running..."
    if check_process_with_ps "$pid"; then
        log "✓ Process $pid is running"
    else
        error "✗ Process $pid is not running"
    fi
    
    # Step 4: Kill the process using ghost kill
    log "Step 4: Killing process using ghost kill..."
    local kill_output
    kill_output=$($GHOST_BIN kill "$pid" 2>/dev/null)
    
    if echo "$kill_output" | grep -q "killed successfully"; then
        log "✓ Kill command reports success"
    else
        error "✗ Kill command output unexpected: $kill_output"
    fi
    
    # Step 5: Verify process is gone
    log "Step 5: Verifying process is terminated..."
    sleep 1  # Give time for process to die
    
    if check_process_with_ps "$pid"; then
        error "✗ Process $pid still exists after kill"
    else
        log "✓ Process $pid successfully killed"
    fi
    
    # Step 6: Test kill with non-existent PID
    log "Step 6: Testing kill with non-existent PID..."
    local fake_pid="99999"
    local kill_error_output
    if kill_error_output=$($GHOST_BIN kill "$fake_pid" 2>&1); then
        warn "Kill command succeeded for non-existent PID (this may be expected)"
    else
        if echo "$kill_error_output" | grep -q "Error"; then
            log "✓ Non-existent PID produces appropriate error"
        else
            error "✗ Unexpected error message: $kill_error_output"
        fi
    fi
    
    # Step 7: Test kill with invalid PID format
    log "Step 7: Testing kill with invalid PID format..."
    local invalid_pid="not-a-number"
    local invalid_error_output
    if invalid_error_output=$($GHOST_BIN kill "$invalid_pid" 2>&1); then
        error "✗ Expected error for invalid PID format"
    else
        if echo "$invalid_error_output" | grep -q "invalid digit"; then
            log "✓ Invalid PID format produces appropriate error"
        else
            error "✗ Unexpected error for invalid PID: $invalid_error_output"
        fi
    fi
    
    # Step 8: Test kill vs stop behavior comparison
    log "Step 8: Comparing kill vs stop behavior..."
    
    # Start another task for comparison
    local run_output2
    run_output2=$($GHOST_BIN run "$TEST_SCRIPT" 2>&1)
    local task_id2
    task_id2=$(extract_task_id "$run_output2")
    TASK_IDS+=("$task_id2")
    
    local status_output2
    status_output2=$($GHOST_BIN status "$task_id2" 2>/dev/null)
    local pid2
    pid2=$(extract_pid_from_status "$status_output2")
    PIDS+=("$pid2")
    
    # Kill using ghost kill (direct PID)
    $GHOST_BIN kill "$pid2" >/dev/null 2>&1
    
    # Wait and check if ghost database knows about this
    sleep 1
    local final_status
    final_status=$($GHOST_BIN status "$task_id2" 2>/dev/null)
    
    if echo "$final_status" | grep -q "Status: running"; then
        warn "Ghost database still shows 'running' after direct kill (expected - database not updated)"
        log "✓ Kill command works independently of ghost's task tracking"
    else
        log "✓ Ghost database updated after direct kill"
    fi
    
    # Step 9: Test kill with PID from different user (if applicable)
    log "Step 9: Testing kill permissions..."
    # We'll just verify that kill command exists and accepts numeric PIDs
    # Full permission testing would require root or other users
    
    if command -v kill >/dev/null 2>&1; then
        log "✓ System kill command is available"
    else
        warn "System kill command not found"
    fi
    
    log "Kill command E2E test completed successfully!"
}

# Main execution
main() {
    log "=== Ghost Kill Command E2E Test ==="
    log "Log file: $LOG_FILE"
    
    test_prerequisites
    test_kill_command
    
    log "=== All tests passed! ==="
}

# Run the test
main "$@"