use std::{
    fs,
    io::{BufRead, BufReader, ErrorKind},
    os::unix::{
        fs::{FileTypeExt, MetadataExt},
        net::{UnixListener, UnixStream},
    },
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    thread,
};

use crate::ipc::wire::Report;

/// Distinguishes servers within one process, which the pid alone cannot do.
static NEXT_SERVER: AtomicU64 = AtomicU64::new(0);

/// Where the socket lives. `XDG_RUNTIME_DIR` is a tmpfs, so nothing survives a
/// reboot to be mistaken for a live atrium.
fn socket_dir() -> PathBuf {
    match std::env::var_os("XDG_RUNTIME_DIR") {
        Some(dir) => PathBuf::from(dir).join("atrium"),
        None => std::env::temp_dir().join(format!("atrium-{}", current_uid())),
    }
}

/// Whoever owns the home directory. Not libc, so the fallback path costs no
/// dependency, and not /proc, which is a thing only Linux has.
fn current_uid() -> u32 {
    dirs::home_dir().and_then(|home| fs::metadata(home).ok()).map_or(0, |home| home.uid())
}

/// An atrium that is killed rather than quit never runs its `Drop`, so its
/// socket outlives it. Clearing the dead ones on the way in keeps the directory
/// from filling up with them.
fn sweep_stale(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for path in entries.flatten().map(|entry| entry.path()) {
        if path.extension().is_none_or(|ext| ext != "sock") {
            continue;
        }
        if abandoned(&path) {
            let _ = fs::remove_file(&path);
        }
    }
}

/// Whether a socket has nobody behind it. Knocking beats reading the pid out of
/// the name: off Linux there is no /proc to look a pid up in, so every pid read
/// as dead and every socket was swept -- and a pid the system has handed out
/// again would read as alive.
///
/// Only a refusal is proof. A connect that fails for want of a file descriptor,
/// or a permission, says nothing about the far end, and sweeping on it would
/// unlink a living atrium's only door.
fn abandoned(path: &Path) -> bool {
    let Ok(meta) = path.metadata() else {
        return false;
    };
    if !meta.file_type().is_socket() {
        return true;
    }
    UnixStream::connect(path).is_err_and(|err| err.kind() == ErrorKind::ConnectionRefused)
}

/// Listens for hook callbacks and hands them to the draw loop.
pub struct StatusServer {
    path: PathBuf,
    reports: Receiver<Report>,
}

impl StatusServer {
    pub fn bind() -> std::io::Result<Self> {
        let dir = socket_dir();
        fs::create_dir_all(&dir)?;
        sweep_stale(&dir);
        let path = dir.join(format!("{}-{}.sock", std::process::id(), NEXT_SERVER.fetch_add(1, Ordering::Relaxed)));
        // A socket left by a crashed atrium would make bind fail.
        let _ = fs::remove_file(&path);

        let listener = UnixListener::bind(&path)?;
        let (tx, reports) = mpsc::channel();
        thread::spawn(move || serve(listener, tx));

        Ok(Self { path, reports })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Everything that arrived since the last look. Never blocks.
    pub fn drain(&self) -> Vec<Report> {
        self.reports.try_iter().collect()
    }
}

impl Drop for StatusServer {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn serve(listener: UnixListener, tx: Sender<Report>) {
    for stream in listener.incoming().flatten() {
        let tx = tx.clone();
        // One short-lived thread per hook, so a client that connects and then
        // stalls cannot hold up the ones behind it.
        thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                if let Some(report) = Report::parse(&line)
                    && tx.send(report).is_err()
                {
                    return;
                }
            }
        });
    }
}

#[cfg(test)]
#[path = "../tests/ipc/server.rs"]
mod tests;
