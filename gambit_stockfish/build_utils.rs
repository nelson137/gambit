use std::{fmt, fs::File, io, process::Command, sync::Mutex};

use shell_escape::escape;
use tracing_subscriber::fmt::MakeWriter;

pub struct LogWriter {
    file: Mutex<File>,
}

impl LogWriter {
    pub fn new(file: File) -> Self {
        Self { file: Mutex::new(file) }
    }
}

impl<'writer> MakeWriter<'writer> for LogWriter {
    type Writer = &'writer Self;

    fn make_writer(&'writer self) -> Self::Writer {
        self
    }
}

impl io::Write for &LogWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        io::stderr().write_all(buf)?;
        let mut file = self.file.lock().unwrap();
        file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        io::stderr().flush()?;
        let mut file = self.file.lock().unwrap();
        file.flush()
    }
}

pub trait CommandExts {
    fn display(&self) -> CommandDisplay<'_>;
}

impl CommandExts for Command {
    fn display(&self) -> CommandDisplay {
        CommandDisplay(self)
    }
}

pub struct CommandDisplay<'cmd>(&'cmd Command);

impl<'cmd> fmt::Display for CommandDisplay<'cmd> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.0.get_program().to_string_lossy())?;

        for arg in self.0.get_args() {
            write!(f, " {}", escape(arg.to_string_lossy()))?;
        }

        Ok(())
    }
}
