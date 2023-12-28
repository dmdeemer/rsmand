extern crate sdl2;

use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::event::Event;
use sdl2::rect::Rect;
use sdl2::keyboard::{Keycode,Mod};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;
use std::thread::sleep;
use std::time::Instant;
use rayon::prelude::*;

#[target_feature(enable = "avx2")]
unsafe fn mand( x: f64, y: f64, max_iter: u64 ) -> Option<f64> {
    let mut z = (x,y);
    let mut prev_dist = x*x+y*y;

    if prev_dist > 4.0 { return Some(1.0); }

    for iter in 0..=max_iter {
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

#[target_feature(enable = "avx2")]
unsafe fn colormap( iter: Option<f64>, rotation: f64 ) -> Color {
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

    let iter: f64 = iter.unwrap() * 0.05 + rotation * n as f64;
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
    zoom: f64, // units: powers of 10.  Zoom 0 = a square 4.0 on a side
    size: (u32,u32),
    side: f64,
    delta: f64,
    x0: f64,
    y0: f64,
    max_iter: u64,
    resolution: u32  // Divisor from full screen resolution
}

impl Default for Zoom {
    fn default() -> Self {
        Zoom {
            center: (0.0,0.0),
            zoom: 0.0,
            size: (0,0),
            side: 4.0,
            delta: 0.0,
            x0: 0.0,
            y0: 0.0,
            max_iter: 200,
            resolution: 4,
        }
    }
}

impl Zoom {
    fn up( &mut self ) {
        self.center.1 = f64::max(-2.0,self.center.1 - 0.1 * self.side );
    }

    fn down( &mut self ) {
        self.center.1 = f64::min(2.0,self.center.1 + 0.1 * self.side );
    }

    fn left( &mut self ) {
        self.center.0 = f64::max(-2.0,self.center.0 - 0.1 * self.side );
    }

    fn right( &mut self ) {
        self.center.0 = f64::min(2.0,self.center.0 + 0.1 * self.side );
    }

    fn calc_side(zoom: f64) -> f64 {
        4.0 * (10.0_f64).powf( -zoom )
    }
    fn zoom_in( &mut self ) {
        self.zoom += 0.1;
        self.side = Zoom::calc_side(self.zoom);
    }

    fn zoom_out( &mut self ) {
        self.zoom = f64::max( 0.0, self.zoom - 0.1 );
        self.side = Zoom::calc_side(self.zoom);
    }

    fn more_iter( &mut self, inc: u64 ) { self.max_iter += inc; }

    fn less_iter( &mut self, dec: u64 ) { self.max_iter = u64::max( 2, self.max_iter - dec ); }

    fn more_resolution( &mut self ) { self.resolution = u32::max( 2, self.resolution - 1 ); }

    fn less_resolution( &mut self ) { self.resolution += 1; }


    fn set_size( &mut self, sz: (u32,u32) ) {
        self.size = sz;
        self.delta = 1.0 / (sz.1 as f64);

        self.x0 = self.center.0 - (sz.0/2) as f64 * self.delta * self.side;
        self.y0 = self.center.1 + 0.5 * self.side;
    }

    fn get_cx( &self, x: usize ) -> f64 {
        self.x0 + (x as f64) * self.delta * self.side
    }

    fn get_cy( &self, y: usize ) -> f64 {
        self.y0 - (y as f64) * self.delta * self.side
    }

    fn print( &self ) {
        println!( "Zoom: ({},{}), zoom=10^{}, maxiter={}",
            self.center.0,
            self.center.1,
            self.zoom,
            self.max_iter );
    }

}

#[target_feature(enable = "avx2")]
unsafe fn draw_row_rgba32( tex: &mut [u8], zoom: &Zoom, y: usize, offset: f64 )
{
    let cy = zoom.get_cy(y);
    for x in 0..(zoom.size.0 as usize) {
        let cx = zoom.get_cx(x);
        let iter = mand( cx, cy, zoom.max_iter );
        let color = colormap(iter, offset);
        tex[x*4+0]=color.b;
        tex[x*4+1]=color.g;
        tex[x*4+2]=color.r;
        tex[x*4+3]=255;
    }
}

fn draw_mandelbrot( canvas: &mut Canvas<Window>, size: (u32,u32), zoom: &mut Zoom, offset: f64 )
{
    let (w,h) = size;
    let (w,h) = (w/zoom.resolution,h/zoom.resolution);

    zoom.set_size((w,h));
    let zoom: &Zoom = zoom; // Drop mutability

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(Some(PixelFormatEnum::RGBA32), w, h)
        .unwrap();

    texture.with_lock(Rect::new(0, 0, w, h), |tex: &mut [u8], stride: usize| {

        tex.par_chunks_mut(stride)
           .take(h as usize)
           .enumerate()
           .for_each(|(y,row)| unsafe { draw_row_rgba32(row, zoom, y, offset); } );

    }).unwrap();

    canvas.copy(&mut texture, None, None).unwrap();
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    println!( "Num Video Displays: {}", video_subsystem.num_video_displays().unwrap() );

    println!( "Video Driver: {}", video_subsystem.current_video_driver() );

    let window_sz = video_subsystem.display_bounds(0).unwrap();
    let window = video_subsystem.window("Mandelbrot", window_sz.width(), window_sz.height())
        .resizable()
//        .maximized()
        .build()
        .unwrap();

    let size = window.drawable_size();
    let mut canvas = window.into_canvas().build().unwrap();

    let mut zoom = Zoom::default();

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    draw_mandelbrot(&mut canvas, size, &mut zoom, 0.0 );
    println!( "First Frame." );
    canvas.present();

    //draw_mandelbrot(&mut canvas);

    let max_frames_per_sec = 30.0;
    let min_frame_time = Duration::from_secs_f64(1.0/max_frames_per_sec);
    let mut tick = Instant::now();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut i = 0;
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} => { break 'running; },
                Event::KeyDown { keycode: Some(key), keymod: m, repeat: false, .. } => {
                    match key {
                        Keycode::Q        => { if m == Mod::LCTRLMOD { break 'running; } },
                        Keycode::Escape   => { break 'running; },
                        Keycode::Kp8 |
                        Keycode::Up       => { zoom.down(); },
                        Keycode::Kp2 |
                        Keycode::Down     => { zoom.up(); },
                        Keycode::Kp4 |
                        Keycode::Left     => { zoom.left(); },
                        Keycode::Kp6 |
                        Keycode::Right    => { zoom.right(); },
                        Keycode::KpPlus   => { zoom.zoom_in(); },
                        Keycode::KpMinus  => { zoom.zoom_out(); },
                        Keycode::Kp9 |
                        Keycode::PageUp   => { zoom.more_iter(25); },
                        Keycode::Kp3 |
                        Keycode::PageDown => { zoom.less_iter(25); },
                        Keycode::Period   => { zoom.more_resolution(); },
                        Keycode::Comma    => { zoom.less_resolution(); },
                        Keycode::Equals   => { zoom.print(); },
                        _ => { println!( "Key: {key:?} Mod: {m:?}" ) }
                    }
                },
                _ => {}
            }
        }
        //::std::thread::sleep(Duration::new(0, 1_000_000_000u32));

        let offset = i as f64 / 1024.0;
        draw_mandelbrot(&mut canvas, size, &mut zoom, offset );
        canvas.present();
        let now = Instant::now();
        let mut frame_duration = now - tick;
        if frame_duration < min_frame_time {
            sleep( min_frame_time - frame_duration);
            tick += min_frame_time;
            frame_duration = min_frame_time;
        }
        else {
            tick = now;
        }
        i += f64::round( frame_duration.as_secs_f64() / min_frame_time.as_secs_f64() * 2.0 ) as i32;
        i %= 1024;
    }
}
