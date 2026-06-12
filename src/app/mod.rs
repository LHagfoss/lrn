pub mod handlers;

use crate::database::DbEngine;
use crate::ui;
use crossterm::event::{self, Event as CEvent};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(PartialEq, Eq)]
pub enum ActivePanel {
    Sidebar,
    Viewer,
}

pub struct App {
    pub vault_path: PathBuf,
    pub current_file: String,
    pub current_file_content: String,
    pub files: Vec<String>,
    pub file_paths: Vec<String>,
    pub selected_index: usize,
    pub should_quit: bool,

    pub show_new_note_modal: bool,
    pub show_rename_modal: bool,
    pub new_note_input: String,
    pub cursor_position: usize, // Track cursor character index location

    pub active_panel: ActivePanel,
    pub is_editing: bool,
    pub(crate) db: DbEngine,
}

impl App {
    pub fn new(vault_path: PathBuf) -> Self {
        let mut db = DbEngine::init().expect("Failed to initialize database");
        db.index_vault(&vault_path)
            .expect("Failed to index vault directory");

        let mut files = Vec::new();
        let mut file_paths = Vec::new();

        if let Ok(db_files) = db.get_all_files() {
            for (path, title) in db_files {
                files.push(title);
                file_paths.push(path);
            }
        }

        let mut app = Self {
            vault_path,
            current_file: String::from("No file selected"),
            current_file_content: String::from(""),
            files,
            file_paths,
            selected_index: 0,
            should_quit: false,
            show_new_note_modal: false,
            show_rename_modal: false,
            new_note_input: String::new(),
            cursor_position: 0,
            active_panel: ActivePanel::Sidebar,
            is_editing: false,
            db,
        };

        app.load_selected_file();
        app
    }

    pub fn run(&mut self) -> io::Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        while !self.should_quit {
            terminal.draw(|f| ui::render(f, self))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::from_secs(0));

            if event::poll(timeout)? {
                if let CEvent::Key(key) = event::read()? {
                    handlers::handle_key_input(self, key);
                }
            }

            if last_tick.elapsed() >= tick_rate {
                last_tick = Instant::now();
            }
        }

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    pub fn reload_files_from_db(&mut self) {
        let _ = self.db.index_vault(&self.vault_path);
        self.files.clear();
        self.file_paths.clear();
        if let Ok(db_files) = self.db.get_all_files() {
            for (path, title) in db_files {
                self.files.push(title);
                self.file_paths.push(path);
            }
        }
    }

    pub fn load_selected_file(&mut self) {
        if !self.file_paths.is_empty() && self.selected_index < self.file_paths.len() {
            let path = &self.file_paths[self.selected_index];
            let title = &self.files[self.selected_index];

            self.current_file = title.clone();
            if let Ok(content) = fs::read_to_string(path) {
                self.current_file_content = content;
            } else {
                self.current_file_content = String::from("");
            }
        }
    }

    pub fn save_current_file(&self) {
        if !self.file_paths.is_empty() && self.selected_index < self.file_paths.len() {
            let path = &self.file_paths[self.selected_index];
            let _ = fs::write(path, &self.current_file_content);
        }
    }
}
