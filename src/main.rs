use anyhow::Result;
use std::cmp::min;
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

impl Buffer {
    fn line(&mut self) -> &mut Vec<char> {
        self.lines.get_mut(self.cy).unwrap()
    }

    pub fn push(&mut self, c: char) {
        let cx = self.cx;

        self.line().insert(cx, c);
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

        for i in self.buffer.roff..(self.buffer.roff + self.buffer.h - 1) {
            if i < self.buffer.lines.len() {
                let line = self.buffer.lines.get(i).unwrap();
                if line.len() < self.buffer.coff {
                    self.bt.add_change("\r\n");
                    continue;
                }

                let part =
                    &line[self.buffer.coff..min(self.buffer.coff + self.buffer.w, line.len())];
                self.bt
                    .add_change(&Vec::from(part).iter().collect::<String>());
                self.bt.add_change("\r\n");
            } else {
                self.bt.add_change("~\r\n");
            }
        }

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
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(c),
                    ..
                }) => {
                    self.buffer.push(c);
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
