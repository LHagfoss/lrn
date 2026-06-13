use crate::app::{ActivePanel, App};
use crate::ui::utils::centered_rect;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
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

    let mut widget = app.text_editor.clone();
    widget.set_block(block);

    if !app.is_editing {
        widget.set_cursor_style(Style::default());
    } else {
        widget.set_cursor_style(Style::default().add_modifier(Modifier::REVERSED));
    }

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
    let help_text = if app.is_editing {
        " Esc: Stop Editing & Save "
    } else if app.show_sidebar_search {
        " Type character sequences to drop sidebar mismatch entries | Enter/Esc: Exit Search "
    } else if app.show_search_bar {
        " Type text string to jump highlight positions inside note content | Enter/Esc: Close "
    } else {
        " q: Quit | Tab: Switch Pane | ↑↓: Navigate | r: Rename | e: Edit | n: New Note | /: Filter Files | f: Search Text | c: Clear Filters "
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

pub fn draw_autocomplete_modal(frame: &mut Frame, app: &App, editor_area: Rect) {
    if !app.show_autocomplete {
        return;
    }

    let filtered_files: Vec<&String> = app
        .files
        .iter()
        .filter(|f| {
            f.to_lowercase()
                .contains(&app.autocomplete_query.to_lowercase())
        })
        .collect();

    let modal_area = centered_rect(40, 25, editor_area);
    frame.render_widget(Clear, modal_area);

    let block = Block::default()
        .title(format!(
            " Link Note (Filtering: '{}') ",
            app.autocomplete_query
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta));

    let items: Vec<ListItem> = filtered_files
        .iter()
        .map(|name| {
            let clean_name = name.strip_suffix(".md").unwrap_or(name);
            ListItem::new(format!("   {}", clean_name))
        })
        .collect();

    let mut list_state = ListState::default();
    if !filtered_files.is_empty() {
        list_state.select(Some(app.autocomplete_index));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(Color::Magenta)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    frame.render_stateful_widget(list, modal_area, &mut list_state);
}
