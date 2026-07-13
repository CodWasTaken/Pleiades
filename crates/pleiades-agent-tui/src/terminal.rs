//! Panic-safe terminal lifecycle management.

use std::io::{self, IsTerminal, Stdout};
use std::panic;
use std::sync::Once;

use crossterm::cursor::Show;
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

pub type PleiadesTerminal = Terminal<CrosstermBackend<Stdout>>;

pub struct TerminalGuard {
    terminal: PleiadesTerminal,
}

impl TerminalGuard {
    pub fn enter() -> io::Result<Self> {
        install_panic_restoration();
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            EnableBracketedPaste
        ) {
            let _ = disable_raw_mode();
            return Err(error);
        }
        let terminal = match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => terminal,
            Err(error) => {
                restore_terminal();
                return Err(error);
            }
        };
        Ok(Self { terminal })
    }

    pub fn terminal_mut(&mut self) -> &mut PleiadesTerminal {
        &mut self.terminal
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.terminal.show_cursor();
        restore_terminal();
    }
}

fn install_panic_restoration() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            restore_terminal();
            previous(info);
        }));
    });
}

pub fn restore_terminal() {
    let _ = disable_raw_mode();
    if !io::stdout().is_terminal() {
        return;
    }
    let _ = execute!(
        io::stdout(),
        DisableBracketedPaste,
        DisableMouseCapture,
        LeaveAlternateScreen,
        Show
    );
}

#[cfg(test)]
mod tests {
    use super::restore_terminal;

    #[test]
    fn restoration_is_idempotent_without_a_tty() {
        restore_terminal();
        restore_terminal();
    }
}
