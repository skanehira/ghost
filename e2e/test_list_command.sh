#!/bin/bash

# E2E test for ghost list command
# Test task listing functionality with filtering

# Load test helpers
source "$(dirname "$0")/test_helpers.sh"

# Test-specific configuration
LOG_FILE="/tmp/ghost_e2e_list_test.log"

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

# Count tasks in list output
count_tasks_in_list() {
    local list_output="$1"
    # Skip header and separator line, count remaining lines
    echo "$list_output" | tail -n +3 | wc -l | tr -d ' '
}

# Check if task appears in list output
task_in_list() {
    local task_id="$1"
    local list_output="$2"
    echo "$list_output" | grep -q "${task_id:0:8}"
}


# Main test function
test_list_command() {
    log "Starting list command E2E test"
    
    # Step 1: Check initial state (should be empty or have previous tasks)
    log "Step 1: Checking initial task list..."
    local initial_list
    initial_list=$($GHOST_BIN list 2>/dev/null)
    local initial_count
    initial_count=$(count_tasks_in_list "$initial_list")
    log "Initial task count: $initial_count"
    
    # Step 2: Start multiple tasks
    log "Step 2: Starting multiple background tasks..."
    
    # Start first task
    local run_output1
    run_output1=$($GHOST_BIN run "$TEST_SCRIPT" 2>&1)
    local task_id1
    task_id1=$(extract_task_id "$run_output1")
    TASK_IDS+=("$task_id1")
    log "Started task 1: $task_id1"
    
    # Start second task
    local run_output2
    run_output2=$($GHOST_BIN run echo "test task 2" 2>&1)
    local task_id2
    task_id2=$(extract_task_id "$run_output2")
    TASK_IDS+=("$task_id2")
    log "Started task 2: $task_id2"
    
    # Start third task
    local run_output3
    run_output3=$($GHOST_BIN run echo "test task 3" 2>&1)
    local task_id3
    task_id3=$(extract_task_id "$run_output3")
    TASK_IDS+=("$task_id3")
    log "Started task 3: $task_id3"
    
    # Wait a moment for tasks to be processed
    sleep 1
    
    # Step 3: Test basic list functionality
    log "Step 3: Testing basic list command..."
    local list_output
    list_output=$($GHOST_BIN list 2>/dev/null)
    local current_count
    current_count=$(count_tasks_in_list "$list_output")
    
    if [[ $current_count -ge 3 ]]; then
        log "✓ List shows at least 3 tasks (found $current_count)"
    else
        error "✗ Expected at least 3 tasks, found $current_count"
    fi
    
    # Step 4: Verify all tasks appear in list
    log "Step 4: Verifying all tasks appear in list..."
    for task_id in "${TASK_IDS[@]}"; do
        if task_in_list "$task_id" "$list_output"; then
            log "✓ Task ${task_id:0:8} found in list"
        else
            error "✗ Task ${task_id:0:8} not found in list"
        fi
    done
    
    # Step 5: Test status filtering - running tasks
    log "Step 5: Testing status filter for running tasks..."
    local running_list
    running_list=$($GHOST_BIN list --status running 2>/dev/null)
    
    # At least task1 should be running (hello_loop.sh)
    if task_in_list "$task_id1" "$running_list"; then
        log "✓ Running task found in --status running filter"
    else
        error "✗ Running task not found in --status running filter"
    fi
    
    # Step 6: Stop one task and test status filtering
    log "Step 6: Stopping task 1 and testing exited filter..."
    $GHOST_BIN stop "$task_id1" >/dev/null 2>&1
    
    # Wait for status update
    sleep 1
    
    # Test exited filter
    local exited_list
    exited_list=$($GHOST_BIN list --status exited 2>/dev/null)
    
    # Should have at least 2 exited tasks (task2, task3 finished naturally, task1 stopped)
    local exited_count
    exited_count=$(count_tasks_in_list "$exited_list")
    if [[ $exited_count -ge 2 ]]; then
        log "✓ Exited filter shows at least 2 tasks (found $exited_count)"
    else
        warn "Expected at least 2 exited tasks, found $exited_count (tasks might still be running)"
    fi
    
    # Step 7: Test invalid status filter
    log "Step 7: Testing invalid status filter..."
    local invalid_output
    if invalid_output=$($GHOST_BIN list --status invalid 2>/dev/null); then
        local invalid_count
        invalid_count=$(count_tasks_in_list "$invalid_output")
        if [[ $invalid_count -eq 0 ]]; then
            log "✓ Invalid status filter returns no results"
        else
            warn "Invalid status filter returned $invalid_count results"
        fi
    else
        log "✓ Invalid status filter handled gracefully"
    fi
    
    # Step 8: Verify list output format
    log "Step 8: Verifying list output format..."
    local full_list
    full_list=$($GHOST_BIN list 2>/dev/null)
    
    # Check header
    if echo "$full_list" | head -1 | grep -q "Task ID"; then
        log "✓ List output has proper header"
    else
        error "✗ List output header is malformed"
    fi
    
    # Check separator line
    if echo "$full_list" | sed -n '2p' | grep -q "^-*$"; then
        log "✓ List output has separator line"
    else
        error "✗ List output separator line is malformed"
    fi
    
    log "List command E2E test completed successfully!"
}

# Main execution
main() {
    log "=== Ghost List Command E2E Test ==="
    log "Log file: $LOG_FILE"
    
    test_prerequisites
    test_list_command
    
    log "=== All tests passed! ==="
}

# Run the test
main "$@"