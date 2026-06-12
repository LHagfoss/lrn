pub mod components;
pub mod utils;

use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

pub fn render(frame: &mut Frame, app: &App) {
    let screen_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)])
        .split(frame.area());

    let workspace_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(screen_chunks[0]);

    components::draw_sidebar(frame, app, workspace_chunks[0]);
    components::draw_viewer(frame, app, workspace_chunks[1]);
    components::draw_footer(frame, app, screen_chunks[1]);
    components::draw_modal(frame, app);
}
