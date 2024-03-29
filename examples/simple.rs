use std::time;

use winit_gameloop::game_loop;

use winit::event::Event;

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
    fn init(&mut self) {}

    fn start(&mut self) {}

    fn event<T>(&mut self, _event: &Event<'_, T>) {}

    fn update_fixed_step(&mut self, _time: time::Duration, _dt: time::Duration) {
        //println!("UPDATE {:?} {:?}", t, dt);
    }

    fn update(&mut self, _time: time::Duration) {
        //println!("UPDATE {:?} {:?}", t, dt);
    }

    fn render(&mut self) {
        //println!("RENDER {:?} {}", lag, dt);
    }

    fn resized(&mut self) {}

    fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn destroy(&self) {}

    fn stats(&self, _game_stats: &game_loop::GameStats) {}
}

fn main() {
    let game_loop = game_loop::GameLoop::new();

    let vulkan_app = SimpleGame::new(&game_loop);
    game_loop.run(vulkan_app);
}
