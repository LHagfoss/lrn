use crate::app::{ActivePanel, App};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs;

pub fn handle_key_input(app: &mut App, key: KeyEvent) {
    // Shared text handling routing logic for modals (Create or Rename)
    if app.show_new_note_modal || app.show_rename_modal {
        match key.code {
            KeyCode::Enter => {
                if !app.new_note_input.trim().is_empty() {
                    let mut target_name = app.new_note_input.trim().to_string();
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
                    app.new_note_input.clear();
                    app.cursor_position = 0;
                }
            }
            KeyCode::Esc => {
                app.show_new_note_modal = false;
                app.show_rename_modal = false;
                app.new_note_input.clear();
                app.cursor_position = 0;
            }
            KeyCode::Left => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    // Option + Left: Skip word backwards
                    let substring = &app.new_note_input[..app.cursor_position];
                    if let Some(idx) = substring.trim_end().rfind(' ') {
                        app.cursor_position = idx + 1;
                    } else {
                        app.cursor_position = 0;
                    }
                } else if app.cursor_position > 0 {
                    app.cursor_position -= 1;
                }
            }
            KeyCode::Right => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    // Option + Right: Skip word forwards
                    let substring = &app.new_note_input[app.cursor_position..];
                    let trimmed = substring.to_string();
                    let space_offset = trimmed.find(|c: char| c.is_whitespace());

                    if let Some(offset) = space_offset {
                        let rest = &trimmed[offset..];
                        let word_start = rest
                            .find(|c: char| !c.is_whitespace())
                            .unwrap_or(rest.len());
                        app.cursor_position += offset + word_start;
                    } else {
                        app.cursor_position = app.new_note_input.len();
                    }
                } else if app.cursor_position < app.new_note_input.len() {
                    app.cursor_position += 1;
                }
            }
            KeyCode::Backspace => {
                if app.cursor_position > 0 {
                    app.new_note_input.remove(app.cursor_position - 1);
                    app.cursor_position -= 1;
                }
            }
            KeyCode::Char(c) => {
                app.new_note_input.insert(app.cursor_position, c);
                app.cursor_position += 1;
            }
            _ => {}
        }
        return;
    }

    // 2. Text Editor Handler
    if app.is_editing {
        match key.code {
            KeyCode::Esc => {
                app.is_editing = false;
                app.save_current_file();
            }
            KeyCode::Enter => {
                app.current_file_content.push('\n');
            }
            KeyCode::Backspace => {
                app.current_file_content.pop();
            }
            KeyCode::Char(c) => {
                app.current_file_content.push(c);
            }
            _ => {}
        }
        return;
    }

    // 3. Navigation/Control Handler
    match key.code {
        KeyCode::Char('q') => {
            app.should_quit = true;
        }
        KeyCode::Char('n') => {
            app.show_new_note_modal = true;
            app.cursor_position = 0;
        }
        KeyCode::Char('r') => {
            if app.active_panel == ActivePanel::Sidebar && !app.files.is_empty() {
                app.new_note_input = app.files[app.selected_index].clone();
                app.cursor_position = app.new_note_input.len();
                app.show_rename_modal = true;
            }
        }
        KeyCode::Char('e') | KeyCode::Char('i') => {
            if app.active_panel == ActivePanel::Viewer && !app.files.is_empty() {
                app.is_editing = true;
            }
        }
        KeyCode::Tab => {
            app.active_panel = match app.active_panel {
                ActivePanel::Sidebar => ActivePanel::Viewer,
                ActivePanel::Viewer => ActivePanel::Sidebar,
            };
        }
        KeyCode::Down => {
            if app.active_panel == ActivePanel::Sidebar {
                if !app.files.is_empty() && app.selected_index < app.files.len() - 1 {
                    app.selected_index += 1;
                    app.load_selected_file();
                }
            }
        }
        KeyCode::Up => {
            if app.active_panel == ActivePanel::Sidebar {
                if app.selected_index > 0 {
                    app.selected_index -= 1;
                    app.load_selected_file();
                }
            }
        }
        _ => {}
    }
}
