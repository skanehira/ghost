#!/bin/bash

# E2E test for ghost stop and status commands
# Test process stopping and status checking functionality

# Load test helpers
source "$(dirname "$0")/test_helpers.sh"

# Test-specific configuration
LOG_FILE="/tmp/ghost_e2e_stop_status_test.log"

# Test data
declare -a TASK_IDS=()

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    # Kill any remaining test processes
    for task_id in "${TASK_IDS[@]}"; do
        $GHOST_BIN stop "$task_id" 2>/dev/null || true
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

# Test status command
test_status_command() {
    log "=== Testing Status Command ==="
    
    # Step 1: Start a long-running task
    log "Step 1: Starting long-running task for status test..."
    local run_output
    run_output=$($GHOST_BIN run "$TEST_SCRIPT" 2>&1)
    local task_id
    task_id=$(extract_task_id "$run_output")
    TASK_IDS+=("$task_id")
    log "Started task: $task_id"
    
    # Step 2: Check status immediately
    log "Step 2: Checking status of running task..."
    local status_output
    status_output=$($GHOST_BIN status "$task_id" 2>/dev/null)
    
    # Verify status contains expected fields
    if echo "$status_output" | grep -q "Task: $task_id"; then
        log "✓ Status shows correct task ID"
    else
        error "✗ Status does not show correct task ID"
    fi
    
    if echo "$status_output" | grep -q "Status: running"; then
        log "✓ Status shows 'running'"
    else
        error "✗ Status does not show 'running'"
    fi
    
    if echo "$status_output" | grep -q "PID:"; then
        log "✓ Status shows PID"
    else
        error "✗ Status does not show PID"
    fi
    
    if echo "$status_output" | grep -q "Command:"; then
        log "✓ Status shows command"
    else
        error "✗ Status does not show command"
    fi
    
    if echo "$status_output" | grep -q "Started:"; then
        log "✓ Status shows start time"
    else
        error "✗ Status does not show start time"
    fi
    
    # Step 3: Verify PID matches ps output
    log "Step 3: Verifying PID consistency with ps..."
    local pid
    pid=$(extract_pid_from_status "$status_output")
    
    if check_process_with_ps "$pid"; then
        log "✓ PID $pid exists in ps output"
    else
        error "✗ PID $pid not found in ps output"
    fi
    
    # Step 4: Test status with non-existent task
    log "Step 4: Testing status with non-existent task..."
    local fake_task_id="non-existent-task-id"
    local error_output
    if error_output=$($GHOST_BIN status "$fake_task_id" 2>&1); then
        error "✗ Expected error for non-existent task, but command succeeded"
    else
        if echo "$error_output" | grep -q "Error"; then
            log "✓ Non-existent task produces appropriate error"
        else
            error "✗ Unexpected error message: $error_output"
        fi
    fi
    
    log "Status command tests completed"
    return 0  # Keep task running for stop tests
}

# Test stop command
test_stop_command() {
    log "=== Testing Stop Command ==="
    
    # Use the task from status test
    local task_id="${TASK_IDS[0]}"
    
    # Step 1: Get initial status
    log "Step 1: Getting initial status before stop..."
    local initial_status
    initial_status=$($GHOST_BIN status "$task_id" 2>/dev/null)
    local pid
    pid=$(extract_pid_from_status "$initial_status")
    
    if echo "$initial_status" | grep -q "Status: running"; then
        log "✓ Task is running before stop"
    else
        error "✗ Task is not running before stop test"
    fi
    
    # Step 2: Stop the task
    log "Step 2: Stopping the task..."
    local stop_output
    stop_output=$($GHOST_BIN stop "$task_id" 2>/dev/null)
    
    if echo "$stop_output" | grep -q "has been exited"; then
        log "✓ Stop command reports success"
    else
        error "✗ Stop command output unexpected: $stop_output"
    fi
    
    # Step 3: Verify process is gone
    log "Step 3: Verifying process is terminated..."
    sleep 1  # Give time for process to die
    
    if check_process_with_ps "$pid"; then
        error "✗ Process $pid still exists after stop"
    else
        log "✓ Process $pid successfully terminated"
    fi
    
    # Step 4: Check status after stop
    log "Step 4: Checking status after stop..."
    local final_status
    final_status=$($GHOST_BIN status "$task_id" 2>/dev/null)
    
    if echo "$final_status" | grep -q "Status: exited"; then
        log "✓ Status updated to 'exited'"
    else
        warn "Status not updated to 'exited' (may need manual refresh)"
    fi
    
    if echo "$final_status" | grep -q "Finished:"; then
        log "✓ Status shows finish time"
    else
        warn "Status does not show finish time"
    fi
    
    # Step 5: Test stop on already stopped task
    log "Step 5: Testing stop on already stopped task..."
    local double_stop_output
    if double_stop_output=$($GHOST_BIN stop "$task_id" 2>&1); then
        error "✗ Expected error for stopping already stopped task"
    else
        if echo "$double_stop_output" | grep -q "not running"; then
            log "✓ Stopping already stopped task produces appropriate error"
        else
            error "✗ Unexpected error for double stop: $double_stop_output"
        fi
    fi
    
    # Step 6: Test force stop
    log "Step 6: Testing force stop..."
    local force_run_output
    force_run_output=$($GHOST_BIN run "$TEST_SCRIPT" 2>&1)
    local force_task_id
    force_task_id=$(extract_task_id "$force_run_output")
    TASK_IDS+=("$force_task_id")
    
    # Wait for task to start
    sleep 1
    
    local force_stop_output
    force_stop_output=$($GHOST_BIN stop "$force_task_id" --force 2>/dev/null)
    
    if echo "$force_stop_output" | grep -q "has been killed"; then
        log "✓ Force stop reports 'killed'"
    else
        error "✗ Force stop output unexpected: $force_stop_output"
    fi
    
    # Step 7: Test stop with non-existent task
    log "Step 7: Testing stop with non-existent task..."
    local fake_task_id="non-existent-task-id"
    local stop_error_output
    if stop_error_output=$($GHOST_BIN stop "$fake_task_id" 2>&1); then
        error "✗ Expected error for stopping non-existent task"
    else
        if echo "$stop_error_output" | grep -q "Error"; then
            log "✓ Non-existent task stop produces appropriate error"
        else
            error "✗ Unexpected error message: $stop_error_output"
        fi
    fi
    
    log "Stop command tests completed"
}

# Main test function
test_stop_status_commands() {
    log "Starting stop and status commands E2E test"
    
    test_status_command
    test_stop_command
    
    log "Stop and status commands E2E test completed successfully!"
}

# Main execution
main() {
    log "=== Ghost Stop and Status Commands E2E Test ==="
    log "Log file: $LOG_FILE"
    
    test_prerequisites
    test_stop_status_commands
    
    log "=== All tests passed! ==="
}

# Run the test
main "$@"