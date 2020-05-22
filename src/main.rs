use anyhow::Result;
use std::cmp::{max, min};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
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

        if c == '\n' {
            let new_line = self.line().drain(cx..).collect();
            self.lines.insert(self.cy + 1, new_line);
            self.move_caret(1, -(self.cy as i32));
        } else {
            self.line().insert(cx, c);
            self.move_caret(0, 1);
        }
    }

    pub fn backspace(&mut self) {
        let (cx, cy) = (self.cx, self.cy);

        if cx == 0 && cy != 0 {
            let line = self.lines.remove(cy);
            self.move_caret(-1, 0);
            let len = self.line().len() as i32 - cx as i32;
            self.move_caret(0, len);
            self.line().extend(line.iter());
        } else if cx != 0 {
            self.line().remove(cx - 1);
            self.move_caret(0, -1);
        }
    }

    pub fn delete(&mut self) {
        let (cx, cy) = (self.cx, self.cy);

        if cx == self.line().len() && self.cy != self.lines.len() - 1 {
            let line = self.lines.remove(cy + 1);
            self.line().extend(line.iter());
        } else if cx != self.line().len() {
            self.line().remove(cx);
        }
    }

    pub fn move_caret(&mut self, row: i32, col: i32) {
        let num_lines = self.lines.len() as i32;
        self.cy = min(max(self.cy as i32 + row, 0), num_lines - 1) as usize;
        if self.cy < self.roff {
            self.roff = self.cy;
        } else if self.cy > self.roff + (self.h as usize - 2) {
            self.roff = self.cy - (self.h as usize - 2);
        }

        let line_len = self.line().len() as i32;
        self.cx = min(max(self.cx as i32 + col, 0), line_len) as usize;
        if self.cx < self.coff {
            self.coff = self.cx;
        } else if self.cx > self.coff + (self.w as usize - 1) {
            self.coff = self.cx - (self.w as usize - 1);
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

    pub fn open(&mut self, path: PathBuf) -> Result<()> {
        let file = File::open(path)?;
        self.buffer.lines = BufReader::new(file)
            .lines()
            .map(|l| l.unwrap().chars().collect())
            .collect();
        Ok(())
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
                x: Position::Absolute(self.buffer.cx),
                y: Position::Absolute(self.buffer.cy),
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
                    key: KeyCode::Tab, ..
                }) => {
                    for _ in 0..4 {
                        self.buffer.push(' ');
                    }
                }
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Char(c),
                    ..
                }) => {
                    self.buffer.push(c);
                }
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Enter,
                    ..
                }) => {
                    self.buffer.push('\n');
                }
                InputEvent::Key(KeyEvent {
                    key: KeyCode::LeftArrow,
                    ..
                }) => self.buffer.move_caret(0, -1),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::RightArrow,
                    ..
                }) => self.buffer.move_caret(0, 1),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::UpArrow,
                    ..
                }) => self.buffer.move_caret(-1, 0),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::DownArrow,
                    ..
                }) => self.buffer.move_caret(1, 0),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Backspace,
                    ..
                }) => self.buffer.backspace(),
                InputEvent::Key(KeyEvent {
                    key: KeyCode::Delete,
                    ..
                }) => self.buffer.delete(),
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
    let mut args: Vec<String> = std::env::args().collect();
    if args.len() == 2 {
        editor.open(PathBuf::from(args.remove(1)))?;
    } else {
        println!("Error: too many arguments");
        println!("usage wilo [FILE]");
        return Ok(());
    }
    editor.run()?;
    Ok(())
}
