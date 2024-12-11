use std::io;

use crossterm::{
    cursor,
    event::MouseButton,
    queue,
    style::{self, Color},
    terminal,
};

use crate::{
    board::BoardDataColor,
    input::{Input, Rect},
};

#[derive(Clone, Copy)]
pub struct PaletteColor {
    r: u8,
    g: u8,
    b: u8,
    pub painted: u32,
    count: u32,
}

pub struct Palette {
    pub colors: Vec<PaletteColor>,

    pub selected_color: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn to_color(&self) -> Color {
        Color::Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

impl From<PaletteColor> for Color {
    fn from(color: PaletteColor) -> Self {
        Color::Rgb {
            r: color.r,
            g: color.g,
            b: color.b,
        }
    }
}

impl Palette {
    pub fn new(colors: Vec<BoardDataColor>) -> Self {
        Self {
            colors: colors
                .iter()
                .map(|c| PaletteColor {
                    r: c.rgb.r,
                    g: c.rgb.g,
                    b: c.rgb.b,
                    painted: 0,
                    count: c.count,
                })
                .collect::<Vec<_>>(),
            selected_color: 0,
        }
    }

    pub fn get_color(&self, index: u8) -> Color {
        self.colors[index as usize].into()
    }

    pub fn update(&mut self, input: &Input) -> io::Result<()> {
        const CELL_WIDTH: u16 = 6;
        const CELL_HEIGHT: u16 = 3;
        const CELLS_PER_ROW: u16 = 5;
        const CELLS_PER_COL: u16 = 2;
        const TOTAL_WIDTH: u16 = CELL_WIDTH * CELLS_PER_ROW as u16;

        let size = terminal::size()?;
        let x = (size.0 - TOTAL_WIDTH) / 2;
        let y = size.1 - 2 * CELL_HEIGHT - 1;

        for row in 0..CELLS_PER_COL {
            for col in 0..CELLS_PER_ROW {
                if input.is_mouse_button_down_in(
                    MouseButton::Left,
                    Rect {
                        x: (x + col * CELL_WIDTH) as i16,
                        y: (y + row * CELL_HEIGHT + 1) as i16,
                        width: CELL_WIDTH,
                        height: CELL_HEIGHT,
                    },
                ) {
                    self.selected_color = (row * CELLS_PER_ROW + col) as u8;
                }
            }
        }

        Ok(())
    }

    fn render_cell(&self, x: u16, y: u16, color_index: u8) -> io::Result<()> {
        const BORDER_COLOR: Color = Color::Black;
        let color = self.colors[color_index as usize];
        let brightness =
            (0.299 * color.r as f32 + 0.587 * color.g as f32 + 0.114 * color.b as f32) / 255.0;

        let mut stdout = io::stdout();

        let background_color = color.into();
        let (zero_foreground_color, foreground_color) = if brightness > 0.5 {
            (
                RGB::new(25, 25, 25).to_color(),
                RGB::new(0, 0, 0).to_color(),
            )
        } else {
            (
                RGB::new(229, 229, 229).to_color(),
                RGB::new(255, 255, 255).to_color(),
            )
        };

        queue!(
            stdout,
            cursor::MoveTo(x, y),
            style::SetBackgroundColor(background_color),
            style::SetForegroundColor(BORDER_COLOR),
            style::Print("🭽▔▔▔▔🭾"),
            cursor::MoveTo(x, y + 1),
            style::Print("▏ "),
        )?;

        if color_index < 9 {
            queue!(
                stdout,
                style::SetForegroundColor(zero_foreground_color),
                style::Print("0"),
                style::SetForegroundColor(foreground_color),
                style::Print(format!("{}", color_index + 1)),
            )?;
        } else {
            queue!(
                stdout,
                style::SetForegroundColor(foreground_color),
                style::Print(format!("{}", color_index + 1)),
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
        )?;

        if self.selected_color == color_index {
            let fill = (6.0 * color.painted as f32 / color.count as f32).round() as usize;
            // i'm too tired do this properly, at least it works for now
            let fill = if fill == 0 {
                0
            } else if fill == 1 {
                1
            } else if fill == 2 || fill == 3 {
                2
            } else if fill == 4 || fill == 5 {
                3
            } else {
                4
            };

            queue!(
                stdout,
                cursor::MoveTo(x + 1, y + 2),
                style::SetBackgroundColor(background_color),
                style::Print("\u{ee00}\u{ee01}\u{ee01}\u{ee02}"),
                cursor::MoveTo(x + 1, y + 2),
                style::Print(&"\u{ee03}\u{ee04}\u{ee04}\u{ee05}"[0..fill * 3]),
            )?;
        }

        queue!(stdout, style::ResetColor)?;

        Ok(())
    }

    pub fn render(&self) -> io::Result<()> {
        let mut stdout = io::stdout();
        let size = terminal::size()?;

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
                /*
                draw_cell(
                    x + col * CELL_WIDTH,
                    y + row * CELL_HEIGHT + 1,
                    (row * CELLS_PER_ROW + col) as u8,
                )?;
                */
                let color_index = (row * CELLS_PER_ROW + col) as u8;
                if color_index as usize >= self.colors.len() {
                    break;
                }

                self.render_cell(x + col * CELL_WIDTH, y + row * CELL_HEIGHT + 1, color_index)?;
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
