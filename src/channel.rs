//! The control channel between `ctl` and a headless host: a Unix socket, on
//! Windows a named pipe. A connection carries one command line and its
//! reply, which may span several lines and ends where the host ends it.

use std::io::Read;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

pub(crate) use imp::{Connection, bind, cleanup, connect};

/// The longest command line a host reads; a longer one is cut there and fails
/// to parse.
const MAX_LINE: u64 = 64 * 1024;

/// The longest reply `ctl` reads.
const REPLY_MAX: u64 = 1024 * 1024;

/// Answers one command line; blocks until the host loop has replied.
pub(crate) type Answer = Arc<dyn Fn(&str) -> String + Send + Sync>;

/// Reads a reply up to `REPLY_MAX` bytes, stopping at `end` when given.
fn read_reply(stream: impl Read, end: Option<u8>) -> std::io::Result<String> {
    let mut bytes = Vec::new();
    let mut stream = stream.take(REPLY_MAX);
    let mut chunk = [0u8; 8192];
    loop {
        let n = stream.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..n]);
        if let Some(end) = end
            && let Some(at) = bytes.iter().position(|&b| b == end)
        {
            bytes.truncate(at);
            break;
        }
    }
    if bytes.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "host closed the connection without a reply",
        ));
    }
    let text = String::from_utf8_lossy(&bytes);
    Ok(text.trim_end_matches(['\r', '\n']).to_owned())
}

#[cfg(unix)]
mod imp {
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::Shutdown;
    use std::os::unix::net::{UnixListener, UnixStream};
    use std::path::Path;
    use std::sync::Arc;

    use super::{Answer, Inflight, MAX_LINE, read_reply};

    pub(crate) struct Listener(UnixListener);

    /// Binds the control socket, replacing a stale one a killed host left behind.
    pub(crate) fn bind(path: &Path) -> std::io::Result<Listener> {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        UnixListener::bind(path).map(Listener)
    }

    impl Listener {
        /// One thread per connection: a command may wait (`wait-idle`, `quit`),
        /// and another client must not queue behind it.
        pub(crate) fn spawn(self, answer: Answer, inflight: Arc<Inflight>) {
            std::thread::spawn(move || {
                for stream in self.0.incoming() {
                    let Ok(stream) = stream else { continue };
                    let answer = answer.clone();
                    let guard = Inflight::enter(&inflight);
                    std::thread::spawn(move || {
                        serve(stream, &answer);
                        drop(guard);
                    });
                }
            });
        }
    }

    /// Reads one command line, has the loop answer it, writes the reply and
    /// closes the connection, which ends the reply for the client.
    fn serve(stream: UnixStream, answer: &Answer) {
        let mut line = String::new();
        if BufReader::new((&stream).take(MAX_LINE))
            .read_line(&mut line)
            .is_err()
        {
            return;
        }
        let reply = answer(&line);
        let mut stream = stream;
        let _ = writeln!(stream, "{reply}");
        let _ = stream.shutdown(Shutdown::Both);
    }

    pub(crate) fn cleanup(path: &Path) {
        let _ = std::fs::remove_file(path);
    }

    pub(crate) struct Connection(UnixStream);

    pub(crate) fn connect(path: &Path) -> std::io::Result<Connection> {
        UnixStream::connect(path).map(Connection)
    }

    impl Connection {
        /// Sends one command line and reads the reply to the end.
        pub(crate) fn request(mut self, line: &str) -> std::io::Result<String> {
            self.0.write_all(line.as_bytes())?;
            self.0.write_all(b"\n")?;
            read_reply(&self.0, None)
        }
    }
}

#[cfg(windows)]
mod imp {
    use std::ffi::OsString;
    use std::fs::File;
    use std::io::{ErrorKind, Write};
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Duration;

    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

    use super::{Answer, Inflight, MAX_LINE, read_reply};

    const PREFIX: &str = r"\\.\pipe\";
    /// `ERROR_PIPE_BUSY`: every instance of the pipe is taken for the moment.
    const ERROR_PIPE_BUSY: i32 = 231;
    /// How often and how long `connect` waits for a free instance.
    const BUSY_TRIES: u32 = 250;
    const BUSY_WAIT: Duration = Duration::from_millis(20);
    /// How long a reply waits for its client to read it and hang up.
    const HANGUP_WAIT: Duration = Duration::from_secs(2);
    /// Ends a reply: a pipe client cannot see the server close the
    /// connection before the server has waited for the client to read, so
    /// the reply carries its own end.
    const END: u8 = 0;

    pub(crate) struct Listener {
        runtime: tokio::runtime::Runtime,
        first: NamedPipeServer,
        name: OsString,
    }

