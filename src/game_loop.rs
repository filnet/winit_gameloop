use std::time;

use winit::event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::utility::frame;
use crate::utility::timer;

pub trait Game {
    fn update(&mut self, t: time::Duration, dt: time::Duration);
    fn render(&mut self, lag: time::Duration, dt: time::Duration);
    fn request_redraw(&self);
    fn destroy(&self);
}

pub struct GameLoop {
    event_loop: winit::event_loop::EventLoop<()>,
}

impl GameLoop {
    pub fn new() -> GameLoop {
        let event_loop = winit::event_loop::EventLoop::new();
        GameLoop { event_loop }
    }

    pub fn build_window(
        &self,
        window_builder: winit::window::WindowBuilder,
    ) -> Result<winit::window::Window, winit::error::OsError> {
        window_builder.build(&self.event_loop)
    }
}

const UPDATE_PERIOD: time::Duration = time::Duration::from_millis(10);

impl GameLoop {
    pub fn run<G: 'static + Game>(self, mut game: G) {
        // FIXME do only when needed (i.e. WaitUntil is used, not minimized, ...)
        let period = timer::set_timer_max_resolution();
        println!("timer resolution set to {:?}", period);

        let mut frame_count = frame::FrameCount::new();
        let mut frame_rate_throttle = frame::FrameRateThrottle::new();

        let mut event_count = 0;
        let mut last_event_time = time::Instant::now();

        let mut redraw = false;
        let mut resized = false;
        let mut minimized = false;

        let mut last_time = time::Instant::now();
        let mut time = time::Duration::new(0, 0);
        let mut accumulator = time::Duration::new(0, 0);

        self.event_loop.run(move |event, _, control_flow| {
            {
                let now = time::Instant::now();
                let _delta = now - last_event_time;
                last_event_time = now;
                event_count += 1;

                /*println!(
                    "EVENT {} {:?} - {:?} ({:?})",
                    event_count, event, now, _delta
                );*/
            }
            //*control_flow = ControlFlow::Wait;
            resized = false;
            match event {
                Event::DeviceEvent { .. } => {}
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            virtual_keycode,
                            state,
                            ..
                        } => match (virtual_keycode, state) {
                            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                *control_flow = ControlFlow::Exit
                            }
                            _ => {}
                        },
                    },
                    WindowEvent::Resized(new_size) => {
                        print!("RESIZE : {:?} {}\n", new_size, redraw);
                        resized = true;
                        minimized = new_size.width == 0 && new_size.height == 0;
                    }
                    _ => {}
                },
                Event::UserEvent { .. } => {}
                Event::Suspended => {}
                Event::Resumed => {}
                Event::NewEvents(start_cause) => {
                    //print!("START : {:?}\n", start_cause);
                    redraw = true;
                    match start_cause {
                        StartCause::Init => {
                            last_time = time::Instant::now();
                            time = time::Duration::new(0, 0);
                            accumulator = time::Duration::new(0, 0);
                        }
                        StartCause::Poll => {}
                        StartCause::ResumeTimeReached {
                            start: _start,
                            requested_resume: _requested_resume,
                        } => {
                            if !minimized {
                                //println!("RESUME TIME REACHED");
                            }
                            /*println!(
                                "ResumeTimeReached (requested={:?}, actual={:?}, lag={:?})",
                                _requested_resume - _start,
                                last_event_time - _start,
                                last_event_time - _requested_resume
                            );*/
                        }
                        StartCause::WaitCancelled { .. } => {
                            if !minimized {
                                //println!("WAIT CANCELLED");
                            }
                            redraw = false;
                        }
                    }
                }
                Event::MainEventsCleared => {
                    // Application update code.
                    let now = time::Instant::now();
                    let frame_duration = now - last_time;
                    last_time = now;

                    accumulator += frame_duration;

                    // TODO cap the number of iterations to avoid spiral of death...
                    while accumulator >= UPDATE_PERIOD {
                        //previousState = currentState;
                        game.update(/*currentState,*/ time, UPDATE_PERIOD);
                        time += UPDATE_PERIOD;
                        accumulator -= UPDATE_PERIOD;
                    }

                    if redraw && !minimized {
                        // Queue a RedrawRequested event.
                        game.request_redraw();
                    }
                }
                Event::RedrawRequested(_) => {
                    //assert!(!minimized, "redraw requested while minimized");
                    // Redraw the application.
                    if !redraw {
                        println!("REDRAW {}", redraw);
                    }
                    //let alpha = 0.;
                    //let alpha = accumulator.div_duration_f32(UPDATE_PERIOD);
                    if !minimized {
                        game.render(accumulator, UPDATE_PERIOD);
                    }
                    //}
                }
                Event::RedrawEventsCleared => {
                    if redraw && !minimized {
                        frame_count.frame();
                        frame_rate_throttle.frame();
                        match frame_rate_throttle.wait_until() {
                            Some(instant) => *control_flow = ControlFlow::WaitUntil(instant),
                            None => *control_flow = ControlFlow::Poll,
                        }
                    }
                    //*control_flow = ControlFlow::Poll;
                    //println!("***** {:?}", control_flow);
                    //println!("***** {:?}", frame_rate_throttle.wait_until());
                }
                Event::LoopDestroyed => {
                    game.destroy();
                }
            }
        })
    }
}
