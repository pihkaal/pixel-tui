use std::io;

use crossterm::terminal;

use crate::{
    palette::Palette,
    rendering::{render_board_cell, render_filled_board_cell},
};

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 10;

#[derive(Clone, Copy)]
pub struct Pixel {
    pub color: u8,
    pub filled: bool,
}

pub struct Board {
    pub x: i16,
    pub y: i16,

    pub pixels: [[Pixel; BOARD_WIDTH]; BOARD_HEIGHT],
    pub palette: Palette,
}

impl Board {
    pub fn new() -> Self {
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

    pub fn width(&self) -> u16 {
        self.pixels[0].len() as u16
    }

    pub fn height(&self) -> u16 {
        self.pixels.len() as u16
    }

    pub fn get(&self, px: u16, py: u16) -> Pixel {
        self.pixels[py as usize][px as usize]
    }

    pub fn get_mut(&mut self, px: u16, py: u16) -> &mut Pixel {
        &mut self.pixels[py as usize][px as usize]
    }

    pub fn set(&mut self, px: u16, py: u16, pixel: Pixel) {
        self.pixels[py as usize][px as usize] = pixel;
    }

    pub fn contains(&self, px: i16, py: i16) -> bool {
        px >= 0 && px < self.width() as i16 && py >= 0 && py < self.height() as i16
    }

    pub fn render(&self) -> io::Result<()> {
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
