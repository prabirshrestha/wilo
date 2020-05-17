use anyhow::Result;
use termwiz::{
    caps::Capabilities,
    terminal::{buffered::BufferedTerminal, SystemTerminal},
};

fn main() -> Result<()> {
    let mut buf = BufferedTerminal::new(SystemTerminal::new(Capabilities::new_from_env()?)?)?;
    buf.add_change("Hello world\n");
    buf.flush()?;
    Ok(())
}
