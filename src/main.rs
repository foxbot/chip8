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
const TILE_WIDTH: u32 = (WIDTH / 64);
const TILE_HEIGHT: u32 = (HEIGHT / 32);

fn main() {
    // TODO: docopt or whatever
    let mut args = std::env::args();
    if args.len() != 2 {
        println!("USAGE: chip8 <rom.ch8>");
        return;
    }
    let _ = args.next().unwrap();
    let path = args.next().unwrap();

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();

    let window = video.window("chip8", WIDTH, HEIGHT).build().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let mut events = sdl.event_pump().unwrap();

    let mut computer = Cpu::new();
    computer.load_rom(&path);

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
                Event::KeyDown { keycode: code, .. } => {
                    if let Some(key) = code {
                        let idx = get_index_from_key(key);
                        if idx == 0xFF {
                            break;
                        }
                        computer.key[idx] = 1;
                    }
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
                    if computer.gfx[row][col] == 1 {
                        let ty = row as i32 * TILE_HEIGHT as i32;
                        let tx = col as i32 * TILE_WIDTH as i32;
                        canvas
                            .fill_rect(Rect::new(tx, ty, TILE_WIDTH, TILE_HEIGHT))
                            .unwrap();
                    }
                }
            }

            canvas.present();
        }

        sleep(Duration::from_millis(1000 / 60));
    }
}

fn get_index_from_key(key: sdl2::keyboard::Keycode) -> usize {
    use sdl2::keyboard::Keycode::{Num1, Num2, Num3, Num4, A, C, D, E, F, Q, R, S, V, W, X, Z};
    match key {
        Num1 => 0x1,
        Num2 => 0x2,
        Num3 => 0x3,
        Num4 => 0xC,
        Q => 0x4,
        W => 0x5,
        E => 0x6,
        R => 0xD,
        A => 0x7,
        S => 0x8,
        D => 0x9,
        F => 0xE,
        Z => 0xA,
        X => 0x0,
        C => 0xB,
        V => 0xF,
        _ => 0xFF,
    }
}
