#[cfg(target_os = "android")]
#[macro_use]
extern crate android_glue;

extern crate glutin;

mod support;

#[cfg(target_os = "android")]
android_start!(main);

fn resize_callback(width: u32, height: u32) {
    println!("Window resized to {}x{}", width, height);
}

fn main() {
    let mut window = glutin::Window::new().unwrap();
    window.set_title("A fantastic window!");
    window.set_window_resize_callback(Some(resize_callback as fn(u32, u32)));
    unsafe { window.make_current() };

    let context = support::load(&window);

    while !window.is_closed() {
        context.draw_frame((0.0, 1.0, 0.0, 1.0));
        window.swap_buffers();

        println!("{:?}", window.wait_events().next());
    }
}
