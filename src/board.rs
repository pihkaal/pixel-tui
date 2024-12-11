// TODO: single char top/bottom two different colors for bigger cells

use std::{any::type_name, fs, io, iter::Peekable, str::FromStr};

use crossterm::{cursor, event::MouseButton, queue, style, terminal};

use crate::{
    input::Input,
    palette::{Palette, RGB},
};

pub struct BoardDataColor {
    pub rgb: RGB,
    pub count: u32,
}

pub struct BoardData {
    pub width: usize,
    pub height: usize,

    pub pixels: Vec<Vec<u8>>,
    pub colors: Vec<BoardDataColor>,
}

impl BoardData {
    pub fn from_pixels(width: usize, height: usize, image_pixels: Vec<RGB>) -> Self {
        let mut colors: Vec<BoardDataColor> = Vec::new();
        let mut pixels: Vec<Vec<u8>> = vec![vec![0; width]; height];

        for y in 0..height {
            for x in 0..width {
                let pixel = image_pixels[y * width + x];
                if let Some(color_index) = colors.iter().position(|c| c.rgb == pixel) {
                    pixels[y][x] = color_index as u8;
                    colors[color_index].count += 1;
                } else {
                    colors.push(BoardDataColor {
                        rgb: pixel,
                        count: 1,
                    });
                    pixels[y][x] = (colors.len() - 1) as u8;
                }
            }
        }

        Self {
            width,
            height,
            pixels,
            colors,
        }
    }

    fn parse_next_token<'a, T: FromStr>(
        tokens: &mut Peekable<impl Iterator<Item = &'a str>>,
        expected: &str,
    ) -> Result<T, String> {
        if let Some(token) = tokens.next() {
            if let Ok(parsed_token) = token.parse::<T>() {
                Ok(parsed_token)
            } else {
                Err(format!(
                    "Invalid {expected} received, expected {} but got {token}",
                    type_name::<T>()
                ))
            }
        } else {
            Err(format!("Expected {expected}, but got EOF"))
        }
    }

    // TODO: handle errors
    pub fn from_ppm_file(file_path: &str) -> io::Result<Self> {
        let mut pixels: Vec<RGB> = Vec::new();

        let tokens = fs::read_to_string(file_path)?
            .split('\n')
            .filter(|l| !l.starts_with('#'))
            .map(|l| l.to_string())
            .collect::<Vec<String>>()
            .join(" ");
        let mut tokens = tokens.split_whitespace().peekable();

        tokens.next(); // P3
        let width = Self::parse_next_token::<usize>(&mut tokens, "u16 width").unwrap();
        let height = Self::parse_next_token::<usize>(&mut tokens, "u16 height").unwrap();
        let _max_color = Self::parse_next_token::<u8>(&mut tokens, "u8 max_color").unwrap();

        for _ in 0..width * height {
            let r = Self::parse_next_token::<u8>(&mut tokens, "u8 red").unwrap();
            let g = Self::parse_next_token::<u8>(&mut tokens, "u8 green").unwrap();
            let b = Self::parse_next_token::<u8>(&mut tokens, "u8 blue").unwrap();

            pixels.push(RGB { r, g, b });
        }

        Ok(Self::from_pixels(width, height, pixels))
    }
}

#[derive(Clone, Copy)]
pub struct Cell {
    pub color: u8,
    pub filled: bool,
}

pub struct Board {
    pub x: i16,
    pub y: i16,

    pub cells: Vec<Vec<Cell>>,
    pub width: usize,
    pub height: usize,
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

    pub fn new(data: BoardData) -> Self {
        let mut cells = vec![
            vec![
                Cell {
                    color: 0,
                    filled: false
                };
                data.width
            ];
            data.height
        ];

        for py in 0..data.height {
            for px in 0..data.width {
                cells[py][px].color = data.pixels[py][px];
            }
        }

        Self {
            x: 0,
            y: 0,

            width: data.width,
            height: data.height,
            cells,
            palette: Palette::new(data.colors),
        }
    }

    pub fn width(&self) -> u16 {
        self.cells[0].len() as u16
    }

    pub fn height(&self) -> u16 {
        self.cells.len() as u16
    }

    pub fn get(&self, px: u16, py: u16) -> Cell {
        self.cells[py as usize][px as usize]
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
                let cell = &mut self.cells[py as usize][px as usize];
                if !cell.filled && cell.color == self.palette.selected_color {
                    cell.filled = true;
                    self.palette.colors[self.palette.selected_color as usize].painted += 1;
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

                let cell = self.get(px, py);
                let mut stdout = io::stdout();

                if cell.filled {
                    queue!(
                        stdout,
                        cursor::MoveTo(cx, cy),
                        style::SetBackgroundColor(self.palette.get_color(cell.color)),
                        style::Print("  "),
                        style::ResetColor
                    )?;
                // TODO: shift palette
                } else if cell.color < 9 {
                    queue!(
                        stdout,
                        cursor::MoveTo(cx, cy),
                        style::Print(CELL_NUMBERS[cell.color as usize]),
                    )?;
                }
            }
        }

        Ok(())
    }
}
