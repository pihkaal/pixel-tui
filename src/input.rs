use std::{collections::HashSet, io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEventKind, MouseButton, MouseEventKind};

pub struct Input {
    pub mouse_x: u16,
    pub mouse_y: u16,

    pub frame_mouses: Vec<FrameMouse>,

    drag_start_x: u16,
    drag_start_y: u16,

    pub keys_pressed: HashSet<KeyCode>,
    pub mouse_buttons_pressed: HashSet<MouseButton>,
    pub mouse_drag: Option<MouseDrag>,
}

pub struct FrameMouse {
    pub x: u16,
    pub y: u16,
    pub active_button: Option<MouseButton>,
}

pub struct MouseDrag {
    pub offset_x: i16,
    pub offset_y: i16,
    pub button: MouseButton,
}

pub struct Rect {
    pub x: i16,
    pub y: i16,
    pub width: u16,
    pub height: u16,
}

impl Rect {
    pub fn contains(&self, px: i16, py: i16) -> bool {
        px >= self.x
            && px < self.x + self.width as i16
            && py >= self.y
            && py < self.y + self.height as i16
    }
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse_x: 0,
            mouse_y: 0,

            frame_mouses: Vec::new(),

            drag_start_x: 0,
            drag_start_y: 0,

            keys_pressed: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_drag: None,
        }
    }

    pub fn process_events(&mut self) -> io::Result<()> {
        // clear drag state and frame mouse positions
        self.mouse_drag = None;
        // we need to keep track of multiple mouse positions
        // per frame because we can receive multiple mouse events
        self.frame_mouses.clear();

        // store drag start position for current frame for the
        // same reason as above
        let mut frame_drag_start_x = self.drag_start_x;
        let mut frame_drag_start_y = self.drag_start_y;

        while event::poll(Duration::ZERO)? {
            match event::read()? {
                Event::Key(e) => match e.kind {
                    KeyEventKind::Press => {
                        self.keys_pressed.insert(e.code);
                    }
                    KeyEventKind::Release => {
                        self.keys_pressed.remove(&e.code);
                    }
                    _ => {}
                },
                Event::Mouse(e) => {
                    self.mouse_x = e.column;
                    self.mouse_y = e.row;

                    let mut frame_mouse = FrameMouse {
                        x: self.mouse_x,
                        y: self.mouse_y,
                        active_button: None,
                    };

                    match e.kind {
                        MouseEventKind::Down(b) => {
                            self.mouse_buttons_pressed.insert(b);

                            self.drag_start_x = self.mouse_x;
                            self.drag_start_y = self.mouse_y;

                            if self.mouse_drag.is_none() {
                                frame_drag_start_x = self.drag_start_x;
                                frame_drag_start_y = self.drag_start_y;
                            }

                            frame_mouse.active_button = Some(b);
                        }
                        MouseEventKind::Up(b) => {
                            self.mouse_buttons_pressed.remove(&b);

                            self.mouse_drag = None;
                        }
                        MouseEventKind::Drag(b) => {
                            self.mouse_drag = Some(MouseDrag {
                                offset_x: self.mouse_x as i16 - frame_drag_start_x as i16,
                                offset_y: self.mouse_y as i16 - frame_drag_start_y as i16,
                                button: b,
                            });

                            self.drag_start_x = self.mouse_x;
                            self.drag_start_y = self.mouse_y;

                            frame_mouse.active_button = Some(b);
                        }
                        _ => {}
                    };

                    self.frame_mouses.push(frame_mouse);
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.keys_pressed.contains(&key)
    }

    pub fn is_mouse_button_down_in(&self, button: MouseButton, rect: Rect) -> bool {
        self.mouse_buttons_pressed.contains(&button)
            && rect.contains(self.mouse_x as i16, self.mouse_y as i16)
    }
}
