use std::io;

use atrium::{AgentSpec, App, Config, VERSION, core::projects, helpers::palette, ipc::hook};
use crossterm::{
    event::{DisableBracketedPaste, EnableBracketedPaste},
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
    Everything goes to the focused agent except chords behind the leader (F12):
    F12 n  hold a new agent      F12 j / k  next / previous
    F12 x  dismiss this one      F12 1-9    jump to that agent
    F12 q  quit                  F12 F12    send the leader itself

CONFIG";

fn print_help() {
    println!("{HELP}");
    println!("    {}", Config::path().display());
    println!("    themes: {}", palette::PRESETS.join(", "));
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
    println!("  leader: {:?}", config.keymap.leader);
    println!("  theme:  {:?}", config.theme.idle);
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
    execute!(io::stdout(), EnableBracketedPaste)?;

    let result = terminal.size().and_then(|size| App::new(spec, config, size.height, size.width)).and_then(|mut app| app.run(&mut terminal));

    execute!(io::stdout(), DisableBracketedPaste)?;
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
