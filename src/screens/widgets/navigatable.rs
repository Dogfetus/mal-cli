use ratatui::layout::{Constraint, Direction, Layout, Rect};
#[derive(Debug, Clone)]
pub struct Navigatable {
    rows: u16,
    cols: u16,

    selected: usize,
    scroll: usize,
}

impl Navigatable{
    pub fn new(size: (u16, u16)) -> Self {
        Self {
            rows: size.0,
            cols: size.1,
            selected: 0,
            scroll: 0,
        }
    }
    pub fn visable_elements(&self) -> usize {
        (self.rows * self.cols) as usize
    }

    pub fn update_scroll(&mut self) {
        let visible_count = self.visable_elements();
        if self.selected < self.scroll {
            self.scroll = self.selected.saturating_sub(self.cols as usize);
        } 

        else if self.selected >= (self.scroll + visible_count) {
            self.scroll = self.selected.saturating_add(self.cols as usize); 
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

    pub fn move_down(&mut self, total_items: usize) {
        if (self.selected + self.cols as usize) < total_items.saturating_sub(1) {
            self.selected = self.selected.saturating_add(self.cols as usize);
        }
        else {
            self.selected = total_items.saturating_sub(1);
        }
        self.update_scroll();
    }

    pub fn move_right(&mut self, total_items: usize) {
        if self.selected < total_items.saturating_sub(1) {
            self.selected = self.selected.saturating_add(1);
        }
        self.update_scroll();
    }

    pub fn visible_indices(&self, total_items: usize) -> impl Iterator<Item = usize> {
        let visible_count = self.visable_elements();
        let start = self.scroll;
        let end = (start + visible_count).min(total_items);
        start..end
    }

    fn create_grid(&self, area: Rect) -> Vec<Vec<Rect>> {
        let row_constraints: Vec<Constraint> =
            vec![Constraint::Percentage(100 / self.rows); self.rows as usize];
        let col_constraints: Vec<Constraint> =
            vec![Constraint::Percentage(100 / self.cols); self.cols as usize];

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

    pub fn construct<T, F>(&self, items: &[T], area: Rect, mut callback: F)
    where
        F: FnMut(&T, Rect, bool),
    {
        if items.is_empty() {
            return;
        }

        let grid = self.create_grid(area);
        let visible_items = self.visible_indices(items.len()).enumerate();
        for (visible_idx, absolute_idx) in visible_items {
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
