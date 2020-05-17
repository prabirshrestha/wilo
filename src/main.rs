use anyhow::Result;
use termwiz::{
    caps::Capabilities,
    input::*,
    terminal::{buffered::BufferedTerminal, SystemTerminal, Terminal},
};

pub struct Editor {
    buf: BufferedTerminal<SystemTerminal>,
    should_quit: bool,
}

impl Editor {
    pub fn new() -> Result<Self> {
        let buf = BufferedTerminal::new(SystemTerminal::new(Capabilities::new_from_env()?)?)?;
        Ok(Self {
            buf,
            should_quit: false,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.buf.terminal().enter_alternate_screen()?;
        self.buf.terminal().set_raw_mode()?;
        self.buf.flush()?;

        loop {
            self.handle_keys()?;
            if self.should_quit {
                break;
            }
        }

        self.buf.flush()?;

        Ok(())
    }

    fn handle_keys(&mut self) -> Result<()> {
        match self.buf.terminal().poll_input(None) {
            Ok(Some(input)) => match input {
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char('Q'),
                    modifiers: Modifiers::CTRL,
                }) => {
                    self.should_quit = true;
                }
                _ => {}
            },
            Ok(None) => {}
            Err(e) => {
                println!("{:?}\r\n", e);
                self.should_quit = true;
            }
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut editor = Editor::new()?;
    editor.run()?;
    Ok(())
}
