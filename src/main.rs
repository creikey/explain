extern crate sdl2;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

trait Drawable {
    fn draw(&self, canvas: &mut Canvas<sdl2::video::Window>);
    fn process_event(&mut self, e: &Event);
}

struct Line {
    points: Vec<Point>,
}

impl Line {
    fn new() -> Self {
        Line { points: Vec::new() }
    }
}

impl Drawable for Line {
    fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::BLACK);
        canvas.draw_lines(&self.points[..]).unwrap();
    }
    fn process_event(&mut self, e: &Event) {
        if let Event::MouseMotion { x, y, .. } = *e {
            self.points.push(Point::new(x, y));
        }
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("explain", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut items: Vec<Box<dyn Drawable>> = Vec::new();
    let mut item_currently_creating: Option<Box<dyn Drawable>> = None;
    'running: loop {
        canvas.set_draw_color(Color::WHITE);
        canvas.clear();

        for event in event_pump.poll_iter() {
            if let Some(item) = &mut item_currently_creating {
                item.process_event(&event);
            }
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                // TODO check the mouse_btn to see if it's left click, and check the tool that is
                // active will create stuff (tool might be trait too)
                Event::MouseButtonDown { mouse_btn, .. } => {
                    item_currently_creating = Some(Box::new(Line::new()));
                }
                Event::MouseButtonUp { mouse_btn, .. } => {
                    if item_currently_creating.is_some() {
                        items.push(item_currently_creating.unwrap());
                        item_currently_creating = None;
                    }
                }
                _ => {}
            }
        }

        for i in items.iter() {
            i.draw(&mut canvas);
        }
        if let Some(item) = &item_currently_creating {
            item.draw(&mut canvas);
        }

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
