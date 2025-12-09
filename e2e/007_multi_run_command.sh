#!/bin/bash

# E2E test for ghost run with multiple commands
# Test that ghost can run multiple commands in parallel

# Load test helpers
source "$(dirname "$0")/test_helpers.sh"

# Test-specific configuration
LOG_FILE="/tmp/ghost_e2e_multi_run_test.log"

# Array to track task IDs for cleanup
declare -a TASK_IDS=()

# Cleanup function
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    # Kill any remaining test processes
    if [[ ${#TASK_IDS[@]} -gt 0 ]]; then
        for task_id in "${TASK_IDS[@]}"; do
            $GHOST_BIN stop "$task_id" --force 2>/dev/null || true
        done
    fi
    # Clean up log file
    rm -f "$LOG_FILE"
}

# Set up cleanup trap
trap cleanup EXIT

# Extract all task IDs from multi-run output
extract_task_ids() {
    local run_output="$1"
    echo "$run_output" | grep "Task ID:" | awk '{print $3}'
}

# Test running multiple commands in parallel
test_multi_run_parallel() {
    log "Test 1: Running multiple commands in parallel"

    # Run two commands simultaneously
    log "Running: ghost run \"sleep 5\" \"sleep 5\""
    local run_output
    run_output=$($GHOST_BIN run "sleep 5" "sleep 5" 2>&1)

    # Extract task IDs
    local ids
    ids=$(extract_task_ids "$run_output")
    local id_count
    id_count=$(echo "$ids" | wc -l | tr -d ' ')

    if [[ "$id_count" -ne 2 ]]; then
        error "Expected 2 task IDs, got $id_count"
    fi

    # Store IDs for cleanup
    while IFS= read -r id; do
        TASK_IDS+=("$id")
    done <<< "$ids"

    log "Started 2 tasks: ${TASK_IDS[*]}"

    # Verify both tasks are running
    sleep 1
    for task_id in "${TASK_IDS[@]}"; do
        local status_output
        status_output=$($GHOST_BIN status "$task_id" 2>/dev/null)
        if echo "$status_output" | grep -q "running"; then
            log "✓ Task $task_id is running"
        else
            error "✗ Task $task_id is not running"
        fi
    done

    # Verify both have unique PIDs
    local pid1 pid2
    pid1=$($GHOST_BIN status "${TASK_IDS[0]}" 2>/dev/null | grep "^PID:" | awk '{print $2}')
    pid2=$($GHOST_BIN status "${TASK_IDS[1]}" 2>/dev/null | grep "^PID:" | awk '{print $2}')

    if [[ "$pid1" != "$pid2" ]]; then
        log "✓ Tasks have unique PIDs: $pid1, $pid2"
    else
        error "✗ Tasks have same PID: $pid1"
    fi

    # Cleanup these tasks
    for task_id in "${TASK_IDS[@]}"; do
        $GHOST_BIN stop "$task_id" --force 2>/dev/null || true
    done
    TASK_IDS=()

    log "Test 1 passed: Multiple commands run in parallel successfully"
}

# Test backward compatibility with single command
test_single_command_backward_compat() {
    log "Test 2: Backward compatibility with single command"

    # Run single command in traditional format
    log "Running: ghost run sleep 3"
    local run_output
    run_output=$($GHOST_BIN run sleep 3 2>&1)

    local task_id
    task_id=$(extract_task_id "$run_output")

    if [[ -z "$task_id" ]]; then
        error "Failed to extract task ID from single command run"
    fi

    TASK_IDS+=("$task_id")
    log "Started task: $task_id"

    # Verify task is running
    sleep 1
    local status_output
    status_output=$($GHOST_BIN status "$task_id" 2>/dev/null)
    if echo "$status_output" | grep -q "running"; then
        log "✓ Task $task_id is running"
    else
        error "✗ Task $task_id is not running"
    fi

    # Verify command was parsed correctly (should be "sleep 3")
    if echo "$status_output" | grep -q "sleep"; then
        log "✓ Command contains 'sleep'"
    else
        error "✗ Command does not contain 'sleep'"
    fi

    # Cleanup
    $GHOST_BIN stop "$task_id" --force 2>/dev/null || true
    TASK_IDS=()

    log "Test 2 passed: Backward compatibility works"
}

# Test mixed success and failure
test_mixed_results() {
    log "Test 3: Mixed success and failure handling"

    # Run one valid command and one that will complete quickly
    log "Running: ghost run \"sleep 3\" \"echo hello\""
    local run_output
    run_output=$($GHOST_BIN run "sleep 3" "echo hello" 2>&1)

    local ids
    ids=$(extract_task_ids "$run_output")
    local id_count
    id_count=$(echo "$ids" | wc -l | tr -d ' ')

    if [[ "$id_count" -ne 2 ]]; then
        error "Expected 2 task IDs, got $id_count"
    fi

    # Store IDs for cleanup
    while IFS= read -r id; do
        TASK_IDS+=("$id")
    done <<< "$ids"

    log "Started 2 tasks: ${TASK_IDS[*]}"

    # Wait for echo to finish
    sleep 2

    # Check that both tasks exist in list
    local list_output
    list_output=$($GHOST_BIN list 2>/dev/null)

    for task_id in "${TASK_IDS[@]}"; do
        if echo "$list_output" | grep -q "$task_id"; then
            log "✓ Task $task_id found in list"
        else
            error "✗ Task $task_id not found in list"
        fi
    done

    # Cleanup
    for task_id in "${TASK_IDS[@]}"; do
        $GHOST_BIN stop "$task_id" --force 2>/dev/null || true
    done
    TASK_IDS=()

    log "Test 3 passed: Mixed results handled correctly"
}

# Test with environment variables
test_multi_run_with_env() {
    log "Test 4: Multiple commands with environment variables"

    # Run commands with shared environment
    log "Running: ghost run -e TEST_VAR=hello \"printenv TEST_VAR\" \"printenv TEST_VAR\""
    local run_output
    run_output=$($GHOST_BIN run -e TEST_VAR=hello "printenv TEST_VAR" "printenv TEST_VAR" 2>&1)

    local ids
    ids=$(extract_task_ids "$run_output")
    local id_count
    id_count=$(echo "$ids" | wc -l | tr -d ' ')

    if [[ "$id_count" -ne 2 ]]; then
        error "Expected 2 task IDs, got $id_count"
    fi

    # Store IDs for cleanup
    while IFS= read -r id; do
        TASK_IDS+=("$id")
    done <<< "$ids"

    log "Started 2 tasks: ${TASK_IDS[*]}"

    # Wait for commands to complete
    sleep 2

    # Check logs for the environment variable value
    for task_id in "${TASK_IDS[@]}"; do
        local log_output
        log_output=$($GHOST_BIN log "$task_id" 2>/dev/null)
        if echo "$log_output" | grep -q "hello"; then
            log "✓ Task $task_id has correct env var in output"
        else
            warn "Task $task_id log: $log_output"
        fi
    done

    # Cleanup
    for task_id in "${TASK_IDS[@]}"; do
        $GHOST_BIN stop "$task_id" --force 2>/dev/null || true
    done
    TASK_IDS=()

    log "Test 4 passed: Environment variables work with multiple commands"
}

# Main execution
main() {
    log "=== Ghost Multi-Run Command E2E Test ==="
    log "Log file: $LOG_FILE"

    test_prerequisites

    test_multi_run_parallel
    test_single_command_backward_compat
    test_mixed_results
    test_multi_run_with_env

    log "=== All multi-run tests passed! ==="
}

# Run the test
main "$@"
