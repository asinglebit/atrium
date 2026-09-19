use std::{env, io, io::Write, os::unix::net::UnixStream};

use crate::ipc::wire::Report;

/// The `atrium hook <Event>` side: say which agent did what, then get out of
/// the way. Never fails loudly -- a hook that errors is a hook that interrupts
/// the agent, and a missed status update is not worth that.
pub fn run(event: &str) -> io::Result<()> {
    // Claude writes its payload to stdin. atrium does not need it, since the
    // event arrives as an argument, but draining it avoids an EPIPE upstream.
    let _ = io::copy(&mut io::stdin().lock(), &mut io::sink());

    let (Ok(agent_id), Ok(socket)) = (env::var("ATRIUM_AGENT_ID"), env::var("ATRIUM_SOCK")) else {
        // Not running under atrium: nothing to report, and that is fine.
        return Ok(());
    };
    let Ok(agent_id) = agent_id.parse() else {
        return Ok(());
    };

    if let Ok(mut stream) = UnixStream::connect(socket) {
        let _ = stream.write_all(Report { agent_id, event: event.to_owned() }.encode().as_bytes());
    }
    Ok(())
}
