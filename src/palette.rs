use std::io;

use crossterm::{
    cursor, queue,
    style::{self, Color},
    terminal,
};

use crate::rendering::render_cell;

pub struct Palette {
    colors: [RGB; 20],
}

pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
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

impl Palette {
    pub fn new() -> Self {
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

    pub fn get_color(&self, index: u8) -> Color {
        self.colors[index as usize].to_color()
    }

    pub fn count(&self) -> u8 {
        self.colors.len() as u8
    }

    pub fn render(&self) -> io::Result<()> {
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
