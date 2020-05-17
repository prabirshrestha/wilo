use anyhow::Result;
use termwiz::{
    caps::Capabilities,
    terminal::{buffered::BufferedTerminal, SystemTerminal, Terminal},
};

pub struct Editor {
    buf: BufferedTerminal<SystemTerminal>,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let buf = BufferedTerminal::new(SystemTerminal::new(Capabilities::new_from_env()?)?)?;
        Ok(Self { buf })
    }

    pub fn run(&mut self) -> Result<()> {
        self.buf.terminal().enter_alternate_screen()?;
        self.buf.flush()?;

        self.buf.add_change("Hello world\n");
        self.buf.flush()?;

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut editor = Editor::new()?;
    editor.run()?;
    Ok(())
}
