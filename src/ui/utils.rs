use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Helper to pad an area
pub fn pad_rect(area: Rect, left: u16, right: u16, top: u16, bottom: u16) -> Rect {
    let new_x = area.x.saturating_add(left);
    let new_y = area.y.saturating_add(top);
    let new_width = area.width.saturating_sub(left + right);
    let new_height = area.height.saturating_sub(top + bottom);
    
    Rect {
        x: new_x,
        y: new_y,
        width: new_width,
        height: new_height,
    }
}

/// Helper to create a centered rect
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
