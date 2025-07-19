#!/bin/bash
# Manual test script for auto-scroll functionality

echo "=== Ghost Auto-Scroll Test ==="
echo "Testing the tail -f like auto-scroll feature..."

# Create a test script that generates continuous output
cat > /tmp/test_continuous_log.sh << 'EOF'
#!/bin/bash
counter=1
while true; do
    echo "Log line $counter - $(date '+%Y-%m-%d %H:%M:%S')"
    counter=$((counter + 1))
    sleep 1
done
EOF

chmod +x /tmp/test_continuous_log.sh

echo ""
echo "1. Spawning a process that generates continuous log output..."
ghost spawn "/tmp/test_continuous_log.sh"

sleep 1
TASK_ID=$(ghost list | grep "test_continuous_log.sh" | head -1 | awk '{print $1}')

if [ -z "$TASK_ID" ]; then
    echo "ERROR: Failed to spawn test process"
    exit 1
fi

echo "   Task ID: $TASK_ID"
echo ""
echo "2. Instructions to test auto-scroll:"
echo "   - Run: ghost tui"
echo "   - Select the test_continuous_log.sh task"
echo "   - Press Enter to view logs"
echo "   - Press 'f' to enable auto-scroll (you should see [Auto-Scroll] indicator)"
echo "   - Watch as new log lines are automatically scrolled into view"
echo "   - Press 'j' or 'k' to manually scroll (auto-scroll will be disabled)"
echo "   - Press 'f' again to re-enable auto-scroll"
echo "   - Press 'q' to exit log view"
echo ""
echo "3. When done testing, stop the process with:"
echo "   ghost stop $TASK_ID"
echo ""
echo "=== Test Setup Complete ==="