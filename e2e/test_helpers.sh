#!/bin/bash

# Test helpers and utilities for Ghost E2E tests

# Exit on any error, undefined variables, and pipe failures
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration paths
GHOST_BIN="${GHOST_BINARY:-./target/release/ghost}"
TEST_SCRIPT="./scripts/hello_loop.sh"
MULTILINE_TEST_SCRIPT="./scripts/multiline_test.sh"

# Ensure we're in the correct directory (project root)
if [[ ! -f "$GHOST_BIN" && ! -f "./target/release/ghost" && ! -f "Cargo.toml" ]]; then
    echo "Error: Must be run from project root directory" >&2
    exit 1
fi

# Build release binary if it doesn't exist
ensure_ghost_binary() {
    if [[ ! -f "$GHOST_BIN" ]]; then
        echo "Building release binary..."
        cargo build --release
    fi
}

# Common helper functions

# Logging functions
log() {
    echo -e "${GREEN}[TEST]${NC} $1" | tee -a "$LOG_FILE"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1" | tee -a "$LOG_FILE"
    exit 1
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

# Extract task ID from ghost run output
extract_task_id() {
    local run_output="$1"
    echo "$run_output" | grep "Task ID:" | awk '{print $3}'
}

# Test prerequisites check
test_prerequisites() {
    log "Checking prerequisites..."
    
    if [[ ! -f "$GHOST_BIN" ]]; then
        error "Ghost binary not found at $GHOST_BIN. Run 'cargo build --release' first."
    fi
    
    if [[ ! -f "$TEST_SCRIPT" ]]; then
        error "Test script not found at $TEST_SCRIPT"
    fi
    
    if [[ ! -x "$TEST_SCRIPT" ]]; then
        chmod +x "$TEST_SCRIPT"
        log "Made test script executable"
    fi
    
    # Check multiline test script if it exists
    if [[ -f "$MULTILINE_TEST_SCRIPT" && ! -x "$MULTILINE_TEST_SCRIPT" ]]; then
        chmod +x "$MULTILINE_TEST_SCRIPT"
        log "Made multiline test script executable"
    fi
    
    log "Prerequisites check passed"
}