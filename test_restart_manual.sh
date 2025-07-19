#!/bin/bash
# Manual test script for restart functionality

echo "=== Ghost Restart Test ==="
echo "Testing that commands are not double-wrapped on restart..."

# Clean up any existing test processes
ghost list | grep "test_restart_script.sh" | awk '{print $1}' | xargs -I {} ghost stop {} -k 2>/dev/null

# Create a test script that shows its command line
cat > /tmp/test_restart_script.sh << 'EOF'
#!/bin/bash
echo "Process started with command: $0 $@"
echo "Working directory: $(pwd)"
echo "TEST_ENV variable: $TEST_ENV"
while true; do
    echo "Running at $(date)"
    sleep 2
done
EOF

chmod +x /tmp/test_restart_script.sh

echo ""
echo "1. Spawning test process..."
cd /tmp
ghost spawn "/tmp/test_restart_script.sh" --env "TEST_ENV=original_value"

# Get the task ID
sleep 1
TASK_ID=$(ghost list | grep "test_restart_script.sh" | head -1 | awk '{print $1}')

if [ -z "$TASK_ID" ]; then
    echo "ERROR: Failed to spawn test process"
    exit 1
fi

echo "   Task ID: $TASK_ID"

# Check the log
sleep 1
echo ""
echo "2. Initial log output:"
ghost log "$TASK_ID" | head -5

# Get the stored command from database
echo ""
echo "3. Checking database for stored command..."
# This would require direct database access, so we'll check via restart

echo ""
echo "4. Now stopping the process..."
ghost stop "$TASK_ID"

sleep 1

echo ""
echo "5. Restarting the process using TUI 'r' command simulation..."
echo "   (In real usage, press 'r' in ghost tui to restart)"

# Since we can't easily simulate TUI interaction, we'll test by re-spawning
# In the fixed version, this should preserve the original command
echo ""
echo "6. For manual test:"
echo "   - Run: ghost tui"
echo "   - Select the stopped task"  
echo "   - Press 'r' to restart"
echo "   - Check if the command is not double-wrapped"

# Clean up
rm -f /tmp/test_restart_script.sh

echo ""
echo "=== Test Complete ==="
echo "If the restart shows '/bin/zsh -lc /bin/zsh -lc ...', the bug is NOT fixed."
echo "If the restart shows only one '/bin/zsh -lc ...', the bug IS fixed."