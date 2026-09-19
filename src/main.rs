use std::io;

use atrium::{
    AgentSpec, App, Config, VERSION,
    core::{profile, projects},
    helpers::palette,
    ipc::hook,
};
use crossterm::{
    event::{DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture},
    execute,
};

/// The subcommand agents call back through. Not a program you would run.
const HOOK_SUBCOMMAND: &str = "hook";

/// Holds a named profile rather than the default one.
const PROFILE_FLAG: &str = "--profile";

const HELP: &str = "\
atrium -- a terminal UI that holds coding agents

USAGE
    atrium                    open on the splash and pick a harness
    atrium --profile NAME     skip it and hold that profile here
    atrium [COMMAND ...]      skip it and hold COMMAND, with no profile at all
    atrium hook <EVENT>       report an agent's status (agents call this, you do not)

OPTIONS
    -h, --help                show this
    -v, --version             show the version
        --check-config        report what the config file says, and what it got wrong

PROFILES
    A profile is a CLI plus what it needs: extra flags, and extra environment.
    A Claude subscription is one of these -- `config_dir` sets CLAUDE_CONFIG_DIR,
    so `work` and `personal` are two profiles rather than two shell aliases.
    They live in profiles.json, and `ctrl+s` -> profiles is where they are made.

    Beside them atrium offers the CLIs it knows and finds installed -- claude,
    opencode, codex -- so an opencode on your PATH needs no profile at all.
    Inside the new-agent modal, `tab` cycles the lot, and the splash lists it.

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

    println!("  installed: (what atrium knows, and where it found it)");
    for program in profile::KNOWN_PROGRAMS {
        match config.installed.iter().find(|found| found.program == program) {
            Some(found) => println!("     {program:<9} {}", found.path.display()),
            None => println!("     {program:<9} not installed"),
        }
    }

    println!("  profiles:  (* is what a bare `atrium` holds)");
    for (index, entry) in config.profiles.iter().enumerate() {
        println!("   {} {}", if index == config.default_profile { "*" } else { " " }, entry.label());
        if let Some(dir) = entry.config_dir() {
            println!("       {}={dir}", profile::CONFIG_DIR_ENV);
        }
        if !entry.args.is_empty() {
            println!("       args: {}", entry.args.join(" "));
        }
    }

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

fn cwd() -> std::path::PathBuf {
    std::env::current_dir().unwrap_or_else(|_| ".".into())
}

/// A named command is held exactly as written, with no profile attached --
/// `atrium bash --norc` should not pick up Claude's environment.
///
/// Nothing at all opens the splash instead of guessing: with no argument
/// nothing was asked for, so atrium asks.
fn agent_spec(args: &[String]) -> Option<AgentSpec> {
    args.split_first().map(|(program, rest)| AgentSpec::new(program.clone(), rest.to_vec(), cwd()))
}

fn profile_names(config: &Config) -> String {
    config.profiles.iter().map(|entry| entry.name.as_str()).collect::<Vec<_>>().join(", ")
}

/// The one flag that replaces having a shell alias per subscription.
fn run_profile(name: Option<&String>, config: Config) -> io::Result<()> {
    let Some(name) = name else {
        eprintln!("--profile needs a name. There is: {}", profile_names(&config));
        std::process::exit(2);
    };
    let Some(profile) = config.profile_named(name).cloned() else {
        eprintln!("no profile named {name:?}. There is: {}", profile_names(&config));
        std::process::exit(1);
    };
    run_tui(Some(AgentSpec::from_profile(&profile, cwd())), config)
}

fn run_tui(spec: Option<AgentSpec>, config: Config) -> io::Result<()> {
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
        Some(PROFILE_FLAG) => return run_profile(args.get(1), Config::load()),
        _ => {},
    }

    let config = Config::load();
    let spec = agent_spec(&args);
    run_tui(spec, config)
}
