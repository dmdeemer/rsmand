use x11rb::connection::Connection;
// use x11rb::errors::ReplyOrIdError;
// use x11rb::COPY_DEPTH_FROM_PARENT;
use x11rb::protocol::xproto as xp;
use x11rb::protocol::Event;
use xp::ConnectionExt;

fn print_screen_information( screen: &xp::Screen ) {
    println!();
    println!("Informations of screen {}:", screen.root);
    println!("  width.........: {}", screen.width_in_pixels);
    println!("  height........: {}", screen.height_in_pixels);
    println!("  white pixel...: {:06x}", screen.white_pixel);
    println!("  black pixel...: {:06x}", screen.black_pixel);
    println!();

}

fn draw_mandelbrot(conn: Connection) -> Result<(), Box<dyn std::error::Error>> {

    let pixmap = conn.generate_id()?;
    conn.shm_create_pixmap(
        pixmap,
        screen.root,
        width,
        1,
        screen.root_depth,
        shmseg,
        offset,
    )?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (conn, screen_num) = x11rb::connect(None).unwrap();
    let screen = &conn.setup().roots[screen_num];

    print_screen_information(screen);

    let values = xp::CreateWindowAux::default().event_mask(xp::EventMask::BUTTON_PRESS);
    let values = values.background_pixel(screen.white_pixel);

    let win_id = conn.generate_id()?;
    conn.create_window(
        24,
        win_id,
        screen.root,
        0,
        0,
        100,
        100,
        0,
        xp::WindowClass::INPUT_OUTPUT,
        0,
        &values
    )?;
    conn.map_window(win_id)?;
    conn.flush()?;
    loop {
        match conn.wait_for_event()
        {
            Ok(Event::ButtonPress(x)) => {
                println!("Button Press at {},{}", x.event_x, x.event_y);
            },
            Ok(x) => {
                println!("Event: {:?}", x);
            },
            Err(e) => {
                println!("ERROR: {:?}", e);
            }
        }
        println!("Event: {:?}", conn.wait_for_event()?);
    }
}