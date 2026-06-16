use crate::app::{ActivePanel, App};
use crate::ui::utils::centered_rect;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph},
};

pub fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.show_sidebar_search {
        Style::default().fg(Color::Yellow)
    } else if app.active_panel == ActivePanel::Sidebar {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let filtered_files: Vec<&String> = app
        .files
        .iter()
        .filter(|f| {
            f.to_lowercase()
                .contains(&app.sidebar_search_query.to_lowercase())
        })
        .collect();

    let items: Vec<ListItem> = filtered_files
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

    let title = if !app.sidebar_search_query.is_empty() {
        format!(" Files (Filter: {}) ", app.sidebar_search_query)
    } else if app.show_sidebar_search {
        String::from(" Search Filename: ")
    } else {
        String::from(" Files ")
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let file_list = List::new(items).block(block);
    frame.render_widget(file_list, area);
}

pub fn draw_viewer(frame: &mut Frame, app: &App, area: Rect) {
    let border_style = if app.active_panel == ActivePanel::Viewer {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let title = format!(" {} ", app.current_file);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let mut widget = app.text_editor.clone();
    widget.set_block(block);

    // Always show cursor — viewer is always interactive for search/navigation
    widget.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(&widget, area);
}

pub fn draw_search_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Search Text Inside Document: ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow));

    let mut widget = app.search_input.clone();
    widget.set_block(block);
    widget.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

    frame.render_widget(&widget, area);
}

pub fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = if app.show_sidebar_search {
        String::from(" Type filenames to filter | Enter/Esc: Done | ↑↓: Navigate ")
    } else if app.show_search_bar {
        String::from(" Search text in document | Enter/Esc: Close ")
    } else if app.show_delete_modal {
        let sel = match app.delete_selection {
            0 => "[ Cancel ]",
            1 => " [ Delete ] ",
            _ => "",
        };
        format!(" Confirm deletion {} ", sel)
    } else if app.active_panel == ActivePanel::Sidebar {
        String::from(" ↑↓: Navigate | e/i: Edit in nvim | Tab: Viewer | r: Rename | n: New Note | d: Delete | /: Filter | f: Search ")
    } else {
        String::from(" ↑↓: Navigate | Tab: Sidebar | Ctrl+O: Follow Link | f: Search | c: Clear ")
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

        let mut widget = app.modal_input.clone();
        widget.set_block(block);
        widget.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));

        frame.render_widget(&widget, modal_area);
    }
}

pub fn draw_delete_modal(frame: &mut Frame, app: &App) {
    if !app.show_delete_modal {
        return;
    }

    // Small centered modal
    let area = centered_rect(42, 3, frame.area());
    frame.render_widget(Clear, area);

    let sel = app.delete_selection;
    let block = Block::default()
        .title(" Delete file ")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Red));

    // Styled buttons on one line: [ Cancel ]  |  [ Delete ]
    let left = if sel == 0 { format!(" [ {} ] ", "Cancel") } else { format!("   {}   ", "Cancel") };
    let right = if sel == 1 { format!(" [ {} ] ", "Delete") } else { format!("   {}   ", "Delete") };

    // Create styled line
    use ratatui::text::{Line, Span};
    let line = Line::from(vec![
        Span::styled(left, Style::default().fg(if sel == 0 { Color::Cyan } else { Color::Gray })),
        Span::raw(" | "),
        Span::styled(right, Style::default().fg(if sel == 1 { Color::Red } else { Color::Gray })),
    ]);

    let msg = Paragraph::new(line).block(block);
    frame.render_widget(msg, area);
}
