extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::rect::Point;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas,RenderTarget};
use std::time::Duration;

fn mand( x: f64, y: f64, max_iter: u64 ) -> f64 {
    let mut z = (x,y);
    let mut prev_dist = x*x+y*y;

    if prev_dist > 4.0 { return 1.0; }

    for iter in 0..max_iter {
        let (zx,zy) = z;
        let zxx = zx * zx;
        let zyy = zy * zy;
        let zxy = zx * zy;
        let dist = zxx+zyy;
        if dist > 4.0 {
            return iter as f64 + (4.0 - prev_dist) / (dist - prev_dist);
        }
        prev_dist = dist;

        z = (zxx - zyy + x, 2.0*zxy + y);
    }

    max_iter as f64
}

fn colormap( iter: f64 ) -> Color {
    static MAP_TABLE: &[(f64,f64,f64); 16] = &[
        (0.5,0.0,0.0),(0.5,0.3,0.0),(0.5,0.5,0.0),
        (0.0,0.5,0.0),(0.0,0.5,0.5),(0.0,0.0,0.5),
        (0.5,0.0,0.5),(0.5,0.5,0.5),
        (1.0,0.0,0.0),(1.0,0.5,0.0),(1.0,1.0,0.0),
        (0.0,1.0,0.0),(0.0,1.0,1.0),(0.0,0.0,1.0),
        (1.0,0.0,1.0),(1.0,1.0,1.0),
    ];

    let i = iter as usize;
    let f = iter - i as f64;
    let (r1,b1,g1) = MAP_TABLE[i % 16];
    let (r2,b2,g2) = MAP_TABLE[(i+1) % 16];

    let r = interpolate(r1,r2,f);
    let g = interpolate(g1,g2,f);
    let b = interpolate(b1,b2,f);

    Color::RGB( r, g, b )
}

fn interpolate( a: f64, b:f64, x:f64) -> u8 {
    ((a + (b-a)*x) * 256.0) as u8
}

fn draw_mandelbrot<T: RenderTarget>( canvas: &mut Canvas<T>, size: (u32,u32), offset: f64 )
{
    let (w,h) = size;

    for y in 0..(h as i32) {
        let cy = y as f64 * 4.0 / (h-1) as f64 - 2.0;
        for x in 0..(w as i32) {
            let cx = x as f64 * 4.0 / (w-1) as f64 - 2.0;
            let iter = mand( cx, cy, 200 );
            let color = colormap(iter + offset);
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
    draw_mandelbrot(&mut canvas, size, 0.0);
    canvas.present();

    //draw_mandelbrot(&mut canvas);

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        i = (i + 1) % 1024;
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
        let offset = i as f64 / 32.0;
        draw_mandelbrot(&mut canvas, size, offset);
        canvas.present();

//        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
