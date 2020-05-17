use anyhow::Result;
use termwiz::{
    caps::Capabilities,
    color::*,
    input::*,
    surface::*,
    terminal::{buffered::BufferedTerminal, SystemTerminal, Terminal},
};

pub struct Editor {
    bt: BufferedTerminal<SystemTerminal>,
    should_quit: bool,
    buffer: Buffer,
}

pub struct Buffer {
    roff: usize,
    coff: usize,
    cx: usize,
    cy: usize,
    w: usize,
    h: usize,
    lines: Vec<Vec<char>>,
}

impl Default for Buffer {
    fn default() -> Self {
        Self {
            roff: 0,
            coff: 0,
            cx: 0,
            cy: 0,
            w: 0,
            h: 0,
            lines: vec![vec![]],
        }
    }
}

impl Editor {
    pub fn new() -> Result<Self> {
        let buf = BufferedTerminal::new(SystemTerminal::new(Capabilities::new_from_env()?)?)?;

        let mut buffer = Buffer::default();
        let (w, h) = buf.dimensions();

        buffer.w = w;
        buffer.h = h;
        Ok(Self {
            bt: buf,
            should_quit: false,
            buffer,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.bt.terminal().enter_alternate_screen()?;
        self.bt.terminal().set_raw_mode()?;
        self.bt.flush()?;

        loop {
            self.draw_screen();
            self.bt.flush()?;

            self.handle_keys()?;
            if self.should_quit {
                break;
            }
        }

        self.bt.flush()?;

        Ok(())
    }

    fn draw_screen(&mut self) {
        self.bt.add_changes(vec![
            Change::CursorShape(CursorShape::Hidden),
            Change::ClearScreen(ColorAttribute::Default),
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(0),
            },
        ]);

        self.bt.add_changes(vec![
            Change::CursorPosition {
                x: Position::Absolute(0),
                y: Position::Absolute(0),
            },
            Change::CursorShape(CursorShape::Default),
        ]);
    }

    fn handle_keys(&mut self) -> Result<()> {
        match self.bt.terminal().poll_input(None) {
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
