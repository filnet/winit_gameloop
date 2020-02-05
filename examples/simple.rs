use std::time;

use winit_gameloop::game_loop;

use winit;

struct SimpleGame {
    window: winit::window::Window,
}

impl SimpleGame {
    pub fn new(game_loop: &game_loop::GameLoop) -> SimpleGame {
        let window = game_loop
            .build_window(
                winit::window::WindowBuilder::new()
                    .with_title("Vulkan Game Loop")
                    .with_inner_size(winit::dpi::PhysicalSize::new(640, 480)),
            )
            .expect("Failed to create window.");
        SimpleGame { window }
    }
}

impl game_loop::Game for SimpleGame {
    fn update(&mut self, t: time::Duration, dt: time::Duration) {
        //println!("UPDATE {:?} {:?}", t, dt);
    }

    fn render(&mut self, lag: time::Duration, dt: time::Duration) {
        //println!("RENDER {:?} {}", lag, dt);
    }

    fn request_redraw(&self) {
        //println!("REDRAW REQUESTED");
    }

    fn destroy(&self) {}
}

fn main() {
    let game_loop = game_loop::GameLoop::new();

    let vulkan_app = SimpleGame::new(&game_loop);
    game_loop.run(vulkan_app);
}
