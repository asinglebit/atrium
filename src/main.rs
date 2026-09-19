use std::io;

use atrium::{AgentSpec, App, Config, VERSION, core::projects, helpers::palette, ipc::hook};
use crossterm::{
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
};

/// What atrium holds when you do not say otherwise.
const DEFAULT_AGENT: &str = "claude";

/// The subcommand agents call back through. Not a program you would run.
const HOOK_SUBCOMMAND: &str = "hook";

const HELP: &str = "\
atrium -- a terminal UI that holds coding agents

USAGE
    atrium [COMMAND ...]      hold COMMAND, or `claude` if none is given
    atrium hook <EVENT>       report an agent's status (agents call this, you do not)

OPTIONS
    -h, --help                show this
    -v, --version             show the version
        --check-config        report what the config file says, and what it got wrong

KEYS
    Actions fire directly; every other key reaches the focused agent.
    ctrl+t  hold a new agent     ctrl+n / ctrl+p  next / previous
    ctrl+g  go to (1-9 jumps)    ctrl+o           show / hide the sidebar
    ctrl+x  close this one       ctrl+s           settings
    ctrl+q  quit

    The mouse works: click a row, wheel to scroll, drag the line between the
    panes to resize the sidebar, right-click anywhere for a menu. Inside the
    agent everything else is forwarded on, so the agent's own mouse support
    keeps working.

    Untouched, so the agent keeps them: ctrl+c, ctrl+d, ctrl+z, ctrl+v,
    ctrl+l, ctrl+r, ctrl+u, ctrl+w, ctrl+a, ctrl+e, ctrl+k, esc.

CONFIG";

fn print_help() {
    println!("{HELP}");
    println!("    {}", Config::path().display());
    println!("    themes: {} presets; colours come from theme.json, shared with guitar", palette::THEME_PRESETS.len());
    println!("    projects come from $ATRIUM_PROJECTS, else ~/projects");
}

/// Says what atrium actually loaded, so a config that does nothing can be
/// diagnosed without starting the TUI over it.
fn check_config() {
    let path = Config::path();
    let config = Config::load();

    println!("config: {}", path.display());
    if !path.exists() {
        println!("  (no file yet -- these are the defaults)");
    }
    let claimed: Vec<String> = config.keymap.claimed().iter().map(|chord| chord.label()).collect();
    println!("  keys:   {}", claimed.join(", "));
    println!("  theme:  {}", config.theme.name.label());
    println!("  projects: {}", projects::default_root().display());

    if config.problems.is_empty() {
        println!("  no problems");
        return;
    }
    println!("\n{} problem(s):", config.problems.len());
    for problem in &config.problems {
        println!("  - {problem}");
    }
    // Exit rather than return an error: main would print it with Debug, and a
    // list already shown once does not need repeating as `Custom { .. }`.
    std::process::exit(1);
}

fn agent_spec(args: &[String]) -> AgentSpec {
    let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());
    match args.split_first() {
        Some((program, rest)) => AgentSpec::new(program.clone(), rest.to_vec(), cwd),
        None => AgentSpec::new(DEFAULT_AGENT, Vec::new(), cwd),
    }
}

fn run_tui(spec: AgentSpec, config: Config) -> io::Result<()> {
    let mut terminal = ratatui::init();
    // ratatui does not turn this on, and without it a paste arrives as a burst
    // of individual keystrokes.
    execute!(io::stdout(), EnableBracketedPaste, EnableMouseCapture)?;

    let result = terminal.size().and_then(|size| App::new(spec, config, size.height, size.width)).and_then(|mut app| app.run(&mut terminal));

    execute!(io::stdout(), DisableMouseCapture, DisableBracketedPaste)?;
    ratatui::restore();
    result
}

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Everything here is handled before ratatui takes over the terminal, so the
    // output stays plain enough for a script to read.
    match args.first().map(String::as_str) {
        Some(HOOK_SUBCOMMAND) => return hook::run(args.get(1).map_or("", String::as_str)),
        Some("-h" | "--help") => {
            print_help();
            return Ok(());
        },
        Some("-v" | "--version") => {
            println!("{VERSION}");
            return Ok(());
        },
        Some("--check-config") => {
            check_config();
            return Ok(());
        },
        _ => {},
    }

    run_tui(agent_spec(&args), Config::load())
}
