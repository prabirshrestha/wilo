use anyhow::Result;
use termwiz::{
    caps::Capabilities,
    terminal::{buffered::BufferedTerminal, SystemTerminal},
};

pub struct Editor {
    buf: BufferedTerminal<SystemTerminal>,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let buf = BufferedTerminal::new(SystemTerminal::new(Capabilities::new_from_env()?)?)?;
        Ok(Self { buf })
    }
}

fn main() -> Result<()> {
    let mut editor = Editor::new()?;
    editor.buf.add_change("Hello world\n");
    editor.buf.flush()?;
    Ok(())
}
