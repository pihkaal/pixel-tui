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

struct Palette {
    colors: [RGB; 20],
}

struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn to_color(&self) -> Color {
        Color::Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

impl Palette {
    fn new() -> Self {
        Self {
            colors: [
                RGB::new(3, 104, 63),
                RGB::new(37, 19, 190),
                RGB::new(242, 55, 94),
                RGB::new(123, 223, 67),
                RGB::new(45, 78, 205),
                RGB::new(200, 120, 14),
                RGB::new(75, 192, 203),
                RGB::new(145, 20, 145),
                RGB::new(56, 34, 89),
                RGB::new(244, 67, 54),
                RGB::new(77, 99, 232),
                RGB::new(188, 65, 101),
                RGB::new(96, 231, 79),
                RGB::new(222, 190, 75),
                RGB::new(105, 135, 90),
                RGB::new(10, 220, 182),
                RGB::new(200, 40, 60),
                RGB::new(90, 24, 180),
                RGB::new(50, 200, 50),
                RGB::new(255, 165, 0),
            ],
        }
    }

    fn get_color(&self, index: u8) -> Color {
        self.colors[index as usize].to_color()
    }

    fn count(&self) -> u8 {
        self.colors.len() as u8
    }

    fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        let size = terminal::size()?;

        let draw_cell = |x: u16, y: u16, color_index: u8| -> io::Result<()> {
            let color = &self.colors[color_index as usize];
            let brightness =
                (0.299 * color.r as f32 + 0.587 * color.g as f32 + 0.114 * color.b as f32) / 255.0;

            let mut stdout = io::stdout();

            queue!(
                stdout,
                style::SetForegroundColor(Color::Black),
                style::SetBackgroundColor(color.to_color()),
                cursor::MoveTo(x, y),
                style::Print("█🬂🬂🬂🬂🬂🬂█"),
                cursor::MoveTo(x, y + 1),
                style::Print("█      █"),
                cursor::MoveTo(x, y + 2),
                style::Print("█🬭🬭🬭🬭🬭🬭█"),
                style::ResetColor
            )?;

            queue!(
                stdout,
                cursor::MoveTo(x + 3, y + 1),
                style::SetBackgroundColor(color.to_color()),
                style::SetForegroundColor(if brightness > 0.5 {
                    Color::Black
                } else {
                    Color::White
                }),
                style::Print(format!("{: >2}", color_index + 1)),
                style::ResetColor,
            )?;

            Ok(())
        };

        const CELL_WIDTH: u16 = 8;
        const CELL_HEIGHT: u16 = 3;
        const CELLS_PER_ROW: u8 = 5;
        const CELLS_PER_COL: u8 = 2;
        const TOTAL_WIDTH: u16 = CELL_WIDTH * CELLS_PER_ROW as u16;

        let x = (size.0 - TOTAL_WIDTH) / 2;
        let y = size.1 - 2 * CELL_HEIGHT - 2;

        queue!(
            stdout,
            cursor::MoveTo(x, y),
            style::SetForegroundColor(Color::Black),
            style::Print("🬭".repeat(TOTAL_WIDTH as usize))
        )?;

        for row in 0..CELLS_PER_COL {
            for col in 0..CELLS_PER_ROW {
                draw_cell(
                    x + col as u16 * CELL_WIDTH,
                    y + row as u16 * CELL_HEIGHT + 1,
                    row * CELLS_PER_ROW + col,
                )?;
            }
        }

        queue!(
            stdout,
            cursor::MoveTo(x, 1 + y + 2 * CELL_HEIGHT),
            style::SetForegroundColor(Color::Black),
            style::Print("🬂".repeat(TOTAL_WIDTH as usize)),
            style::ResetColor
        )?;

        Ok(())
    }
}

struct Board {
    x: i16,
    y: i16,

    pixels: [[Pixel; 75]; 75],
    palette: Palette,
}

impl Board {
    fn new() -> Self {
        Self {
            x: 0,
            y: 0,

            pixels: [[Pixel {
                color: 0,
                filled: false,
            }; 75]; 75],
            palette: Palette::new(),
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
                        style::SetBackgroundColor(self.palette.get_color(pixel.color)),
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
            let random_color = rand::random::<u8>() % board.palette.count();
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
                        current_color = (current_color + 1) % board.palette.count();
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
        board.palette.render()?;

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
