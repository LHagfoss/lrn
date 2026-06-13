use crate::app::{ActivePanel, App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Style};
use std::fs;
use tui_textarea::{Input, Key, TextArea};

pub fn handle_key_input(app: &mut App, key: KeyEvent) {
    let input_event = match key.code {
        KeyCode::Char(c) => {
            let mut mods = KeyModifiers::empty();
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                mods.insert(KeyModifiers::CONTROL);
            }
            if key.modifiers.contains(KeyModifiers::ALT) {
                mods.insert(KeyModifiers::ALT);
            }
            Input {
                key: Key::Char(c),
                ctrl: mods.contains(KeyModifiers::CONTROL),
                alt: mods.contains(KeyModifiers::ALT),
                shift: key.modifiers.contains(KeyModifiers::SHIFT),
            }
        }
        KeyCode::Backspace => Input {
            key: Key::Backspace,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Enter => Input {
            key: Key::Enter,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Left => Input {
            key: Key::Left,
            ctrl: false,
            alt: key.modifiers.contains(KeyModifiers::ALT)
                || key.modifiers.contains(KeyModifiers::CONTROL),
            shift: false,
        },
        KeyCode::Right => Input {
            key: Key::Right,
            ctrl: false,
            alt: key.modifiers.contains(KeyModifiers::ALT)
                || key.modifiers.contains(KeyModifiers::CONTROL),
            shift: false,
        },
        KeyCode::Up => Input {
            key: Key::Up,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Down => Input {
            key: Key::Down,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Delete => Input {
            key: Key::Delete,
            ctrl: false,
            alt: false,
            shift: false,
        },
        _ => Input::default(),
    };

    // --- CRITICAL FIX: HIGHEST PRIORITY INTERCEPT FOR AUTOCOMPLETE ---
    if app.is_editing && app.show_autocomplete {
        let filtered_files: Vec<&String> = app
            .files
            .iter()
            .filter(|f| {
                f.to_lowercase()
                    .contains(&app.autocomplete_query.to_lowercase())
            })
            .collect();

        match key.code {
            KeyCode::Esc => {
                app.show_autocomplete = false;
                app.autocomplete_query.clear();
                app.autocomplete_index = 0;

                // Wipe out the incomplete link characters entirely
                app.text_editor.delete_char();
                app.text_editor.delete_char();
            }
            KeyCode::Up => {
                if app.autocomplete_index > 0 {
                    app.autocomplete_index -= 1;
                }
            }
            KeyCode::Down => {
                if !filtered_files.is_empty() && app.autocomplete_index < filtered_files.len() - 1 {
                    app.autocomplete_index += 1;
                }
            }
            KeyCode::Enter => {
                if !filtered_files.is_empty() && app.autocomplete_index < filtered_files.len() {
                    let chosen_file = filtered_files[app.autocomplete_index];
                    let link_name = chosen_file.strip_suffix(".md").unwrap_or(chosen_file);

                    // Drop the name cleanly inside the pre-rendered [[]] container
                    app.text_editor.insert_str(link_name);

                    // Safely jump the cursor past the closing "]]" brackets
                    app.text_editor
                        .move_cursor(tui_textarea::CursorMove::Forward);
                    app.text_editor
                        .move_cursor(tui_textarea::CursorMove::Forward);
                }
                app.show_autocomplete = false;
                app.autocomplete_query.clear();
                app.autocomplete_index = 0;
            }
            KeyCode::Backspace => {
                if app.autocomplete_query.is_empty() {
                    app.show_autocomplete = false;
                    app.text_editor.delete_char();
                } else {
                    app.autocomplete_query.pop();
                    app.autocomplete_index = 0;
                    app.text_editor.delete_char();
                }
            }
            KeyCode::Char(c) => {
                if c == ']' {
                    app.show_autocomplete = false;
                    app.autocomplete_query.clear();
                    app.autocomplete_index = 0;
                    app.text_editor
                        .move_cursor(tui_textarea::CursorMove::Forward);
                    app.text_editor
                        .move_cursor(tui_textarea::CursorMove::Forward);
                } else {
                    app.autocomplete_query.push(c);
                    app.autocomplete_index = 0;
                    app.text_editor.insert_char(c);
                }
            }
            _ => {}
        }
        return;
    }

    if app.show_sidebar_search {
        match key.code {
            KeyCode::Esc => {
                app.show_sidebar_search = false;
                app.sidebar_search_query.clear();
                app.selected_index = 0;
                app.load_selected_file();
            }
            KeyCode::Enter => {
                app.show_sidebar_search = false;
            }
            KeyCode::Backspace => {
                app.sidebar_search_query.pop();
                app.selected_index = 0;
                app.load_selected_file();
            }
            KeyCode::Char(c) => {
                app.sidebar_search_query.push(c);
                app.selected_index = 0;
                app.load_selected_file();
            }
            _ => {}
        }
        return;
    }

    if app.show_search_bar {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                app.show_search_bar = false;
                let _ = app.text_editor.set_search_pattern("");
            }
            _ => {
                if input_event != Input::default() {
                    app.search_input.input(input_event);
                    let query = app.search_input.lines()[0].clone();

                    if query.is_empty() {
                        let _ = app.text_editor.set_search_pattern("");
                    } else {
                        let _ = app.text_editor.set_search_pattern(&query);
                    }
                }
            }
        }
        return;
    }

    if app.show_new_note_modal || app.show_rename_modal {
        match key.code {
            KeyCode::Enter => {
                let input_raw = app.modal_input.lines()[0].trim().to_string();
                if !input_raw.is_empty() {
                    let mut target_name = input_raw;
                    if !target_name.ends_with(".md") {
                        target_name.push_str(".md");
                    }

                    if app.show_rename_modal && !app.file_paths.is_empty() {
                        let old_path_str = &app.file_paths[app.selected_index];
                        let old_path = std::path::Path::new(old_path_str);
                        let new_path = app.vault_path.join(&target_name);

                        if fs::rename(old_path, &new_path).is_ok() {
                            app.reload_files_from_db();
                            if let Some(pos) = app.files.iter().position(|f| f == &target_name) {
                                app.selected_index = pos;
                            }
                            app.load_selected_file();
                        }
                        app.show_rename_modal = false;
                    } else if app.show_new_note_modal {
                        let file_path = app.vault_path.join(&target_name);
                        let _ = fs::write(file_path, format!("# {}\n", target_name));
                        app.reload_files_from_db();
                        if let Some(pos) = app.files.iter().position(|f| f == &target_name) {
                            app.selected_index = pos;
                        }
                        app.load_selected_file();
                        app.show_new_note_modal = false;
                    }
                    app.modal_input = TextArea::default();
                }
            }
            KeyCode::Esc => {
                app.show_new_note_modal = false;
                app.show_rename_modal = false;
                app.modal_input = TextArea::default();
            }
            _ => {
                if input_event != Input::default() {
                    app.modal_input.input(input_event);
                }
            }
        }
        return;
    }

    if app.is_editing {
        match key.code {
            KeyCode::Esc => {
                app.is_editing = false;
                app.save_current_file();
                app.reload_files_from_db();

                if !app.search_input.lines()[0].is_empty() {
                    let query = &app.search_input.lines()[0];
                    let _ = app.text_editor.set_search_pattern(query);
                }
            }
            KeyCode::Char('[') => {
                let cursor = app.text_editor.cursor();
                let lines = app.text_editor.lines();
                let current_line = &lines[cursor.0];

                if current_line.as_bytes().get(cursor.1) == Some(&b']') {
                    app.text_editor.delete_char();

                    app.text_editor.insert_str("]]");

                    app.text_editor.move_cursor(tui_textarea::CursorMove::Back);
                    app.text_editor.move_cursor(tui_textarea::CursorMove::Back);

                    app.show_autocomplete = true;
                    app.autocomplete_query.clear();
                    app.autocomplete_index = 0;
                } else {
                    app.text_editor.insert_str("[]");
                    app.text_editor.move_cursor(tui_textarea::CursorMove::Back);
                }
            }
            KeyCode::Char('(') => {
                app.text_editor.insert_str("()");
                app.text_editor.move_cursor(tui_textarea::CursorMove::Back);
            }
            KeyCode::Char('{') => {
                app.text_editor.insert_str("{}");
                app.text_editor.move_cursor(tui_textarea::CursorMove::Back);
            }
            KeyCode::Char('"') => {
                app.text_editor.insert_str("\"\"");
                app.text_editor.move_cursor(tui_textarea::CursorMove::Back);
            }
            _ => {
                if input_event != Input::default() {
                    app.text_editor.input(input_event);
                }
            }
        }
        return;
    }

    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('n') => {
            app.show_new_note_modal = true;
            app.modal_input = TextArea::default();
        }
        KeyCode::Char('r') => {
            if app.active_panel == ActivePanel::Sidebar && !app.files.is_empty() {
                let current_name = app.files[app.selected_index].clone();
                app.modal_input = TextArea::new(vec![current_name]);
                app.show_rename_modal = true;
            }
        }
        KeyCode::Char('/') => {
            if app.active_panel == ActivePanel::Sidebar {
                app.show_sidebar_search = true;
            }
        }
        KeyCode::Char('f') => {
            if app.active_panel == ActivePanel::Viewer {
                app.show_search_bar = true;
            }
        }
        KeyCode::Char('c') => {
            if app.active_panel == ActivePanel::Sidebar {
                app.sidebar_search_query.clear();
                app.selected_index = 0;
                app.load_selected_file();
            } else {
                app.search_input = TextArea::default();
                let _ = app.text_editor.set_search_pattern("");
            }
        }
        KeyCode::Char('e') | KeyCode::Char('i') => {
            if app.active_panel == ActivePanel::Viewer && !app.files.is_empty() {
                app.is_editing = true;
            }
        }
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if app.active_panel == ActivePanel::Viewer {
                let cursor = app.text_editor.cursor();
                let lines = app.text_editor.lines();
                if cursor.0 < lines.len() {
                    let current_line = &lines[cursor.0];
                    let col = cursor.1;

                    if let Some(start_bracket) = current_line[..col].rfind("[[") {
                        if let Some(end_bracket) = current_line[col..].find("]]") {
                            let extracted_title = current_line
                                [start_bracket + 2..col + end_bracket]
                                .trim()
                                .to_string();
                            let mut lookup_filename = extracted_title;
                            if !lookup_filename.ends_with(".md") {
                                lookup_filename.push_str(".md");
                            }

                            if let Some(pos) = app.files.iter().position(|f| f == &lookup_filename)
                            {
                                app.sidebar_search_query.clear();
                                app.selected_index = pos;
                                app.load_selected_file();
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Tab => {
            app.active_panel = match app.active_panel {
                ActivePanel::Sidebar => ActivePanel::Viewer,
                ActivePanel::Viewer => ActivePanel::Sidebar,
            };
        }
        KeyCode::Down => {
            let filtered_count = app
                .files
                .iter()
                .filter(|f| {
                    f.to_lowercase()
                        .contains(&app.sidebar_search_query.to_lowercase())
                })
                .count();
            if app.active_panel == ActivePanel::Sidebar && filtered_count > 0 {
                if app.selected_index < filtered_count - 1 {
                    app.selected_index += 1;
                    app.load_selected_file();
                }
            }
        }
        KeyCode::Up => {
            if app.active_panel == ActivePanel::Sidebar && app.selected_index > 0 {
                app.selected_index -= 1;
                app.load_selected_file();
            }
        }
        _ => {}
    }
}
