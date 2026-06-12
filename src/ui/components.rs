use crate::app::{ActivePanel, App};
use crate::ui::utils::centered_rect;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};

pub fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.active_panel == ActivePanel::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(i, f)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(f.as_str()).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Files ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let file_list = List::new(items).block(block);
    frame.render_widget(file_list, area);
}

pub fn draw_viewer(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.is_editing {
        Style::default().fg(Color::Yellow)
    } else if app.active_panel == ActivePanel::Viewer {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = if app.is_editing {
        format!(" Editing: {} ", app.current_file)
    } else {
        format!(" {} ", app.current_file)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let paragraph = Paragraph::new(format!(
        "{}{}",
        app.current_file_content,
        if app.is_editing { "_" } else { "" }
    ))
    .block(block);
    frame.render_widget(paragraph, area);
}

pub fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.is_editing {
        " Esc: Stop Editing & Save "
    } else {
        " q: Quit | Tab: Switch Pane | ↑↓: Navigate | r: Rename | e: Edit | n: New Note "
    };
    frame.render_widget(Paragraph::new(help_text), area);
}

pub fn draw_modal(frame: &mut Frame, app: &App) {
    if app.show_new_note_modal || app.show_rename_modal {
        let modal_area = centered_rect(35, 3, frame.area());
        frame.render_widget(Clear, modal_area);

        let title = if app.show_rename_modal {
            " Rename file: "
        } else {
            " Name of new file: "
        };

        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow));

        // Slice string buffer to slot cursor indicator directly at index location position
        let prefix = &app.new_note_input[..app.cursor_position];
        let suffix = &app.new_note_input[app.cursor_position..];
        let display_text = format!("{}|{}", prefix, suffix);

        let paragraph = Paragraph::new(display_text).block(block);
        frame.render_widget(paragraph, modal_area);
    }
}
