use std::io;

use atrium::App;
use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
};
use portable_pty::CommandBuilder;

/// What atrium holds when you do not say otherwise.
const DEFAULT_AGENT: &str = "claude";

fn agent_command() -> CommandBuilder {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut cmd = match args.split_first() {
        Some((program, rest)) => {
            let mut cmd = CommandBuilder::new(program);
            cmd.args(rest);
            cmd
        },
        None => CommandBuilder::new(DEFAULT_AGENT),
    };
    if let Ok(cwd) = std::env::current_dir() {
        cmd.cwd(cwd);
    }
    cmd
}

fn main() -> io::Result<()> {
    let cmd = agent_command();

    let mut terminal = ratatui::init();
    // ratatui does not turn this on, and without it a paste arrives as a burst
    // of individual keystrokes.
    execute!(io::stdout(), EnableBracketedPaste)?;

    let result = terminal.size().and_then(|size| App::new(cmd, size.height, size.width)).and_then(|mut app| app.run(&mut terminal));

    execute!(io::stdout(), DisableBracketedPaste)?;
    ratatui::restore();
    result
}
