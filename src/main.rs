extern crate sdl2;
mod cpu;

use cpu::{Cpu, GFX_COLS, GFX_ROWS};
use sdl2::event::Event;
use sdl2::keyboard::Keycode::Escape;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use std::thread::sleep;
use std::time::Duration;

const WIDTH: u32 = 640;
const HEIGHT: u32 = 320;
const TILE_WIDTH: u32 = WIDTH / 64;
const TILE_HEIGHT: u32 = HEIGHT / 32;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();

    let window = video.window("chip8", WIDTH, HEIGHT).build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut events = sdl.event_pump().unwrap();

    let mut computer = Cpu::new();
    computer.load_rom();

    'game: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Escape),
                    ..
                } => {
                    break 'game;
                }
                _ => {}
            }
        }

        #[cfg(debug_assertions)]
        {
            println!("{:?}", &computer);
        }

        computer.cycle();

        if computer.draw {
            canvas.set_draw_color(Color::RGB(0, 0, 0));
            canvas.clear();
            canvas.set_draw_color(Color::RGB(255, 255, 255));

            for row in 0..GFX_ROWS {
                for col in 0..GFX_COLS {
                    if computer.gfx[(row * 16) + col] == 1 {
                        let ty = row as i32 * TILE_HEIGHT as i32;
                        let tx = row as i32 * TILE_WIDTH as i32;
                        canvas
                            .fill_rect(Rect::new(ty, tx, TILE_WIDTH, TILE_HEIGHT))
                            .unwrap();
                    }
                }
            }

            canvas.present();
        }

        sleep(Duration::from_millis(1000 / 2));
    }
}
