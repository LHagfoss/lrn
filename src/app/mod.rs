pub mod handlers;

use crate::database::DbEngine;
use crate::ui;
use crossterm::event::{self, Event as CEvent};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tui_textarea::TextArea;

#[derive(PartialEq, Eq)]
pub enum ActivePanel {
    Sidebar,
    Viewer,
}

pub struct App<'a> {
    pub vault_path: PathBuf,
    pub current_file: String,

    pub text_editor: TextArea<'a>,
    pub modal_input: TextArea<'a>,
    pub search_input: TextArea<'a>,

    pub show_autocomplete: bool,
    pub autocomplete_query: String,
    pub autocomplete_index: usize,

    pub files: Vec<String>,
    pub file_paths: Vec<String>,
    pub selected_index: usize,
    pub should_quit: bool,

    pub show_new_note_modal: bool,
    pub show_rename_modal: bool,
    pub show_search_bar: bool,
    pub show_sidebar_search: bool,
    pub sidebar_search_query: String,

    pub active_panel: ActivePanel,
    pub is_editing: bool,
    pub(crate) db: DbEngine,
}

impl<'a> App<'a> {
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
            text_editor: TextArea::default(),
            modal_input: TextArea::default(),
            search_input: TextArea::default(),
            show_autocomplete: false,
            autocomplete_query: String::new(),
            autocomplete_index: 0,
            files,
            file_paths,
            selected_index: 0,
            should_quit: false,
            show_new_note_modal: false,
            show_rename_modal: false,
            show_search_bar: false,
            show_sidebar_search: false,
            sidebar_search_query: String::new(),
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
        let filtered: Vec<(usize, &String)> = self
            .files
            .iter()
            .enumerate()
            .filter(|(_, name)| {
                name.to_lowercase()
                    .contains(&self.sidebar_search_query.to_lowercase())
            })
            .collect();

        if !filtered.is_empty() && self.selected_index < filtered.len() {
            let direct_index = filtered[self.selected_index].0;
            let path = &self.file_paths[direct_index];
            let title = &self.files[direct_index];

            self.current_file = title.clone();
            if let Ok(content) = fs::read_to_string(path) {
                let lines: Vec<String> = content.lines().map(String::from).collect();
                let textarea = TextArea::new(lines);

                self.text_editor = textarea;

                if !self.search_input.lines()[0].is_empty() {
                    let query = &self.search_input.lines()[0];
                    let _ = self.text_editor.set_search_pattern(query);
                }
            } else {
                self.text_editor = TextArea::default();
            }
        }
    }

    pub fn save_current_file(&self) {
        let filtered: Vec<(usize, &String)> = self
            .files
            .iter()
            .enumerate()
            .filter(|(_, name)| {
                name.to_lowercase()
                    .contains(&self.sidebar_search_query.to_lowercase())
            })
            .collect();

        if !filtered.is_empty() && self.selected_index < filtered.len() {
            let direct_index = filtered[self.selected_index].0;
            let path = &self.file_paths[direct_index];
            let content = self.text_editor.lines().join("\n");
            let _ = fs::write(path, content);
        }
    }
}
