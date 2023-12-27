extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::rect::Point;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas,RenderTarget};
//use std::time::Duration;

fn mand( x: f64, y: f64, max_iter: u64 ) -> Option<f64> {
    let mut z = (x,y);
    let mut prev_dist = x*x+y*y;

    if prev_dist > 4.0 { return Some(1.0); }

    for iter in 0..max_iter {
        let (zx,zy) = z;
        let zxx = zx * zx;
        let zyy = zy * zy;
        let zxy = zx * zy;
        let dist = zxx+zyy;
        if dist > 4.0 {
            let dist = dist.sqrt();
            let prev_dist = prev_dist.sqrt();
            return Some(iter as f64 + (2.0 - prev_dist) / (dist - prev_dist));
        }
        prev_dist = dist;

        z = (zxx - zyy + x, 2.0*zxy + y);
    }

    None
}

fn mand2( c: (f64,f64), orbit: Vec<(f64,f64)>, max_iter: usize ) -> Option<f64> {
    let mut d = c;
    let (dx,dy) = d;
    let (zx,zy) = orbit[0];
    let dist_x = dx + zx;
    let dist_y = dy + zy;
    let mut prev_dist_sq = dist_x * dist_x + dist_y * dist_y;
    if prev_dist_sq > 4.0 { return Some(1.0); }

    let max_iter = usize::min(max_iter, orbit.len());

    for iter in 1..max_iter {
        let (dx,dy) = d;
        let (zx,zy) = orbit[iter];
        d = ( dx*dx - dy*dy + 2.0*(zx*dx + zy*dy) + c.0,
              2.0*(dx*dy + zx*dy + zy*dx) + c.1 );
        let (dx,dy) = d;
        let dist_x = dx + zx;
        let dist_y = dy + zy;
        let dist_sq = dist_x * dist_x + dist_y * dist_y;
        if dist_sq > 4.0 {
            let dist = dist_sq.sqrt();
            let prev_dist = prev_dist_sq.sqrt();
            return Some(iter as f64 + (2.0 - prev_dist) / (dist - prev_dist));
        }
        prev_dist_sq = dist_sq;
    }

    None
}



fn colormap( iter: Option<f64>, rotation: f64 ) -> Color {
    static MAP_TABLE: &[(u8,u8,u8); 5] = &[
        (184,141,242), (242,133,226), (236,218,242), (68,50,227), (240,49,97)
    ];
    // static MAP_TABLE: &[(f64,f64,f64); 16] = &[
    //     (0.5,0.0,0.0),(0.5,0.3,0.0),(0.5,0.5,0.0),
    //     (0.0,0.5,0.0),(0.0,0.5,0.5),(0.0,0.0,0.5),
    //     (0.5,0.0,0.5),(0.5,0.5,0.5),
    //     (1.0,0.0,0.0),(1.0,0.5,0.0),(1.0,1.0,0.0),
    //     (0.0,1.0,0.0),(0.0,1.0,1.0),(0.0,0.0,1.0),
    //     (1.0,0.0,1.0),(1.0,1.0,1.0),
    // ];

    let n: usize = MAP_TABLE.len();
    if iter.is_none() { return Color::BLACK; }

    let iter: f64 = iter.unwrap() * 0.2 + rotation * n as f64;
    let i = iter as usize;
    let f = iter - i as f64;
    let (r1,g1,b1) = MAP_TABLE[i % n];
    let (r2,g2,b2) = MAP_TABLE[(i+1) % n];

    let r = interpolate(r1,r2,f);
    let g = interpolate(g1,g2,f);
    let b = interpolate(b1,b2,f);

    Color::RGB( r, g, b )
}

fn interpolate( a:u8, b:u8, x:f64) -> u8 {
    let a = a as f64;
    let b = b as f64;
    ((a + (b-a)*x)) as u8
}

struct Zoom {
    center: (f64,f64),
    zoom: f64 // units: powers of 10.  Zoom 0 = a square 4.0 on a side
}

impl Default for Zoom {
    fn default() -> Self {
        Zoom { center: (0.0,0.0), zoom: 0.0 }
    }
}

impl Zoom {
    fn side( &self ) -> f64 {
        4.0 * (10.0_f64).powf( -self.zoom )
    }

    fn up( &mut self ) {
        self.center.1 = f64::max(-2.0,self.center.1 - 0.1 * self.side() );
    }

    fn down( &mut self ) {
        self.center.1 = f64::min(2.0,self.center.1 + 0.1 * self.side() );
    }

    fn left( &mut self ) {
        self.center.0 = f64::max(-2.0,self.center.0 - 0.1 * self.side() );
    }

    fn right( &mut self ) {
        self.center.0 = f64::min(2.0,self.center.0 + 0.1 * self.side() );
    }

    fn zoom_in( &mut self ) {
        self.zoom += 0.1;
    }

    fn zoom_out( &mut self ) {
        self.zoom = f64::max( 0.0, self.zoom - 0.1 );
    }
}

fn draw_mandelbrot<T: RenderTarget>( canvas: &mut Canvas<T>, size: (u32,u32), zoom: &Zoom, offset: f64 )
{
    let (w,h) = size;

    let side = zoom.side();
    let d = 1.0 / (h as f64);
    let x0 = zoom.center.0 - (size.0/2) as f64 * d * side;
    let y0 = zoom.center.1 + (size.1/2) as f64 * d * side;

    for y in 0..(h as i32) {
        let cy = y0 - (y as f64) * d * side;
        for x in 0..(w as i32) {
            let cx = x0 + (x as f64) * d * side;
            let iter = mand( cx, cy, 200 );
            let color = colormap(iter, offset);
            canvas.set_draw_color(color);
            canvas.draw_point(Point::new(x,y)).unwrap();
        }
    }
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Mandelbrot", 800, 600)
//        .position_centered()
        .build()
        .unwrap();

    let size = window.drawable_size();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut zoom = Zoom::default();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    draw_mandelbrot(&mut canvas, size, &zoom, 0.0);
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
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => { zoom.down(); },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => { zoom.up(); },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => { zoom.left(); },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => { zoom.right(); },
                Event::KeyDown { keycode: Some(Keycode::KpPlus), .. } => { zoom.zoom_in(); },
                Event::KeyDown { keycode: Some(Keycode::KpMinus), .. } => { zoom.zoom_out(); },
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
        let offset = i as f64 / 1024.0;
        draw_mandelbrot(&mut canvas, size, &zoom, offset);
        canvas.present();

//        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}
