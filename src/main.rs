use std::io;

use atrium::{AgentSpec, App, ipc::hook};
use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
    execute,
};

/// What atrium holds when you do not say otherwise.
const DEFAULT_AGENT: &str = "claude";

/// The subcommand agents call back through. Not a program you would run.
const HOOK_SUBCOMMAND: &str = "hook";

fn agent_spec(args: &[String]) -> AgentSpec {
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    match args.split_first() {
        Some((program, rest)) => AgentSpec::new(program.clone(), rest.to_vec(), cwd),
        None => AgentSpec::new(DEFAULT_AGENT, Vec::new(), cwd),
    }
}

fn run_tui(spec: AgentSpec) -> io::Result<()> {
    let mut terminal = ratatui::init();
    // ratatui does not turn this on, and without it a paste arrives as a burst
    // of individual keystrokes.
    execute!(io::stdout(), EnableBracketedPaste)?;

    let result = terminal.size().and_then(|size| App::new(spec, size.height, size.width)).and_then(|mut app| app.run(&mut terminal));

    execute!(io::stdout(), DisableBracketedPaste)?;
    ratatui::restore();
    result
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.first().is_some_and(|a| a == HOOK_SUBCOMMAND) {
        return hook::run(args.get(1).map_or("", String::as_str));
    }

    run_tui(agent_spec(&args))
}
