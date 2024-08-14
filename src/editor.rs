use std::{
    io::{self, Read, Seek, Write},
    process::exit,
};

use colored::Colorize;
use crossterm::{
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, size},
    ExecutableCommand,
};
use edit::Builder;

pub(crate) struct Editor {
    current_text: String,
    preview_display: bool,
}

impl Editor {
    pub(crate) fn new(starting_text: String) -> Self {
        Editor {
            current_text: starting_text,
            preview_display: true,
        }
    }
    fn write_text(&self) -> anyhow::Result<()> {
        let mut stdout = io::stdout();
        let output = format!(
            "Processed text:\n{}\n({}) to open in {}, ({}) to toggle Markdown preview, ({}) to submit\n",
            {
                if self.preview_display {
                    termimad::term_text(&self.current_text).to_string()
                } else {
                    self.current_text.clone()
                }
            },
            "e".cyan(),
            edit::get_editor()?.to_str().unwrap().split({
                if cfg!(unix) { "/" }
                else { "\\" }
            }).last().unwrap(),
            "p".bright_red(),
            "enter".green()
        );
        stdout.execute(Print(output))?;
        Ok(())
    }
    pub(crate) fn prompt(&mut self) -> anyhow::Result<String> {
        let mut file = Builder::new().suffix(".md").tempfile()?;
        file.write_all(self.current_text.as_bytes())?;
        // Have to do this to work around this issue: https://github.com/crossterm-rs/crossterm/issues/124
        if cfg!(windows) {
            read()?;
        }
        let event_kind = {
            if cfg!(windows) {
                KeyEventKind::Release
            } else {
                KeyEventKind::Press
            }
        };
        let mut current_dimensions = size()?;

        self.write_text()?;
        loop {
            enable_raw_mode()?;
            match read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('e'),
                    modifiers: KeyModifiers::NONE,
                    kind,
                    ..
                }) if kind == event_kind => {
                    edit::edit_file(&file)?;
                    self.current_text = String::new();
                    file.seek(std::io::SeekFrom::Start(0))?;
                    file.read_to_string(&mut self.current_text)?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('p'),
                    modifiers: KeyModifiers::NONE,
                    kind,
                    ..
                }) if kind == event_kind => self.preview_display = !self.preview_display,
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    kind,
                    ..
                }) if kind == event_kind => {
                    disable_raw_mode()?;
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    disable_raw_mode()?;
                    println!("Operation cancelled. Exiting...");
                    exit(1);
                }
                Event::Resize(col, row) if (col, row) != current_dimensions => {
                    current_dimensions = (col, row);
                }
                _ => {
                    continue;
                }
            }
            clearscreen::clear()?;
            disable_raw_mode()?;
            self.write_text()?;
        }
        Ok(self.current_text.clone())
    }
}
