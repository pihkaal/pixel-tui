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
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 10;

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

// TODO: allow to render outside of the screen
fn render_filled_board_cell(x: u16, y: u16, color: Color) -> io::Result<()> {
    let mut stdout = io::stdout();

    queue!(
        stdout,
        cursor::MoveTo(x, y),
        style::SetBackgroundColor(color),
        style::Print("      "),
        cursor::MoveTo(x, y + 1),
        style::Print("      "),
        cursor::MoveTo(x, y + 2),
        style::Print("      "),
        style::ResetColor
    )?;

    Ok(())
}

fn render_board_cell(x: u16, y: u16, number: u8, first_col: bool) -> io::Result<()> {
    render_cell(
        x,
        y,
        Color::Rgb {
            r: 143,
            g: 149,
            b: 170,
        },
        Color::White,
        None,
        number,
        first_col,
    )
}

fn render_cell(
    x: u16,
    y: u16,
    zero_foreground_color: Color,
    foreground_color: Color,
    background_color: Option<Color>,
    number: u8,
    first_col: bool,
) -> io::Result<()> {
    const BORDER_COLOR: Color = Color::Black;
    let mut stdout = io::stdout();

    if first_col && x > 0 {
        queue!(
            stdout,
            cursor::MoveTo(x - 1, y),
            style::SetForegroundColor(BORDER_COLOR),
            style::Print("▕"),
            cursor::MoveTo(x - 1, y + 1),
            style::Print("▕"),
            cursor::MoveTo(x - 1, y + 2),
            style::Print("▕"),
        )?;
    }

    if let Some(background_color) = background_color {
        queue!(stdout, style::SetBackgroundColor(background_color))?;
    }

    queue!(
        stdout,
        cursor::MoveTo(x, y),
        style::SetForegroundColor(BORDER_COLOR),
        style::Print("🭽▔▔▔▔🭾"),
        cursor::MoveTo(x, y + 1),
        style::Print("▏ "),
    )?;

    if number < 9 {
        queue!(
            stdout,
            style::SetForegroundColor(zero_foreground_color),
            style::Print("0"),
            style::SetForegroundColor(foreground_color),
            style::Print(format!("{}", number + 1)),
        )?;
    } else {
        queue!(
            stdout,
            style::SetForegroundColor(foreground_color),
            style::Print(format!("{}", number + 1)),
        )?;
    }

    queue!(
        stdout,
        style::SetForegroundColor(BORDER_COLOR),
        style::Print(" ▕"),
        cursor::MoveTo(x, y + 2),
        style::Print("🭼▁▁▁▁🭿"),
        style::ResetColor,
        //
        style::SetForegroundColor(BORDER_COLOR),
        cursor::MoveTo(x + 6, y),
        style::Print("▏"),
        cursor::MoveTo(x + 6, y + 1),
        style::Print("▏"),
        cursor::MoveTo(x + 6, y + 2),
        style::Print("▏"),
        style::ResetColor,
    )?;

    Ok(())
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

            if brightness > 0.5 {
                render_cell(
                    x,
                    y,
                    RGB::new(25, 25, 25).to_color(),
                    RGB::new(0, 0, 0).to_color(),
                    Some(color.to_color()),
                    color_index,
                    false,
                )?;
            } else {
                render_cell(
                    x,
                    y,
                    RGB::new(229, 229, 229).to_color(),
                    RGB::new(255, 255, 255).to_color(),
                    Some(color.to_color()),
                    color_index,
                    false,
                )?;
            }

            Ok(())
        };

        const CELL_WIDTH: u16 = 6;
        const CELL_HEIGHT: u16 = 3;
        const CELLS_PER_ROW: u16 = 5;
        const CELLS_PER_COL: u16 = 2;
        const TOTAL_WIDTH: u16 = CELL_WIDTH * CELLS_PER_ROW as u16;

        let x = (size.0 - TOTAL_WIDTH) / 2;
        let y = size.1 - 2 * CELL_HEIGHT - 1;

        for oy in 1..(CELLS_PER_ROW * CELL_HEIGHT) {
            queue!(
                stdout,
                cursor::MoveTo(x - 1, y + oy),
                style::SetForegroundColor(Color::Black),
                style::Print("█"),
            )?;
        }

        for row in 0..CELLS_PER_COL {
            for col in 0..CELLS_PER_ROW {
                draw_cell(
                    x + col * CELL_WIDTH,
                    y + row * CELL_HEIGHT + 1,
                    (row * CELLS_PER_ROW + col) as u8,
                )?;
            }
        }

        for oy in 1..(CELLS_PER_ROW * CELL_HEIGHT) {
            queue!(
                stdout,
                cursor::MoveTo(x + TOTAL_WIDTH, y + oy),
                style::SetForegroundColor(Color::Black),
                style::Print("█"),
            )?;
        }

        queue!(
            stdout,
            cursor::MoveTo(x - 1, size.1 - 7),
            style::SetForegroundColor(Color::Black),
            style::Print("🬭".repeat(TOTAL_WIDTH as usize + 2)),
        )?;

        // arrows
        let ax = x + TOTAL_WIDTH + 3;
        let ay = size.1 - 5;
        queue!(
            stdout,
            cursor::MoveTo(ax, ay + 1),
            style::Print("▀▄"),
            cursor::MoveTo(ax, ay + 2),
            style::Print(" ▄▀"),
            cursor::MoveTo(ax, ay + 3),
            style::Print("▀"),
        )?;

        let ax = x - 6;
        let ay = size.1 - 5;
        queue!(
            stdout,
            cursor::MoveTo(ax, ay + 1),
            style::Print(" ▄▀"),
            cursor::MoveTo(ax, ay + 2),
            style::Print("▀▄  "),
            cursor::MoveTo(ax, ay + 3),
            style::Print("  ▀"),
            style::ResetColor,
        )?;

        Ok(())
    }
}

struct Board {
    x: i16,
    y: i16,

    pixels: [[Pixel; BOARD_WIDTH]; BOARD_HEIGHT],
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
            }; BOARD_WIDTH]; BOARD_HEIGHT],
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
        let size = terminal::size()?;

        let mut first_cx: Option<u16> = None;

        for px in 0..self.width() {
            for py in 0..self.height() {
                let cx = self.x + 6 * px as i16;
                let cy = self.y + 3 * py as i16;

                if cx < 0 || cx + 6 >= size.0 as i16 || cy < 0 || cy + 2 >= size.1 as i16 {
                    continue;
                }

                let cx = cx as u16;
                let cy = cy as u16;

                if first_cx.is_none() {
                    first_cx = Some(cx);
                }

                let pixel = self.get(px, py);
                let is_first_col = first_cx.unwrap() == cx;
                if pixel.filled {
                    render_filled_board_cell(cx, cy, self.palette.get_color(pixel.color))?;
                } else {
                    render_board_cell(cx, cy, pixel.color, is_first_col)?;
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

    let mut p = (0, 0);

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
                        let px = (event.column as i16 - board.x) / 6;
                        let py = (event.row as i16 - board.y) / 3;

                        if !board.contains(px, py) {
                            continue;
                        }

                        p = (px, py);
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
                "::: bx: {}, by: {} ::: c: {} ::: px: {}, py: {} :::",
                board.x,
                board.y,
                current_color + 1,
                p.0,
                p.1
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
