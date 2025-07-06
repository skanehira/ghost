use ratatui::widgets::TableState;

/// Wrapper for managing table scrolling with TableState
#[derive(Debug, Default)]
pub struct TableScroll {
    state: TableState,
    total_items: usize,
}

impl TableScroll {
    pub fn new() -> Self {
        let mut state = TableState::default();
        state.select(Some(0));
        Self {
            state,
            total_items: 0,
        }
    }

    pub fn with_items(total_items: usize) -> Self {
        let mut state = TableState::default();
        if total_items > 0 {
            state.select(Some(0));
        }
        Self { state, total_items }
    }

    pub fn state_mut(&mut self) -> &mut TableState {
        &mut self.state
    }

    pub fn selected(&self) -> Option<usize> {
        self.state.selected()
    }

    pub fn select(&mut self, index: Option<usize>) {
        self.state.select(index);
    }

    pub fn next(&mut self) {
        if self.total_items == 0 {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.total_items - 1 {
                    i // Stay at the last item
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.total_items == 0 {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    0 // Stay at the first item
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn first(&mut self) {
        if self.total_items > 0 {
            self.state.select(Some(0));
        }
    }

    pub fn last(&mut self) {
        if self.total_items > 0 {
            self.state.select(Some(self.total_items - 1));
        }
    }

    pub fn set_total_items(&mut self, total: usize) {
        self.total_items = total;
        // Adjust selection if current selection is out of bounds
        if let Some(selected) = self.state.selected() {
            if selected >= total && total > 0 {
                self.state.select(Some(total - 1));
            } else if total == 0 {
                self.state.select(None);
            }
        } else if total > 0 {
            self.state.select(Some(0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_table_scroll() {
        let scroll = TableScroll::new();
        assert_eq!(scroll.selected(), Some(0));
        assert_eq!(scroll.total_items, 0);
    }

    #[test]
    fn test_with_items() {
        let scroll = TableScroll::with_items(5);
        assert_eq!(scroll.selected(), Some(0));
        assert_eq!(scroll.total_items, 5);
    }

    #[test]
    fn test_with_zero_items() {
        let scroll = TableScroll::with_items(0);
        assert_eq!(scroll.selected(), None);
        assert_eq!(scroll.total_items, 0);
    }

    #[test]
    fn test_next_navigation() {
        let mut scroll = TableScroll::with_items(3);

        // Start at 0
        assert_eq!(scroll.selected(), Some(0));

        // Move to 1
        scroll.next();
        assert_eq!(scroll.selected(), Some(1));

        // Move to 2
        scroll.next();
        assert_eq!(scroll.selected(), Some(2));

        // Stay at 2 (no wrap)
        scroll.next();
        assert_eq!(scroll.selected(), Some(2));
        
        // Confirm it stays at 2
        scroll.next();
        assert_eq!(scroll.selected(), Some(2));
    }

    #[test]
    fn test_previous_navigation() {
        let mut scroll = TableScroll::with_items(3);

        // Start at 0
        assert_eq!(scroll.selected(), Some(0));

        // Stay at 0 (no wrap)
        scroll.previous();
        assert_eq!(scroll.selected(), Some(0));
        
        // Move to 2
        scroll.select(Some(2));
        assert_eq!(scroll.selected(), Some(2));

        // Move to 1
        scroll.previous();
        assert_eq!(scroll.selected(), Some(1));

        // Move to 0
        scroll.previous();
        assert_eq!(scroll.selected(), Some(0));
        
        // Stay at 0
        scroll.previous();
        assert_eq!(scroll.selected(), Some(0));
    }

    #[test]
    fn test_first_last() {
        let mut scroll = TableScroll::with_items(5);

        // Move to middle
        scroll.select(Some(2));
        assert_eq!(scroll.selected(), Some(2));

        // Go to first
        scroll.first();
        assert_eq!(scroll.selected(), Some(0));

        // Go to last
        scroll.last();
        assert_eq!(scroll.selected(), Some(4));
    }

    #[test]
    fn test_empty_table_navigation() {
        let mut scroll = TableScroll::with_items(0);

        // All navigation should do nothing
        scroll.next();
        assert_eq!(scroll.selected(), None);

        scroll.previous();
        assert_eq!(scroll.selected(), None);

        scroll.first();
        assert_eq!(scroll.selected(), None);

        scroll.last();
        assert_eq!(scroll.selected(), None);
    }

    #[test]
    fn test_set_total_items() {
        let mut scroll = TableScroll::with_items(5);
        scroll.select(Some(3));

        // Reduce items - selection should adjust
        scroll.set_total_items(2);
        assert_eq!(scroll.selected(), Some(1));
        assert_eq!(scroll.total_items, 2);

        // Set to zero - selection should be None
        scroll.set_total_items(0);
        assert_eq!(scroll.selected(), None);
        assert_eq!(scroll.total_items, 0);

        // Increase from zero - selection should be Some(0)
        scroll.set_total_items(3);
        assert_eq!(scroll.selected(), Some(0));
        assert_eq!(scroll.total_items, 3);
    }

    #[test]
    fn test_single_item_navigation() {
        let mut scroll = TableScroll::with_items(1);

        // Should stay at 0
        assert_eq!(scroll.selected(), Some(0));

        // Next should stay at 0
        scroll.next();
        assert_eq!(scroll.selected(), Some(0));

        // Previous should stay at 0
        scroll.previous();
        assert_eq!(scroll.selected(), Some(0));
    }
}
