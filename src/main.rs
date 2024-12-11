use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

use board::BoardData;
use crossterm::{
    cursor,
    event::{self, KeyCode, KeyboardEnhancementFlags},
    execute, queue,
    terminal::{self, ClearType},
};

use crate::{board::Board, input::Input};

mod board;
mod input;
mod palette;

const FPS: u64 = 60;

fn main() -> io::Result<()> {
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        event::EnableMouseCapture,
        event::PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES),
        cursor::Hide
    )?;

    let board_data = BoardData::from_ppm_file("images/test.ppm")?;
    let mut board = Board::new(board_data);

    let mut input = Input::new();

    let mut quit = false;
    while !quit {
        let start = std::time::Instant::now();

        // begin update
        input.process_events()?;

        if input.is_key_down(KeyCode::Char('q')) {
            quit = true;
        }

        board.update(&input)?;
        // end update

        // render
        queue!(
            stdout,
            terminal::BeginSynchronizedUpdate,
            terminal::Clear(ClearType::All)
        )?;

        board.render()?;
        board.palette.render()?;

        queue!(stdout, terminal::EndSynchronizedUpdate)?;
        stdout.flush()?;
        // end render

        let elapsed = start.elapsed();
        if elapsed < Duration::from_millis(1000 / FPS) {
            thread::sleep(Duration::from_millis(1000 / FPS) - elapsed);
        }
    }

    execute!(
        stdout,
        event::DisableMouseCapture,
        event::PopKeyboardEnhancementFlags,
        terminal::LeaveAlternateScreen,
        cursor::Show
    )?;
    terminal::disable_raw_mode()?;

    Ok(())
}
