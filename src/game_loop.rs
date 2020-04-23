use std::time;

use winit::event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::utility::frame;
use crate::utility::timer;

pub trait Game {
    fn event<T>(&mut self, event: &Event<'_, T>);
    fn update(&mut self, t: time::Duration, dt: time::Duration);
    fn render(&mut self, lag: time::Duration, dt: time::Duration);
    fn resized(&mut self);
    fn request_redraw(&self);
    fn destroy(&self);
}

#[derive(Default)]
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
        frame_rate_throttle.set_target_frame_rate(frame::TargetFrameRate::FramePerSeconds(60));

        // GLOBAL VARIABLES

        // settings
        let redraw_on_resize = true;

        // game time
        let mut last_time = time::Instant::now();
        let mut time = time::Duration::new(0, 0);
        let mut accumulator = time::Duration::new(0, 0);

        // stats
        let mut event_count = 0;
        let mut last_event_time = time::Instant::now();

        // loop variables
        let mut invalidated = false;
        let mut resized = false;
        let mut minimized = false;

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
            //resized = false;
            game.event(&event);
            match event {
                Event::DeviceEvent { .. } => {}
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            virtual_keycode,
                            state,
                            ..
                        } => {
                            if let (Some(VirtualKeyCode::Escape), ElementState::Pressed) =
                                (virtual_keycode, state)
                            {
                                *control_flow = ControlFlow::Exit
                            }
                        }
                    },
                    WindowEvent::Resized(new_size) => {
                        println!("RESIZED : {:?} {}", new_size, invalidated);
                        resized = true;
                        minimized = new_size.width == 0 && new_size.height == 0;

                        game.resized();
                    }
                    _ => {}
                },
                Event::UserEvent { .. } => {}
                Event::Suspended => {}
                Event::Resumed => {}
                Event::NewEvents(start_cause) => {
                    //println!("START : {:?}", start_cause);
                    invalidated = true;
                    resized = false;
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
                            //println!("START : {:?}", start_cause);
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
                            //println!("START : {:?}", start_cause);
                            if !minimized {
                                //println!("WAIT CANCELLED");
                            }
                            invalidated = false;
                        }
                    }
                }
                Event::MainEventsCleared => {
                    // Application update code.
                    if invalidated {
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
                    }
                    //println!("{} {} {}", redraw, resized, minimized);
                    let redraw = invalidated || (redraw_on_resize && resized);
                    if redraw && !minimized {
                        // Queue a RedrawRequested event.
                        //println!("REDRAW REQUESTED");
                        game.request_redraw();
                    }
                }
                Event::RedrawRequested(_) => {
                    let redraw = invalidated || (redraw_on_resize && resized);
                    if redraw && !minimized {
                        //println!("REDRAW");
                        //let alpha = 0.;
                        //let alpha = accumulator.div_duration_f32(UPDATE_PERIOD);
                        game.render(accumulator, UPDATE_PERIOD);
                    }
                }
                Event::RedrawEventsCleared => {
                    if invalidated && !minimized {
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
