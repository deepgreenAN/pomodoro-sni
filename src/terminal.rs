use crate::AppError;

use std::process::Command;

#[derive(Debug)]
enum TerminalType {
    GnomeTerminal,
    Konsole,
    Xfce4Terminal,
    Alacritty,
    Kitty,
    MateTerminal,
    Ptyxis,
    LxTerminal,
    QTerminal,
    Kgx,
}

impl TerminalType {
    fn command(&self) -> String {
        use TerminalType::*;

        let terminal = match self {
            GnomeTerminal => "gnome-terminal",
            Konsole => "konsole",
            Xfce4Terminal => "xfce4-terminal",
            Alacritty => "alacritty",
            Kitty => "kitty",
            MateTerminal => "mate-terminal",
            Ptyxis => "ptyxis",
            LxTerminal => "lxterminal",
            QTerminal => "qterminal",
            Kgx => "kgx",
        };

        terminal.to_owned()
    }
    fn execute_args(&self) -> Vec<String> {
        use TerminalType::*;

        match self {
            GnomeTerminal => vec!["--".to_owned()],
            Konsole => vec!["-e".to_owned()],
            Xfce4Terminal => vec!["-x".to_owned()],
            Alacritty => vec!["-e".to_owned()],
            Kitty => vec![],
            MateTerminal => vec!["-x".to_owned()],
            Ptyxis => vec!["--new-window".to_owned(), "--".to_owned()],
            LxTerminal => vec!["-e".to_owned()],
            QTerminal => vec!["-e".to_owned()],
            Kgx => vec!["-e".to_owned()],
        }
    }

    fn available() -> Result<Self, AppError> {
        use TerminalType::*;

        let terminals = [
            GnomeTerminal,
            Konsole,
            Xfce4Terminal,
            Alacritty,
            Kitty,
            MateTerminal,
            Ptyxis,
            LxTerminal,
            QTerminal,
            Kgx,
        ];

        terminals.into_iter().find(|term|{
            which::which(term.command()).is_ok()
        }).ok_or(AppError::CustomError("Could not find available terminal. Please use command line arguments instead of interactive cli.".to_owned()))
    }
}

/// 利用可能なターミナルのCommandを取得する．
pub fn get_available_terminal_cmd() -> Result<Command, AppError> {
    let term = TerminalType::available()?;
    let mut cmd = Command::new(term.command());

    for arg in term.execute_args() {
        cmd.arg(arg);
    }

    Ok(cmd)
}
