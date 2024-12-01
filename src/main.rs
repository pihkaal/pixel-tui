use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use rand;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, MouseButton, MouseEventKind},
    execute, queue,
    style::{self, Color},
    terminal::{self, ClearType},
};

const FPS: u64 = 60;

#[derive(Clone, Copy)]
struct Pixel {
    color: u8,
    filled: bool,
}

struct Board {
    x: i16,
    y: i16,

    pixels: [[Pixel; 10]; 10],
    palette: [Color; 3],
}

impl Board {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,

            pixels: [[Pixel {
                color: 0,
                filled: false,
            }; 10]; 10],
            palette: [Color::Red, Color::Green, Color::Blue],
        }
    }

    fn width(&self) -> u16 {
        self.pixels[0].len() as u16
    }

    fn height(&self) -> u16 {
        self.pixels.len() as u16
    }

    fn get(&self, px: u16, py: u16) -> Pixel {
        self.pixels[py as usize][px as usize]
    }

    fn get_mut(&mut self, px: u16, py: u16) -> &mut Pixel {
        &mut self.pixels[py as usize][px as usize]
    }

    fn set(&mut self, px: u16, py: u16, pixel: Pixel) {
        self.pixels[py as usize][px as usize] = pixel;
    }

    fn contains(&self, px: i16, py: i16) -> bool {
        px >= 0 && px < self.width() as i16 && py >= 0 && py < self.height() as i16
    }

    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        let size = terminal::size()?;

        for px in 0..self.width() {
            for py in 0..self.height() {
                let cx = self.x + 2 * px as i16;
                let cy = self.y + py as i16;

                if cx < 0 || cx + 1 >= size.0 as i16 || cy < 0 || cy >= size.1 as i16 {
                    continue;
                }

                queue!(stdout, cursor::MoveTo(cx as u16, cy as u16))?;

                let pixel = self.get(px, py);
                if pixel.filled {
                    queue!(
                        stdout,
                        style::SetBackgroundColor(self.palette[pixel.color as usize]),
                        style::Print("  "),
                        style::ResetColor
                    )?;
                } else {
                    if px % 2 == 1 {
                        queue!(stdout, style::Print(to_subscript(pixel.color + 1)))?;
                    } else {
                        queue!(stdout, style::Print(to_superscript(pixel.color + 1)))?;
                    }
                }
            }
        }

        Ok(())
    }
}

const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

fn to_superscript(n: u8) -> String {
    format!(
        "{: >2}",
        n.to_string()
            .chars()
            .map(|c| SUPERSCRIPTS[c.to_digit(10).unwrap() as usize])
            .collect::<String>(),
    )
}

fn to_subscript(n: u8) -> String {
    format!(
        "{: >2}",
        n.to_string()
            .chars()
            .map(|c| SUBSCRIPTS[c.to_digit(10).unwrap() as usize])
            .collect::<String>()
    )
}

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture,
        cursor::Hide
    )?;

    let mut board = Board::new();

    // randomize board
    for row in 0..board.height() {
        for col in 0..board.width() {
            let random_color = rand::random::<u8>() % 3;
            board.set(
                col,
                row,
                Pixel {
                    color: random_color,
                    filled: false,
                },
            );
        }
    }

    let mut current_color: u8 = 0;
    let mut drag_start = (0, 0);

    let mut quit = false;
    while !quit {
        let start = std::time::Instant::now();

        // update
        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(event) => match event.code {
                    KeyCode::Char('q') => {
                        quit = true;
                        break;
                    }
                    _ => {}
                },
                Event::Mouse(event) => match event.kind {
                    MouseEventKind::Drag(MouseButton::Left)
                    | MouseEventKind::Down(MouseButton::Left) => {
                        let px = (event.column as i16 - board.x) / 2;
                        let py = event.row as i16 - board.y;

                        if !board.contains(px, py) {
                            continue;
                        }

                        let pixel = board.get_mut(px as u16, py as u16);
                        if pixel.color == current_color {
                            pixel.filled = true;
                        }
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        current_color = (current_color + 1) % (board.palette.len() as u8);
                    }
                    MouseEventKind::Down(MouseButton::Middle) => {
                        drag_start = (event.column, event.row);
                    }
                    MouseEventKind::Drag(MouseButton::Middle) => {
                        board.x += event.column as i16 - drag_start.0 as i16;
                        board.y += event.row as i16 - drag_start.1 as i16;
                        drag_start = (event.column, event.row);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // render
        queue!(
            stdout,
            terminal::BeginSynchronizedUpdate,
            terminal::Clear(ClearType::All)
        )?;

        board.render()?;

        queue!(
            stdout,
            cursor::MoveTo(0, 0),
            style::Print(format!(
                "::: bx: {}, by: {} ::: c: {} :::",
                board.x, board.y, current_color
            )),
            terminal::EndSynchronizedUpdate
        )?;

        stdout.flush()?;

        let elapsed = start.elapsed();
        if elapsed < Duration::from_millis(1000 / FPS) {
            thread::sleep(Duration::from_millis(1000 / FPS) - elapsed);
        }
    }

    execute!(
        stdout,
        event::DisableMouseCapture,
        terminal::LeaveAlternateScreen,
        cursor::Show
    )?;
    terminal::disable_raw_mode()?;

    Ok(())
}
