extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::rect::Point;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas,RenderTarget};
use std::time::Duration;

fn mand( x: f64, y: f64, max_iter: u64 ) -> u64 {
    let mut iter = 0;

    let mut z = (x,y);

    while iter < max_iter {
        let (zx,zy) = z;
        let zxx = zx * zx;
        let zyy = zy * zy;
        let zxy = zx * zy;
        if (zxx+zyy) > 4.0 { break; }
        z = (zxx - zyy + x, 2.0*zxy + y);
        iter += 1;
    }

    iter
}

fn colormap( iter: u64 ) -> Color {
    static MAP_TABLE: &[u32; 32] = &[
        0x00770000, 0x00ff0000, 0x00770000, 0x00ff0000, // red
        0x00774400, 0x00ff8800, 0x00774400, 0x00ff8800, // orange
        0x00777700, 0x00ffff00, 0x00777700, 0x00ffff00, // yellow
        0x00007700, 0x0000ff00, 0x00007700, 0x0000ff00, // gren
        0x00007777, 0x0000ffff, 0x00007777, 0x0000ffff, // cyan
        0x00000077, 0x000000ff, 0x00000077, 0x000000ff, // blue
        0x00770077, 0x00ff00ff, 0x00770077, 0x00ff00ff, // purple
        0x00777777, 0x00ffffff, 0x00777777, 0x00ffffff, // white
    ];

    let color: u32 = MAP_TABLE[ (iter & 31) as usize ];

    Color::RGBA(
        ((color >> 16) & 0xFF) as u8,
        ((color >> 8) & 0xFF) as u8,
        (color & 0xFF) as u8, 255)
}


fn draw_mandelbrot<T: RenderTarget>( canvas: &mut Canvas<T>, size: (u32,u32), counter: u64 )
{
    let (w,h) = size;

    for y in 0..(h as i32) {
        let cy = y as f64 * 4.0 / (h-1) as f64 - 2.0;
        for x in 0..(w as i32) {
            let cx = x as f64 * 4.0 / (w-1) as f64 - 2.0;
            let iter = mand( cx, cy, 200 );
            let color = colormap(iter + counter);
            canvas.set_draw_color(color);
            canvas.draw_point(Point::new(x,y)).unwrap();
        }
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Mandelbrot", 800, 600)
        .position_centered()
        .build()
        .unwrap();

    let size = window.drawable_size();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    draw_mandelbrot(&mut canvas, size, 0);
    canvas.present();

    //draw_mandelbrot(&mut canvas);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 32;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::KeyDown { keycode: x, ..  } => {
                    println!( "Key: {x:?}" );
                },
                _ => {}
            }
        }
        // The rest of the game loop goes here...


        // canvas.clear();
        // canvas.set_draw_color(Color::RGB(255, 255, 0));
        // canvas.draw_point(Point::new(40,40)).unwrap();
        draw_mandelbrot(&mut canvas, size, i);
        canvas.present();

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
