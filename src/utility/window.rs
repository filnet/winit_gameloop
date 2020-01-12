use winit::dpi::PhysicalSize;

// Utilities
pub fn init_window(
    event_loop: &winit::event_loop::EventLoop<()>,
    title: &str,
    width: u32,
    height: u32,
) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(PhysicalSize::new(width, height))
        .build(event_loop)
        .expect("Failed to create window.")
}
