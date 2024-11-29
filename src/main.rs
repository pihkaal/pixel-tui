use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

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

const SUBSCRIPTS: [char; 10] = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
const SUPERSCRIPTS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];

fn to_superscript(n: u8) -> String {
    let str: String = n
        .to_string()
        .chars()
        .map(|c| SUPERSCRIPTS[c.to_digit(10).unwrap() as usize])
        .collect();

    if str.len() == 3 {
        format!(" {}", str)
    } else {
        str
    }
}

fn to_subscript(n: u8) -> String {
    let str: String = n
        .to_string()
        .chars()
        .map(|c| SUBSCRIPTS[c.to_digit(10).unwrap() as usize])
        .collect();

    if str.len() == 3 {
        format!(" {}", str)
    } else {
        str
    }
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

    const PIXEL: Pixel = Pixel {
        color: 0,
        filled: false,
    };
    let mut board = [[PIXEL; 1000]; 1000];

    let palette = [Color::Red, Color::Green, Color::Blue];

    let mut current_color: u8 = 0;

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
                        board[event.row as usize][(event.column - event.column % 2) as usize]
                            .filled = true;
                        board[event.row as usize][(event.column - event.column % 2) as usize]
                            .color = current_color;
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        current_color = (current_color + 1) % (palette.len() as u8);
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // render
        queue!(stdout, terminal::Clear(ClearType::All))?;

        let size = terminal::size()?;
        for row in 0..size.1 {
            for col in (0..size.0).step_by(2) {
                let pixel = board[row as usize][col as usize];
                queue!(stdout, cursor::MoveTo(col, row))?;
                if pixel.filled {
                    let color = palette[pixel.color as usize];
                    queue!(
                        stdout,
                        style::SetBackgroundColor(color),
                        style::Print("  "),
                        style::ResetColor
                    )?;
                } else {
                    if (col / 2) % 2 == 1 {
                        queue!(stdout, style::Print(to_subscript(pixel.color)))?;
                    } else {
                        queue!(stdout, style::Print(to_superscript(pixel.color)))?;
                    }
                }
            }
        }

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
