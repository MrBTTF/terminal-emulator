use std::{
    sync::Mutex,
    sync::{mpsc, Weak},
};

use glutin::event::{ElementState, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::{graphics::cursor::Cursor, resources::Resources};

use super::{
    state::{Event, EventActor, EventReceiver, EventSender, InputEvent, OutputEvent},
    textdisplay::TextDisplay,
};

const GREEN: (u8, u8, u8, u8) = (0, 227, 48, 255);

pub struct TextField {
    textdisplay: TextDisplay,
    cursor: Cursor,
    input_position: u32,
    prefix: String,
    event_sender: Option<EventSender>,
    event_receiver: Option<Mutex<EventReceiver>>,
    user_input: Option<String>,
}

impl TextField {
    pub fn new(
        res: &Resources,
        gl: &gl::Gl,
        width: u32,
        height: u32,
    ) -> Result<TextField, anyhow::Error> {
        let color = GREEN;
        let textdisplay = TextDisplay::new(res, gl, width, height, color)?;
        let cursor = Cursor::new(
            res,
            gl,
            textdisplay.line_height as f32 * 0.6,
            textdisplay.line_height as f32,
            width as f32,
            height as f32,
            color,
        )?;

        anyhow::Ok(TextField {
            textdisplay,
            cursor,
            input_position: 0,
            prefix: String::new(),
            event_sender: None,
            event_receiver: None,
            user_input: None,
        })
    }

    fn cursor_on_first_subline(&self) -> bool {
        if let Some(last_line) = self.textdisplay.lines().last() {
            last_line.len() / self.textdisplay.get_line_width() == 0
        } else {
            false
        }
    }

    fn move_cursor_to_end(&mut self) {
        if self.textdisplay.get_line_width() == 0 {
            return;
        }
        if let Some(current_line) = self.textdisplay.lines().last() {
            // println!("line count: {}", self.textdisplay.get_lines_count());
            let mut y = self.textdisplay.get_lines_count().saturating_sub(1);
            if let Some(last_line) = self.textdisplay.lines().last() {
                if last_line.len() == self.textdisplay.get_line_width() {
                    y += 1;
                }
            }
            // println!("y: {}", y);
            self.cursor
                .move_to((current_line.len() % self.textdisplay.get_line_width()) as u32, y as u32);
        }
    }

    pub fn append(&mut self, s: &str) {
        self.textdisplay.append(s);
        self.move_cursor_to_end();
        if !self.cursor_on_first_subline() {
            self.input_position = 0;
        }
    }
    pub fn newline(&mut self) {
        self.textdisplay.newline();
        self.move_cursor_to_end();
    }

    pub fn enter(&mut self) {
        if let Some(line) = self.textdisplay.lines().last() {
            self.event_sender
                .as_ref()
                .unwrap()
                .send(Event::InputEvent(InputEvent::UserText(line.clone())));
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor.x >= self.input_position {
            self.textdisplay.remove_last_char();
            self.move_cursor_to_end();

            // if self.cursor_on_first_subline() {
            //     self.input_position = self.prefix.len() as u32 + 1;
            // }
        }
    }

    pub fn update_size(&mut self, width: i32, height: i32) {
        self.textdisplay.update_size(width, height);
        self.cursor.update_size(width as f32, height as f32);
        self.move_cursor_to_end();
    }

    pub fn render(&self, gl: &gl::Gl) {
        self.textdisplay.render(gl);
        self.cursor.render(gl);
    }

    pub fn handle_event(&mut self, event: &glutin::event::Event<()>) {
        if let glutin::event::Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::Resized(physical_size) => {
                    self.update_size(physical_size.width as i32, physical_size.height as i32);
                }
                WindowEvent::ReceivedCharacter(c) => {
                    // println!("{}, {:#?}", c, c);
                    match c {
                        '\u{8}' | '\r' => (), //backspace
                        _ => {
                            self.append(&c.to_string());
                        }
                    }
                }
                WindowEvent::KeyboardInput {
                    device_id: _,
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    is_synthetic: _,
                } => {
                    match keycode {
                        VirtualKeyCode::Back => {
                            self.backspace();
                        }
                        VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                            self.enter();
                        }
                        _ => (),
                    };
                }
                _ => (),
            }
        }
        self.handle_output_event();
    }

    fn handle_output_event(&mut self) {
        let r = self.event_receiver.as_ref().unwrap().lock().unwrap().try_recv();
        if let Ok(event) = r {
            // println!("{:?}", event);

            if let Event::OutputEvent(oe) = event {
                match oe {
                    OutputEvent::Print(s) => {
                        self.append(&s);
                    }
                    OutputEvent::InputPos(pos) => {
                        self.input_position = pos;
                    }
                    OutputEvent::Newline => {
                        self.newline();
                    }
                }
            }
        };
    }
}

impl EventActor for TextField {
    fn set_event_sender(&mut self, event_sender: EventSender) {
        self.event_sender = Some(event_sender);
    }

    fn set_event_receiver(&mut self, event_receiver: Mutex<EventReceiver>) {
        self.event_receiver = Some(event_receiver);
    }
}