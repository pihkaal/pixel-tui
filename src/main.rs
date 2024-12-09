use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, MouseButton, MouseEventKind},
    execute, queue, style,
    terminal::{self, ClearType},
};

use rand;

use crate::board::{Board, Pixel};

mod board;
mod palette;
mod rendering;

const FPS: u64 = 60;

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
