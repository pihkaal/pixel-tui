use std::io;

use crossterm::{cursor, event::MouseButton, queue, style, terminal};

use crate::{input::Input, palette::Palette};

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

const CELL_NUMBERS: [&str; 9] = [
    "\u{31}\u{FE0F}",
    "\u{32}\u{FE0F}",
    "\u{33}\u{FE0F}",
    "\u{34}\u{FE0F}",
    "\u{35}\u{FE0F}",
    "\u{36}\u{FE0F}",
    "\u{37}\u{FE0F}",
    "\u{38}\u{FE0F}",
    "\u{39}\u{FE0F}",
];

impl Board {
    pub const CELL_WIDTH: u16 = 2;
    pub const CELL_HEIGHT: u16 = 1;

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

    pub fn set(&mut self, px: u16, py: u16, pixel: Pixel) {
        self.pixels[py as usize][px as usize] = pixel;
    }

    pub fn contains(&self, px: i16, py: i16) -> bool {
        px >= 0 && px < self.width() as i16 && py >= 0 && py < self.height() as i16
    }

    pub fn update(&mut self, input: &Input) -> io::Result<()> {
        self.palette.update(input)?;

        if let Some(mouse_drag) = &input.mouse_drag {
            if mouse_drag.button == MouseButton::Middle {
                self.x += mouse_drag.offset_x as i16;
                self.y += mouse_drag.offset_y as i16;
            }
        }

        for frame_mouse in &input.frame_mouses {
            if frame_mouse.active_button != Some(MouseButton::Left) {
                continue;
            }

            let px = (frame_mouse.x as i16 - self.x) / Self::CELL_WIDTH as i16;
            let py = (frame_mouse.y as i16 - self.y) / Self::CELL_HEIGHT as i16;

            if self.contains(px, py) {
                let pixel = &mut self.pixels[py as usize][px as usize];
                if pixel.color == self.palette.selected_color {
                    pixel.filled = true;
                }
            }
        }

        Ok(())
    }

    pub fn render(&self) -> io::Result<()> {
        let size = terminal::size()?;

        let mut first_cx: Option<u16> = None;

        // TODO: intersect board rect with terminal rect
        for px in 0..self.width() {
            for py in 0..self.height() {
                let cx = self.x + Self::CELL_WIDTH as i16 * px as i16;
                let cy = self.y + Self::CELL_HEIGHT as i16 * py as i16;

                if cx < 0
                    || cx + Self::CELL_WIDTH as i16 >= size.0 as i16
                    || cy < 0
                    || cy + Self::CELL_HEIGHT as i16 - 1 >= size.1 as i16
                {
                    continue;
                }

                let cx = cx as u16;
                let cy = cy as u16;

                if first_cx.is_none() {
                    first_cx = Some(cx);
                }

                let pixel = self.get(px, py);
                let mut stdout = io::stdout();

                if pixel.filled {
                    queue!(
                        stdout,
                        cursor::MoveTo(cx, cy),
                        style::SetBackgroundColor(self.palette.get_color(pixel.color)),
                        style::Print("  "),
                        style::ResetColor
                    )?;
                } else {
                    queue!(
                        stdout,
                        cursor::MoveTo(cx, cy),
                        style::Print(CELL_NUMBERS[pixel.color as usize]),
                    )?;
                }
            }
        }

        Ok(())
    }
}
