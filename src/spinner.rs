use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Braille spinner frames, matching the sequence used by `install.sh`.
pub const SPINNER_FRAMES: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// A terminal spinner shared by every long-running CLI step (search,
/// update, uninstall). Redraws with a plain `\r` overwrite rather than a
/// progress-bar library, matching `install.sh`'s approach.
pub struct Spinner {
    message: Arc<Mutex<String>>,
    running: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    finished: bool,
    quiet: bool,
}

impl Spinner {
    /// Starts an animated spinner with the given initial message.
    pub fn new(message: impl Into<String>) -> Self {
        Self::start(message, false)
    }

    /// Returns a no-op spinner that never prints anything.
    pub fn hidden() -> Self {
        Self::start(String::new(), true)
    }

    fn start(message: impl Into<String>, quiet: bool) -> Self {
        let message = Arc::new(Mutex::new(message.into()));
        let running = Arc::new(AtomicBool::new(true));

        let handle = if quiet {
            None
        } else {
            let message = Arc::clone(&message);
            let running = Arc::clone(&running);
            Some(std::thread::spawn(move || {
                use std::io::Write as _;

                let mut frame = 0;
                while running.load(Ordering::Relaxed) {
                    let text = message.lock().unwrap().clone();
                    let mut stderr = std::io::stderr();
                    let _ = write!(
                        stderr,
                        "\r\x1b[2K{} {}",
                        SPINNER_FRAMES[frame % SPINNER_FRAMES.len()],
                        text
                    );
                    let _ = stderr.flush();
                    frame += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }))
        };

        Self {
            message,
            running,
            handle,
            finished: false,
            quiet,
        }
    }

    /// Updates the message shown next to the spinner.
    pub fn set_message(&self, message: impl Into<String>) {
        if let Ok(mut guard) = self.message.lock() {
            *guard = message.into();
        }
    }

    /// Stops the spinner and clears its line, printing nothing further.
    pub fn finish_and_clear(&mut self) {
        self.stop();
        if !self.quiet {
            eprint!("\r\x1b[2K");
            let _ = std::io::Write::flush(&mut std::io::stderr());
        }
    }

    /// Stops the spinner and replaces its line with `message`.
    pub fn finish_with_message(&mut self, message: impl Into<String>) {
        self.stop();
        if !self.quiet {
            eprintln!("\r\x1b[2K{}", message.into());
        }
    }

    fn stop(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        self.stop();
    }
}