    /// `app-a` becomes `\\.\pipe\app-a`; a full pipe path stays as it is.
    fn pipe_name(control: &Path) -> std::io::Result<OsString> {
        let text = control.to_string_lossy();
        if text.starts_with(PREFIX) {
            return Ok(control.as_os_str().to_owned());
        }
        if text.is_empty() || text.contains(['\\', '/']) {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "on Windows the control channel is a pipe name such as myapp-agent-1",
            ));
        }
        Ok(OsString::from(format!("{PREFIX}{text}")))
    }

    fn instance(name: &OsString, first: bool) -> std::io::Result<NamedPipeServer> {
        ServerOptions::new()
            // Fails when another process already owns the name, instead of
            // sharing it with that process.
            .first_pipe_instance(first)
            .reject_remote_clients(true)
            .create(name)
    }

    pub(crate) fn bind(control: &Path) -> std::io::Result<Listener> {
        let name = pipe_name(control)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let first = {
            let _guard = runtime.enter();
            instance(&name, true)?
        };
        Ok(Listener {
            runtime,
            first,
            name,
        })
    }

    impl Listener {
        /// A task per connection, with a fresh pipe instance waiting for the
        /// next client while it runs: a command may wait (`wait-idle`, `quit`),
        /// and another client must not queue behind it.
        pub(crate) fn spawn(self, answer: Answer, inflight: Arc<Inflight>) {
            let Listener {
                runtime,
                mut first,
                name,
            } = self;
            std::thread::spawn(move || {
                runtime.block_on(async move {
                    loop {
                        let connected = first.connect().await;
                        let next = match instance(&name, false) {
                            Ok(next) => next,
                            Err(e) => {
                                eprintln!("[remote] control pipe: {e}");
                                return;
                            }
                        };
                        let pipe = std::mem::replace(&mut first, next);
                        if connected.is_err() {
                            continue;
                        }
                        let answer = answer.clone();
                        let guard = Inflight::enter(&inflight);
                        tokio::spawn(async move {
                            serve(pipe, answer).await;
                            drop(guard);
                        });
                    }
                });
            });
        }
    }

    /// Reads one command line, has the loop answer it, writes the reply with
    /// its end marker, waits for the client to hang up, then disconnects,
    /// so closing the pipe cannot discard a reply that has not been read yet.
    async fn serve(pipe: NamedPipeServer, answer: Answer) {
        let mut reader = BufReader::new(pipe);
        let mut line = String::new();
        if (&mut reader)
            .take(MAX_LINE)
            .read_line(&mut line)
            .await
            .is_err()
        {
            return;
        }
        let Ok(reply) = tokio::task::spawn_blocking(move || answer(&line)).await else {
            return;
        };
        let pipe = reader.get_mut();
        let mut bytes = format!("{reply}\n").into_bytes();
        bytes.push(END);
        if pipe.write_all(&bytes).await.is_err() {
            return;
        }
        let _ = pipe.flush().await;
        let _ = pipe.shutdown().await;
        let _ = tokio::time::timeout(HANGUP_WAIT, async {
            let mut rest = [0u8; 64];
            while let Ok(1..) = pipe.read(&mut rest).await {}
        })
        .await;
        let _ = pipe.disconnect();
    }

    pub(crate) fn cleanup(_control: &Path) {}

    pub(crate) struct Connection(File);

    /// Opens the host's pipe, waiting a moment while its instances are busy.
    pub(crate) fn connect(control: &Path) -> std::io::Result<Connection> {
        let name = pipe_name(control)?;
        let mut tries = 0;
        loop {
            match File::options().read(true).write(true).open(&name) {
                Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY) && tries < BUSY_TRIES => {
                    tries += 1;
                    std::thread::sleep(BUSY_WAIT);
                }
                result => return result.map(Connection),
            }
        }
    }

    impl Connection {
        /// Sends one command line and reads the reply up to its end marker.
        pub(crate) fn request(mut self, line: &str) -> std::io::Result<String> {
            self.0.write_all(line.as_bytes())?;
            self.0.write_all(b"\n")?;
            read_reply(&self.0, Some(END))
        }
    }
}

/// Connections that have not written their reply yet, so the exit can let
/// the last one (`quit`'s own) reach its client.
#[derive(Default)]
pub(crate) struct Inflight {
    count: Mutex<usize>,
    idle: Condvar,
}

pub(crate) struct InflightGuard(Arc<Inflight>);

impl Inflight {
    fn enter(this: &Arc<Self>) -> InflightGuard {
        *this.count.lock().expect("inflight count") += 1;
        InflightGuard(this.clone())
    }

    pub(crate) fn drain(&self, limit: Duration) {
        let count = self.count.lock().expect("inflight count");
        let _ = self
            .idle
            .wait_timeout_while(count, limit, |count| *count > 0);
    }
}

impl Drop for InflightGuard {
    fn drop(&mut self) {
        *self.0.count.lock().expect("inflight count") -= 1;
        self.0.idle.notify_all();
    }
}
