use is_terminal::IsTerminal;

pub struct Output {
    stderr_is_tty: bool,
}

impl Output {
    pub fn new() -> Self {
        Self {
            stderr_is_tty: std::io::stderr().is_terminal(),
        }
    }

    pub fn error(&self, message: &str) {
        if self.stderr_is_tty {
            eprintln!("\x1b[31merror:\x1b[0m {message}");
        } else {
            eprintln!("error: {message}");
        }
    }
}
