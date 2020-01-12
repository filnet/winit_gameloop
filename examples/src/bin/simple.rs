use winit_gameloop::game_loop;

fn main() {
    let game_loop = game_loop::GameLoop::new();

    game_loop.run(
        move |t, dt| {
            //println!("UPDATE {:?} {:?}", t, dt);
        },
        |dt, alpha| {
            //println!("RENDER {:?} {}", dt, alpha);
        },
    )
}
