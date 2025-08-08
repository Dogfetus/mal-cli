use ratatui::layout::{Constraint, Direction, Layout, Rect};
#[derive(Debug, Clone)]
pub struct Navigatable {
    rows: u16,
    cols: u16,

    total_items: usize,
    reverse: bool,

    selected: usize,
    scroll: usize,
}

impl Navigatable{

    /// size: (rows, cols)
    pub fn new(size: (u16, u16)) -> Self {
        Self {
            reverse: false,
            total_items: 0,
            rows: size.0,
            cols: size.1,
            selected: 0,
            scroll: 0,
        }
    }

    pub fn back_to_start(&mut self) {
        self.selected = 0;
        self.scroll = 0;
    }

    pub fn as_reverse(&mut self) -> &mut Self {
        self.reverse = true;
        self
    }

    pub fn in_reverse(&self) -> bool {
        self.reverse
    }

    pub fn visible_elements(&self) -> usize {
        (self.rows * self.cols) as usize
    }

    pub fn update_scroll(&mut self) {
        let visible_count = self.visible_elements();
        if self.selected < self.scroll {
            self.scroll = self.scroll.saturating_sub(self.cols as usize);
        } 

        else if self.selected >= (self.scroll + visible_count) {
            self.scroll = self.scroll.saturating_add(self.cols as usize); 
        }
    }

    pub fn change_size(&mut self, size: (u16, u16)) {
        self.rows = size.0;
        self.cols = size.1;
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(self.cols as usize);
        self.update_scroll();
    }

    pub fn move_left(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.update_scroll();
    }

    pub fn move_down(&mut self) {
        if (self.selected + self.cols as usize) < self.total_items.saturating_sub(1) {
            self.selected = self.selected.saturating_add(self.cols as usize);
        }
        else {
            self.selected = self.total_items.saturating_sub(1);
        }
        self.update_scroll();
    }

    pub fn move_right(&mut self) {
        if self.selected < self.total_items.saturating_sub(1) {
            self.selected = self.selected.saturating_add(1);
        }
        self.update_scroll();
    }

    pub fn visible_indices(&self) -> std::ops::Range<usize> {
        let visible_count = self.visible_elements();
        let start = self.scroll;
        let end = (start + visible_count).min(self.total_items);
        start..end
    }

    pub fn get_visible_items<'a, T>(&'a self, items: &'a [T]) -> &'a [T] {
        let range = self.visible_indices();
        let start = range.start.min(items.len());
        let end = range.end.min(items.len());
        &items[start..end]
    }

    pub fn get_visible_items_mut<'a, T>(&'a mut self, items: &'a mut [T]) -> &'a mut [T] {
        let range = self.visible_indices();
        let start = range.start.min(items.len());
        let end = range.end.min(items.len());
        &mut items[start..end]
    }

    fn create_balanced_constraints(&self, count: u16) -> Vec<Constraint> {
        let base_percentage = 100 / count;
        let remainder = 100 % count;
        let mut constraints = Vec::new();
        for i in 0..count {
            let percentage = if i == count - 1 {
                // Give the remainder to the last constraint
                base_percentage + remainder
            } else {
                base_percentage
            };
            constraints.push(Constraint::Percentage(percentage));
        }
        constraints
    }

    fn create_grid(&self, area: Rect) -> Vec<Vec<Rect>> {
        let row_constraints = self.create_balanced_constraints(self.rows);
        let col_constraints = self.create_balanced_constraints(self.cols);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(row_constraints)
            .split(area);

        let mut grid = Vec::new();
        for row in rows.iter() {
            let cols = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(col_constraints.clone())
                .split(*row);
            grid.push(cols.to_vec());
        }
        grid
    }

    pub fn get_selected_item<'a, T>(&'a self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            return None;
        }
        let index = self.selected.min(items.len() - 1);
        items.get(index)
    }

    pub fn get_item_at_index<'a, T>(&'a self, items: &'a [T], index: usize) -> Option<&'a T> {
        if items.is_empty() || index >= items.len() {
            return None;
        }
        items.get(index)
    }

    pub fn get_item_at_index_mut<'a, T>(&'a mut self, items: &'a mut [T], index: usize) -> Option<&'a mut T> {
        if items.is_empty() || index >= items.len() {
            return None;
        }
        items.get_mut(index)
    }

    pub fn get_selected_item_mut<'a, T>(&'a mut self, items: &'a mut [T]) -> Option<&'a mut T> {
        if items.is_empty() {
            return None;
        }
        let index = self.selected.min(items.len() - 1);
        items.get_mut(index)
    }

    pub fn get_selected_item_mut_and_index<'a, T>(&'a mut self, items: &'a mut [T]) -> Option<(&'a mut T, usize)> {
        if items.is_empty() {
            return None;
        }
        let index = self.selected.min(items.len() - 1);
        items.get_mut(index).map(|item| (item, index))
    }

    pub fn get_selected_index(&self) -> usize {
        self.selected
    }

    pub fn construct<T, F>(&mut self, items: &[T], area: Rect, mut callback: F)
    where
        F: FnMut(&T, Rect, bool),
    {
        if items.is_empty() {
            return;
        }

        self.total_items = items.len();

        let grid = self.create_grid(area);

        if self.reverse {
            for (visible_idx, absolute_idx) in self.visible_indices().enumerate().rev() {
                let row = visible_idx / self.cols as usize;
                let col = visible_idx % self.cols as usize;

                if row < grid.len() && col < grid[row].len() {
                    let item = &items[absolute_idx];
                    let is_selected = absolute_idx == self.selected;
                    callback(item, grid[row][col], is_selected);
                }
            }
        }

        else {
            for (visible_idx, absolute_idx) in self.visible_indices().enumerate() {
                let row = visible_idx / self.cols as usize;
                let col = visible_idx % self.cols as usize;

                if row < grid.len() && col < grid[row].len() {
                    let item = &items[absolute_idx];
                    let is_selected = absolute_idx == self.selected;
                    callback(item, grid[row][col], is_selected);
                }
            }
        }
    }

    pub fn construct_mut<T, F>(&mut self, items: &mut [T], area: Rect, mut callback: F)
    where
        F: FnMut(&mut T, Rect, bool),
    {
        if items.is_empty() {
            return;
        }

        self.total_items = items.len();

        let grid = self.create_grid(area);

        if self.reverse {
            for (visible_idx, absolute_idx) in self.visible_indices().enumerate().rev() {
                let row = visible_idx / self.cols as usize;
                let col = visible_idx % self.cols as usize;

                if row < grid.len() && col < grid[row].len() {
                    let item = &mut items[absolute_idx];
                    let is_selected = absolute_idx == self.selected;
                    callback(item, grid[row][col], is_selected);
                }
            }
        }

        else {
            for (visible_idx, absolute_idx) in self.visible_indices().enumerate() {
                let row = visible_idx / self.cols as usize;
                let col = visible_idx % self.cols as usize;

                if row < grid.len() && col < grid[row].len() {
                    let item = &mut items[absolute_idx];
                    let is_selected = absolute_idx == self.selected;
                    callback(item, grid[row][col], is_selected);
                }
            }
        }
    }
}
