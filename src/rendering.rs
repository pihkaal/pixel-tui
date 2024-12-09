use std::io;

use crossterm::{
    cursor, queue,
    style::{self, Color},
};

// TODO: allow to render outside of the screen
pub fn render_filled_board_cell(x: u16, y: u16, color: Color) -> io::Result<()> {
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

pub fn render_board_cell(x: u16, y: u16, number: u8, first_col: bool) -> io::Result<()> {
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

pub fn render_cell(
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
