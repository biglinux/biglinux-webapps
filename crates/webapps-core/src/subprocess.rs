use std::ffi::{OsStr, OsString};
use std::io;
use std::process::{Command, Output, Stdio};

#[derive(Debug, Clone)]
pub struct SubprocessSpec {
    program: OsString,
    args: Vec<OsString>,
}

#[derive(Debug, Default)]
pub struct SubprocessSpecBuilder {
    program: Option<OsString>,
    args: Vec<OsString>,
}

impl SubprocessSpec {
    pub fn builder() -> SubprocessSpecBuilder {
        SubprocessSpecBuilder::default()
    }

    pub fn run(&self) -> io::Result<Output> {
        self.command().output()
    }

    pub fn spawn_detached(&self) -> io::Result<()> {
        self.command()
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
    }

    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command.args(&self.args);
        command
    }
}

impl SubprocessSpecBuilder {
    pub fn program(mut self, program: impl AsRef<OsStr>) -> Self {
        self.program = Some(program.as_ref().to_os_string());
        self
    }

    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_os_string());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|arg| arg.as_ref().to_os_string()));
        self
    }

    pub fn build(self) -> SubprocessSpec {
        SubprocessSpec {
            program: self.program.unwrap_or_default(),
            args: self.args,
        }
    }
}
