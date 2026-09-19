use std::{
    io::{self, Read, Write},
    sync::{Arc, Mutex},
    thread,
};

use portable_pty::{Child, CommandBuilder, MasterPty, PtyPair, PtySize, native_pty_system};
use tui_term::vt100;

/// Lines of history kept above the visible screen, per agent.
const SCROLLBACK: usize = 10_000;

/// The parser is shared: a reader thread writes to it, the draw loop reads it.
pub type SharedParser = Arc<Mutex<vt100::Parser>>;

/// One agent CLI running on its own pty, with a parsed screen to render.
pub struct PtySession {
    parser: SharedParser,
    writer: Box<dyn Write + Send>,
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send + Sync>,
    rows: u16,
    cols: u16,
}

/// portable-pty reports anyhow errors; the rest of atrium speaks io::Error.
fn io_err(e: impl std::fmt::Display) -> io::Error {
    io::Error::other(e.to_string())
}

impl PtySession {
    pub fn spawn(mut cmd: CommandBuilder, rows: u16, cols: u16) -> io::Result<Self> {
        // vt100 parses what a real xterm would emit, so tell the child that is
        // what it is talking to.
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");

        let size = PtySize { rows, cols, pixel_width: 0, pixel_height: 0 };
        let PtyPair { slave, master } = native_pty_system().openpty(size).map_err(io_err)?;

        let child = slave.spawn_command(cmd).map_err(io_err)?;
        // Dropping our end of the slave is what lets the reader see EOF once
        // the child exits; without it the read below blocks forever.
        drop(slave);

        let mut reader = master.try_clone_reader().map_err(io_err)?;
        let writer = master.take_writer().map_err(io_err)?;

        let parser: SharedParser = Arc::new(Mutex::new(vt100::Parser::new(rows, cols, SCROLLBACK)));
        let sink = Arc::clone(&parser);
        thread::spawn(move || {
            let mut buf = [0u8; 8192];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if let Ok(mut parser) = sink.lock() {
                            parser.process(&buf[..n]);
                        }
                    },
                }
            }
        });

        Ok(Self { parser, writer, master, child, rows, cols })
    }

    pub fn parser(&self) -> &SharedParser {
        &self.parser
    }

    pub fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.writer.write_all(bytes)?;
        self.writer.flush()
    }

    /// Resizes both halves: the kernel (which signals the child) and the parser
    /// (which decides how the screen reflows).
    pub fn resize(&mut self, rows: u16, cols: u16) -> io::Result<()> {
        if (rows, cols) == (self.rows, self.cols) || rows == 0 || cols == 0 {
            return Ok(());
        }
        self.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }).map_err(io_err)?;
        if let Ok(mut parser) = self.parser.lock() {
            parser.screen_mut().set_size(rows, cols);
        }
        self.rows = rows;
        self.cols = cols;
        Ok(())
    }

    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.process_id()
    }
}

/// Dismissing an agent drops its session, and dropping one has to end the
/// child. Letting the pty close on its own only hangs the child up, which a CLI
/// is free to ignore -- and waiting afterwards reaps it, so a dismissed agent
/// leaves no zombie behind for as long as atrium runs.
impl Drop for PtySession {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
#[path = "../tests/core/pty.rs"]
mod tests;
