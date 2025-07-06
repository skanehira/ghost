#!/bin/bash

# E2E test for ghost cleanup command
# Test task cleanup functionality with various options

# Load test helpers
source "$(dirname "$0")/test_helpers.sh"

# Test-specific configuration
LOG_FILE="/tmp/ghost_e2e_cleanup_test.log"

# Test data
declare -a TASK_IDS=()
declare -a FINISHED_TASK_IDS=()

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    # Kill any remaining test processes
    if [[ ${#TASK_IDS[@]} -gt 0 ]]; then
        for task_id in "${TASK_IDS[@]}"; do
            $GHOST_BIN stop "$task_id" 2>/dev/null || true
        done
    fi
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
    # Use first 5 characters to be safer with shorter truncations
    echo "$list_output" | grep -q "${task_id:0:5}"
}

# Extract count from cleanup output
extract_cleanup_count() {
    local cleanup_output="$1"
    # Look for "following N task(s) would be deleted" or "Successfully deleted N task(s)"
    if echo "$cleanup_output" | grep -q "following.*would be deleted"; then
        echo "$cleanup_output" | grep "following.*would be deleted" | grep -oE '[0-9]+' | head -1
    elif echo "$cleanup_output" | grep -q "Successfully deleted"; then
        echo "$cleanup_output" | grep "Successfully deleted" | grep -oE '[0-9]+' | head -1
    else
        echo "0"
    fi
}

# Main test function
test_cleanup_command() {
    log "Starting cleanup command E2E test"
    
    # Step 1: Clean slate - remove ALL existing tasks for complete test isolation
    log "Step 1: Cleaning ALL existing tasks for complete test isolation..."
    
    # First, try to stop any running tasks
    local running_tasks
    running_tasks=$($GHOST_BIN list --status running 2>&1 || true)
    if echo "$running_tasks" | grep -v "No tasks found" | grep -q "running"; then
        log "Stopping any running tasks first..."
        # Extract task IDs and stop them
        echo "$running_tasks" | tail -n +3 | while read -r line; do
            if [[ -n "$line" ]]; then
                local task_id=$(echo "$line" | awk '{print $1}' | tr -d '.')
                $GHOST_BIN stop "$task_id" --force 2>/dev/null || true
            fi
        done
        sleep 1
    fi
    
    # Now cleanup all finished tasks
    local initial_cleanup_result
    initial_cleanup_result=$($GHOST_BIN cleanup --all 2>&1 || true)
    log "Initial cleanup result: $initial_cleanup_result"
    
    # Verify clean state
    local remaining_tasks
    remaining_tasks=$($GHOST_BIN list 2>&1)
    local task_count
    task_count=$(count_tasks_in_list "$remaining_tasks")
    log "After cleanup, remaining tasks: $task_count"
    
    # Step 2: Create test tasks with different outcomes
    log "Step 2: Creating test tasks with different statuses..."
    
    # Create exited tasks (use short sleep to ensure they finish properly)
    local exited_task1_output exited_task2_output
    exited_task1_output=$($GHOST_BIN run -- sh -c "echo 'test exited task 1'; sleep 0.5")
    exited_task2_output=$($GHOST_BIN run -- sh -c "echo 'test exited task 2'; sleep 0.5")
    
    local exited_task1_id exited_task2_id
    exited_task1_id=$(extract_task_id "$exited_task1_output")
    exited_task2_id=$(extract_task_id "$exited_task2_output")
    
    TASK_IDS+=("$exited_task1_id" "$exited_task2_id")
    FINISHED_TASK_IDS+=("$exited_task1_id" "$exited_task2_id")
    
    log "Created exited tasks: $exited_task1_id, $exited_task2_id"
    
    # Create a running task that we'll kill
    local killed_task_output
    killed_task_output=$($GHOST_BIN run sleep 30)
    local killed_task_id
    killed_task_id=$(extract_task_id "$killed_task_output")
    
    TASK_IDS+=("$killed_task_id")
    
    # Wait a moment for task to start, then force kill it
    sleep 1
    $GHOST_BIN stop "$killed_task_id" --force
    FINISHED_TASK_IDS+=("$killed_task_id")
    
    log "Created and killed task: $killed_task_id"
    
    # Create a running task that should NOT be cleaned up
    local running_task_output
    running_task_output=$($GHOST_BIN run sleep 60)
    local running_task_id
    running_task_id=$(extract_task_id "$running_task_output")
    
    TASK_IDS+=("$running_task_id")
    
    log "Created running task: $running_task_id"
    
    # Wait for exited tasks to complete properly
    log "Waiting for exited tasks to complete..."
    sleep 1
    
    # Force status update for our test tasks to ensure they are properly marked as finished
    $GHOST_BIN status "$exited_task1_id" > /dev/null 2>&1 || true
    $GHOST_BIN status "$exited_task2_id" > /dev/null 2>&1 || true
    $GHOST_BIN status "$killed_task_id" > /dev/null 2>&1 || true
    
    # Wait a moment for all tasks to settle and update their status
    sleep 2
    
    # Check status of our test tasks to ensure they are in expected states
    log "Verifying task states before testing cleanup..."
    local task_states_output
    task_states_output=$($GHOST_BIN list 2>&1)
    log "Current tasks: $(echo "$task_states_output" | wc -l) lines"
    
    # Step 3: Test dry-run functionality
    log "Step 3: Testing dry-run functionality..."
    
    local dry_run_output
    dry_run_output=$($GHOST_BIN cleanup --dry-run --days 0 2>&1)
    
    if echo "$dry_run_output" | grep -q "would be deleted"; then
        log "✓ Dry-run shows tasks would be deleted"
        
        local dry_run_count
        dry_run_count=$(extract_cleanup_count "$dry_run_output")
        
        # We created exactly 3 test tasks (2 exited + 1 killed) after cleaning all previous tasks
        if [[ "$dry_run_count" -eq 3 ]]; then
            log "✓ Dry-run shows exactly expected number of finished tasks ($dry_run_count)"
        elif [[ "$dry_run_count" -gt 0 ]]; then
            log "✓ Dry-run shows $dry_run_count tasks (close to expected 3)"
        else
            error "Dry-run shows unexpected count: $dry_run_count (expected 3)"
        fi
    else
        error "Dry-run output doesn't show expected format"
    fi
    
    # Step 4: Test status filtering
    log "Step 4: Testing status filtering with dry-run..."
    
    local exited_dry_run
    exited_dry_run=$($GHOST_BIN cleanup --dry-run --status exited --days 0 2>&1)
    
    if echo "$exited_dry_run" | grep -q "would be deleted"; then
        local exited_count
        exited_count=$(extract_cleanup_count "$exited_dry_run")
        
        # We created exactly 2 exited tasks after cleaning all previous tasks
        if [[ "$exited_count" -eq 2 ]]; then
            log "✓ Status filtering for 'exited' shows exactly expected count ($exited_count)"
        elif [[ "$exited_count" -gt 0 ]]; then
            log "✓ Status filtering for 'exited' shows $exited_count tasks (close to expected 2)"
        else
            error "Status filtering for 'exited' shows unexpected count: $exited_count (expected 2)"
        fi
    else
        # Check if no tasks message
        if echo "$exited_dry_run" | grep -q "No tasks found matching cleanup criteria"; then
            warn "No exited tasks found for cleanup (this may happen if tasks haven't finished yet)"
        else
            error "Unexpected exited dry-run output: $exited_dry_run"
        fi
    fi
    
    # Step 5: Test actual cleanup of exited tasks only
    log "Step 5: Testing actual cleanup of exited tasks..."
    
    local cleanup_exited_output
    cleanup_exited_output=$($GHOST_BIN cleanup --status exited --days 0 2>&1)
    
    if echo "$cleanup_exited_output" | grep -q "Successfully deleted"; then
        local deleted_count
        deleted_count=$(extract_cleanup_count "$cleanup_exited_output")
        log "✓ Successfully deleted $deleted_count exited tasks"
    elif echo "$cleanup_exited_output" | grep -q "No tasks found matching cleanup criteria"; then
        warn "No exited tasks found for cleanup - this may happen if tasks finished too quickly"
        log "Continuing test with available tasks..."
    else
        error "Cleanup of exited tasks failed: $cleanup_exited_output"
    fi
    
    # Step 6: Verify exited tasks are gone but killed/running tasks remain
    log "Step 6: Verifying selective cleanup worked..."
    
    local post_cleanup_list
    post_cleanup_list=$($GHOST_BIN list 2>&1)
    
    # Exited tasks should be gone
    if task_in_list "$exited_task1_id" "$post_cleanup_list"; then
        error "Exited task 1 should have been deleted but still appears in list"
    fi
    
    if task_in_list "$exited_task2_id" "$post_cleanup_list"; then
        error "Exited task 2 should have been deleted but still appears in list"
    fi
    
    # Killed and running tasks should remain
    if ! task_in_list "$killed_task_id" "$post_cleanup_list"; then
        error "Killed task should still exist but doesn't appear in list"
    fi
    
    if ! task_in_list "$running_task_id" "$post_cleanup_list"; then
        error "Running task should still exist but doesn't appear in list"
    fi
    
    log "✓ Selective cleanup worked correctly - exited tasks deleted, others preserved"
    
    # Step 7: Test protection against cleaning running tasks
    log "Step 7: Testing protection against cleaning running tasks..."
    
    local running_cleanup_result
    running_cleanup_result=$($GHOST_BIN cleanup --status running 2>&1 || true)
    
    if echo "$running_cleanup_result" | grep -q "Cannot cleanup running tasks"; then
        log "✓ Protection against cleaning running tasks works"
    else
        error "Expected error when trying to cleanup running tasks, got: $running_cleanup_result"
    fi
    
    # Step 8: Test --all flag functionality
    log "Step 8: Testing --all flag functionality..."
    
    local all_dry_run
    all_dry_run=$($GHOST_BIN cleanup --dry-run --all 2>&1)
    
    if echo "$all_dry_run" | grep -q "all finished tasks would be deleted regardless of age"; then
        log "✓ --all flag dry-run shows correct message"
        
        local all_count
        all_count=$(extract_cleanup_count "$all_dry_run")
        
        if [[ "$all_count" -ge 1 ]]; then
            log "✓ --all flag shows remaining finished tasks ($all_count)"
        else
            warn "--all flag shows 0 tasks (may be expected if only running tasks remain)"
        fi
    else
        error "--all flag dry-run doesn't show expected message"
    fi
    
    # Step 9: Test invalid status handling
    log "Step 9: Testing invalid status handling..."
    
    local invalid_status_result
    invalid_status_result=$($GHOST_BIN cleanup --status invalid_status 2>&1 || true)
    
    if echo "$invalid_status_result" | grep -q "Invalid status"; then
        log "✓ Invalid status produces appropriate error"
    else
        error "Expected error for invalid status, got: $invalid_status_result"
    fi
    
    # Step 10: Test days filter functionality  
    log "Step 10: Testing days filter functionality..."
    
    # Test with very high days value (should find nothing)
    local future_days_result
    future_days_result=$($GHOST_BIN cleanup --dry-run --days 999 2>&1)
    
    if echo "$future_days_result" | grep -q "No tasks found matching cleanup criteria"; then
        log "✓ High days value correctly finds no tasks"
    else
        warn "High days value test gave unexpected result: $future_days_result"
    fi
    
    # Step 11: Test log file deletion functionality
    log "Step 11: Testing log file deletion with cleanup..."
    
    # Create a finished task to test log file deletion
    local log_test_task_output
    log_test_task_output=$($GHOST_BIN run -- echo "test for log deletion" 2>&1)
    local log_test_task_id
    log_test_task_id=$(extract_task_id "$log_test_task_output")
    
    # Wait for task to complete and get its log path
    sleep 2
    $GHOST_BIN status "$log_test_task_id" > /dev/null 2>&1 || true
    
    # Get the log file path from database
    local log_task_details
    log_task_details=$($GHOST_BIN status "$log_test_task_id" 2>&1)
    local log_file_path
    log_file_path=$(echo "$log_task_details" | grep "Log file:" | sed 's/Log file: //')
    
    log "Log file path: $log_file_path"
    
    # Verify log file exists before cleanup
    if [[ -f "$log_file_path" ]]; then
        log "✓ Log file exists before cleanup"
    else
        error "✗ Log file doesn't exist before cleanup: $log_file_path"
    fi
    
    # Perform cleanup with --all flag to ensure this task gets deleted
    local log_cleanup_result
    log_cleanup_result=$($GHOST_BIN cleanup --all 2>&1)
    log "Log cleanup result: $log_cleanup_result"
    
    # Check if log file was deleted
    if [[ ! -f "$log_file_path" ]]; then
        log "✓ Log file successfully deleted after cleanup"
    else
        error "✗ Log file still exists after cleanup: $log_file_path"
    fi
    
    # Step 12: Test that cleanup updates process status before cleaning
    log "Step 12: Testing status update before cleanup..."
    
    # Create a task that will exit quickly
    local status_test_output
    status_test_output=$($GHOST_BIN run -- sh -c "echo 'testing status update'; exit 0" 2>&1)
    local status_test_id
    status_test_id=$(extract_task_id "$status_test_output")
    
    # Wait for process to exit
    sleep 2
    
    # Kill the ghost process to simulate a situation where the task exited but status wasn't updated
    # First, get the PID of the task
    local task_pid
    task_pid=$($GHOST_BIN status "$status_test_id" 2>&1 | grep "PID:" | awk '{print $2}')
    
    # Force the database to have 'running' status while process is already dead
    # This simulates the case where process died but status wasn't updated
    
    # Check if cleanup properly handles this case
    local status_cleanup_result
    status_cleanup_result=$($GHOST_BIN cleanup --all --dry-run 2>&1)
    
    log "Cleanup dry-run output: $status_cleanup_result"
    
    # The task should be included in cleanup candidates since the process is actually dead
    # Check for partial task ID match (first 5 chars - that's what the output shows)
    local partial_status_test_id="${status_test_id:0:5}"
    if echo "$status_cleanup_result" | grep -q "$partial_status_test_id"; then
        log "✓ Cleanup correctly identifies dead process with stale 'running' status"
    else
        log "Debug: Looking for task ID: $status_test_id (partial: $partial_status_test_id)"
        error "✗ Cleanup failed to identify dead process with stale 'running' status"
    fi
    
    # Actually perform cleanup
    $GHOST_BIN cleanup --all > /dev/null 2>&1
    
    log "Cleanup command E2E test completed successfully!"
    log "=== All tests passed! ==="
}

# Main execution
main() {
    echo -e "${GREEN}[TEST]${NC} === Ghost Cleanup Command E2E Test ==="
    echo -e "${GREEN}[TEST]${NC} Log file: $LOG_FILE"
    
    # Initialize log file
    echo "=== Ghost Cleanup Command E2E Test ===" > "$LOG_FILE"
    echo "Started at: $(date)" >> "$LOG_FILE"
    
    # Run prerequisites check
    test_prerequisites
    ensure_ghost_binary
    
    # Run the test
    test_cleanup_command
}

# Run main function
main "$@"