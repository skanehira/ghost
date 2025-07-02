#!/bin/bash

# E2E test for ghost run command with cwd functionality
# Test that current working directory is properly used when running commands

# Load test helpers
source "$(dirname "$0")/test_helpers.sh"

# Test-specific configuration
LOG_FILE="/tmp/ghost_e2e_cwd_test.log"

# Test data
declare -a TASK_IDS=()

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    # Kill any remaining test processes
    for task_id in "${TASK_IDS[@]}"; do
        $GHOST_BIN stop "$task_id" 2>/dev/null || true
    done
    # Clean up log file and test directories
    rm -f "$LOG_FILE"
    rm -rf /tmp/ghost_cwd_test_*
}

# Set up cleanup trap
trap cleanup EXIT

# Main test function
test_cwd_functionality() {
    log "Starting cwd functionality E2E test"
    
    # Step 1: Create test directories
    log "Step 1: Creating test directories..."
    local test_dir1="/tmp/ghost_cwd_test_dir1"
    local test_dir2="/tmp/ghost_cwd_test_dir2"
    
    mkdir -p "$test_dir1"
    mkdir -p "$test_dir2"
    
    # Create different content files in each directory
    echo "Content from dir1" > "$test_dir1/content.txt"
    echo "Content from dir2" > "$test_dir2/content.txt"
    
    log "Created test directories: $test_dir1 and $test_dir2"
    
    # Step 2: Run command in specific directory using --cwd
    log "Step 2: Running command with --cwd $test_dir1..."
    local run_output1
    run_output1=$($GHOST_BIN run --cwd "$test_dir1" cat content.txt 2>&1)
    local task_id1
    task_id1=$(extract_task_id "$run_output1")
    TASK_IDS+=("$task_id1")
    log "Started task 1 in dir1: $task_id1"
    
    # Step 3: Run command in different directory using --cwd
    log "Step 3: Running command with --cwd $test_dir2..."
    local run_output2
    run_output2=$($GHOST_BIN run --cwd "$test_dir2" cat content.txt 2>&1)
    local task_id2
    task_id2=$(extract_task_id "$run_output2")
    TASK_IDS+=("$task_id2")
    log "Started task 2 in dir2: $task_id2"
    
    # Step 4: Wait for tasks to complete
    log "Step 4: Waiting for tasks to complete..."
    sleep 2
    
    # Force status update to ensure tasks are marked as finished
    $GHOST_BIN status "$task_id1" > /dev/null 2>&1 || true
    $GHOST_BIN status "$task_id2" > /dev/null 2>&1 || true
    
    # Step 5: Check log outputs to verify cwd was used correctly
    log "Step 5: Verifying cwd was used correctly..."
    
    local log_content1
    log_content1=$($GHOST_BIN log "$task_id1" 2>&1)
    
    local log_content2
    log_content2=$($GHOST_BIN log "$task_id2" 2>&1)
    
    # Check if log content matches expected directory content
    if echo "$log_content1" | grep -q "Content from dir1"; then
        log "✓ Task 1 correctly executed in dir1 (found 'Content from dir1')"
    else
        error "✗ Task 1 did not execute in correct directory. Log content: $log_content1"
    fi
    
    if echo "$log_content2" | grep -q "Content from dir2"; then
        log "✓ Task 2 correctly executed in dir2 (found 'Content from dir2')"
    else
        error "✗ Task 2 did not execute in correct directory. Log content: $log_content2"
    fi
    
    # Step 6: Test that cwd is stored in database
    log "Step 6: Verifying cwd is stored in database..."
    local status_output1
    status_output1=$($GHOST_BIN status "$task_id1" 2>&1)
    
    local status_output2
    status_output2=$($GHOST_BIN status "$task_id2" 2>&1)
    
    # Check if status shows the cwd information
    if echo "$status_output1" | grep -q "Working directory: $test_dir1"; then
        log "✓ Task 1 status shows correct working directory"
    else
        error "✗ Task 1 status doesn't show working directory. Output: $status_output1"
    fi
    
    if echo "$status_output2" | grep -q "Working directory: $test_dir2"; then
        log "✓ Task 2 status shows correct working directory"
    else
        error "✗ Task 2 status doesn't show working directory. Output: $status_output2"
    fi
    
    # Step 7: Test error handling with non-existent directory
    log "Step 7: Testing error handling with non-existent directory..."
    local nonexistent_dir="/tmp/ghost_nonexistent_dir_12345"
    local error_output
    
    if error_output=$($GHOST_BIN run --cwd "$nonexistent_dir" echo "test" 2>&1); then
        warn "Expected error for non-existent directory, but command succeeded: $error_output"
    else
        log "✓ Correctly handled non-existent directory error"
    fi
    
    log "CWD functionality E2E test completed successfully!"
}

# Main execution
main() {
    log "=== Ghost CWD Functionality E2E Test ==="
    log "Log file: $LOG_FILE"
    
    test_prerequisites
    test_cwd_functionality
    
    log "=== All tests passed! ==="
}

# Run the test
main "$@"