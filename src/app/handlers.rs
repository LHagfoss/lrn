use crate::app::{ActivePanel, App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs;
use std::path::Path;
use tui_textarea::TextArea;

pub fn handle_key_input(app: &mut App, key: KeyEvent) {
    // --- Search bar (document-level text search) ---
    if app.show_search_bar {
        match key.code {
            KeyCode::Esc => {
                app.show_search_bar = false;
                app.search_input = TextArea::default();
            }
            KeyCode::Enter => {
                let query = app.search_input.lines()[0].clone();
                app.show_search_bar = false;
                if !query.is_empty() {
                    let _ = app.text_editor.set_search_pattern(&query);
                }
            }
            _ => {
                let input_event = convert_key_to_textarea_input(key);
                if input_event.key != tui_textarea::Key::Null {
                    app.search_input.input(input_event);
                }
            }
        }
        return;
    }

    // --- Sidebar filter mode ---
    if app.show_sidebar_search {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                app.show_sidebar_search = false;
            }
            KeyCode::Char(c) => {
                app.sidebar_search_query.push(c);
            }
            KeyCode::Backspace => {
                app.sidebar_search_query.pop();
            }
            _ => {}
        }
        return;
    }

    // --- Delete confirmation modal ---
    if app.show_delete_modal {
        match key.code {
            KeyCode::Left | KeyCode::Char('c') => {
                app.delete_selection = 0;
            }
            KeyCode::Right | KeyCode::Char('x') => {
                app.delete_selection = 1;
            }
            KeyCode::Enter => {
                if app.delete_selection == 1 {
                    if !app.file_paths.is_empty() && app.selected_index < app.files.len() {
                        let path = &app.file_paths[app.selected_index];
                        let _ = fs::remove_file(path);
                        app.reload_files_from_db();
                        if !app.files.is_empty() {
                            if app.selected_index >= app.files.len() {
                                app.selected_index = app.files.len() - 1;
                            }
                            app.load_selected_file();
                        } else {
                            app.current_file = String::from("No file selected");
                            app.text_editor = TextArea::default();
                        }
                    }
                }
                app.show_delete_modal = false;
                app.modal_input = TextArea::default();
                app.delete_selection = 0;
            }
            KeyCode::Esc => {
                app.show_delete_modal = false;
                app.modal_input = TextArea::default();
                app.delete_selection = 0;
            }
            _ => {}
        }
        return;
    }

    // --- Rename / New note modals ---
    if app.show_rename_modal || app.show_new_note_modal {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                let name = app.modal_input.lines()[0].clone();
                app.modal_input = TextArea::default();

                if !name.is_empty() {
                    // Trim trailing .md if user added it explicitly
                    let target_name = if name.ends_with(".md") {
                        name.clone()
                    } else {
                        format!("{}.md", name)
                    };

                    if app.show_rename_modal {
                        // Rename existing file
                        if !app.files.is_empty() && app.selected_index < app.files.len() {
                            let old_path = Path::new(&app.file_paths[app.selected_index]);
                            let parent = old_path.parent().unwrap_or(&app.vault_path);
                            let new_path = parent.join(&target_name);

                            if let Err(e) = fs::rename(old_path, &new_path) {
                                eprintln!("Rename failed: {}", e);
                            } else {
                                app.reload_files_from_db();
                                // Re-select the renamed file
                                app.sidebar_search_query.clear();
                                if let Some(pos) = app.files.iter().position(|f| f == &target_name) {
                                    app.selected_index = pos;
                                    app.load_selected_file();
                                }
                            }
                        }
                    } else if app.show_new_note_modal {
                        // Create new file
                        let new_path = app.vault_path.join(&target_name);
                        let _ = fs::write(&new_path, "");
                        app.reload_files_from_db();
                        // Select the new file
                        app.sidebar_search_query.clear();
                        if let Some(pos) = app.files.iter().position(|f| f == &target_name) {
                            app.selected_index = pos;
                            app.load_selected_file();
                        }
                    }
                }

                app.show_rename_modal = false;
                app.show_new_note_modal = false;
            }
            _ => {
                let input_event = convert_key_to_textarea_input(key);
                if input_event.key != tui_textarea::Key::Null {
                    app.modal_input.input(input_event);
                }
            }
        }
        return;
    }

    // --- Main event loop ---
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
            // Launch external nvim for editing (full vim keybinds + text)
            if app.active_panel == ActivePanel::Viewer && !app.files.is_empty() {
                app.edit_with_nvim();
            }
        }
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Ctrl+O: follow wiki-link at cursor, navigate to target
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

                        if let Some(pos) = app.files.iter().position(|f| f == &lookup_filename) {
                            app.sidebar_search_query.clear();
                            app.selected_index = pos;
                            app.load_selected_file();
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
        KeyCode::Char('d') => {
            if app.active_panel == ActivePanel::Sidebar && !app.files.is_empty() {
                let current_name = app.files[app.selected_index].clone();
                app.modal_input = TextArea::new(vec![format!("Delete '{}'?", current_name)]);
                app.show_delete_modal = true;
            }
        }
        KeyCode::Char('z') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let _ = app.text_editor.undo();
        }
        KeyCode::Char('y') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let _ = app.text_editor.redo();
        }
        _ => {}
    }
}

/// Convert a crossterm KeyEvent into a tui_textarea Input.
fn convert_key_to_textarea_input(key: KeyEvent) -> tui_textarea::Input {
    match key.code {
        KeyCode::Char(c) => tui_textarea::Input {
            key: tui_textarea::Key::Char(c),
            ctrl: key.modifiers.contains(KeyModifiers::CONTROL),
            alt: false,
            shift: key.modifiers.contains(KeyModifiers::SHIFT),
        },
        KeyCode::Backspace => tui_textarea::Input {
            key: tui_textarea::Key::Backspace,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Enter => tui_textarea::Input {
            key: tui_textarea::Key::Enter,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Left => tui_textarea::Input {
            key: tui_textarea::Key::Left,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Right => tui_textarea::Input {
            key: tui_textarea::Key::Right,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Up => tui_textarea::Input {
            key: tui_textarea::Key::Up,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Down => tui_textarea::Input {
            key: tui_textarea::Key::Down,
            ctrl: false,
            alt: false,
            shift: false,
        },
        KeyCode::Delete => tui_textarea::Input {
            key: tui_textarea::Key::Delete,
            ctrl: false,
            alt: false,
            shift: false,
        },
        _ => tui_textarea::Input::default(),
    }
}
