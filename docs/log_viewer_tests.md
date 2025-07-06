# Log Viewer Test Documentation

## Overview
This document outlines the existing log viewer tests to ensure compatibility when migrating to tui-scrollview.

## Existing Test Cases

### 1. `test_log_viewer_display`
- **Purpose**: Tests basic log viewer rendering
- **Verifies**: 
  - Header contains "Log Viewer" and task ID
  - Footer contains keybinds: "j/k:Scroll", "gg/G:Top/Bottom", "Esc:Back"

### 2. `test_log_view_key_handling`
- **Purpose**: Tests keyboard navigation in log view
- **Verifies**:
  - 'l' key switches to log view mode
  - 'j' scrolls down
  - 'k' scrolls up
  - 'g' goes to top (offset = 0)
  - 'G' goes to bottom
  - 'Esc' returns to task list

### 3. `test_log_viewer_updates_on_file_change`
- **Purpose**: Tests dynamic file updates
- **Verifies**:
  - Initial lines are loaded correctly
  - File changes are detected on re-render
  - Line count updates when new lines are added
  - Footer shows correct line count

### 4. `test_log_viewer_caches_file_content`
- **Purpose**: Tests caching performance with large files
- **Verifies**:
  - First render reads file (slower)
  - Second render uses cache (faster)
  - Cache significantly improves performance

### 5. `test_log_viewer_reloads_on_file_modification`
- **Purpose**: Tests file modification detection
- **Verifies**:
  - File modification time changes trigger reload
  - Line count updates after modification

### 6. `test_log_viewer_only_processes_visible_lines`
- **Purpose**: Tests rendering performance optimization
- **Verifies**:
  - Only visible lines are processed (not all 100k lines)
  - Render time is fast (<50ms) even with huge files
  - Buffer contains only visible content

### 7. `test_log_viewer_memory_limit`
- **Purpose**: Tests memory constraints
- **Verifies**:
  - Maximum 10,000 lines loaded into memory
  - Can still scroll near the end
  - Prevents memory issues with large files

### 8. `test_log_viewer_auto_updates_on_file_change`
- **Purpose**: Tests background file updates
- **Verifies**:
  - File changes made in background are detected
  - Line count updates automatically

### 9. `test_log_viewer_incremental_update`
- **Purpose**: Tests incremental loading
- **Verifies**:
  - Only new lines are loaded when file grows
  - Existing content is preserved
  - New lines are visible when scrolled to bottom

## Key Requirements for tui-scrollview Implementation

1. **Memory Limit**: MAX_LINES_IN_MEMORY = 10,000
2. **Performance**: Must handle 100k+ line files efficiently
3. **Incremental Updates**: Only load new content when file grows
4. **Caching**: Maintain cache for performance
5. **Navigation**: Support j/k, gg/G keyboard shortcuts
6. **Line Numbers**: Display line numbers (future enhancement)
7. **Header/Footer**: Maintain current header/footer format