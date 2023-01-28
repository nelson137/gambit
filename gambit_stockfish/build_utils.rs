use std::{fmt, process::Command};

use shell_escape::escape;

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
