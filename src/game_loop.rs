use std::time;

use winit::event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;

use crate::utility::frame;
use crate::utility::timer;
use crate::utility::window;

pub struct GameLoop {}

impl GameLoop {
    pub fn new() -> GameLoop {
        GameLoop {}
    }
}

const UPDATE_PERIOD: time::Duration = time::Duration::from_millis(10);

impl GameLoop {
    pub fn run<U, R>(self, mut update: U, mut render: R) -> !
    where
        U: 'static + FnMut(time::Duration, time::Duration),
        R: 'static + FnMut(time::Duration, f32),
    {
        // FIXME do only when needed (i.e. WaitUntil is used, not minimized, ...)
        let period = timer::set_timer_max_resolution();
        println!("timer resolution set to {:?}", period);

        let mut frame_count = frame::FrameCount::new();
        let mut frame_rate_throttle = frame::FrameRateThrottle::new();

        let mut redraw = false;
        let mut event_count = 0;
        let mut last_event_time = time::Instant::now();

        let event_loop = winit::event_loop::EventLoop::new();
        let window = window::init_window(&event_loop, "Title", 640, 480);
        //window.set_cursor_grab(true);
        //window.set_cursor_visible(false);

        let mut last_time = time::Instant::now();
        let mut time = time::Duration::new(0, 0);
        let mut accumulator = time::Duration::new(0, 0);

        event_loop.run(move |event, _, control_flow| {
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
            match event {
                Event::DeviceEvent { .. } => {}
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            match input {
                                KeyboardInput {
                                    virtual_keycode,
                                    state,
                                    ..
                                } => {
                                    match (virtual_keycode, state) {
                                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                            *control_flow = ControlFlow::Exit
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        WindowEvent::Resized(_new_size) => {
                            print!("RESIZE : {:?}\n", _new_size);
                        }
                        _ => {}
                    }
                }
                Event::UserEvent { .. } => {}
                Event::Suspended => {}
                Event::Resumed => {}
                Event::NewEvents(start_cause) => {
                    //print!("START : {:?}\n", start_cause);
                    redraw = true;
                    match start_cause {
                        StartCause::Init => {
                        }
                        StartCause::ResumeTimeReached {
                            start: _start,
                            requested_resume: _requested_resume,
                        } => {
                            /*println!(
                                "ResumeTimeReached (requested={:?}, actual={:?}, lag={:?})",
                                _requested_resume - _start,
                                last_event_time - _start,
                                last_event_time - _requested_resume
                            );*/
                        }
                        StartCause::Poll => {
                        }
                        StartCause::WaitCancelled { .. } => {
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
                        update(/*currentState,*/ time, UPDATE_PERIOD);
                        time += UPDATE_PERIOD;
                        accumulator -= UPDATE_PERIOD;
                    }

                    if redraw {
                        // Queue a RedrawRequested event.
                        window.request_redraw();
                    }
                }
                Event::RedrawRequested(_) => {
                    // Redraw the application.
                    let alpha = 0.;
                    //let alpha = accumulator.div_duration_f32(UPDATE_PERIOD);
                    render(accumulator, alpha);
                }
                Event::RedrawEventsCleared => {
                    if redraw {
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
                Event::LoopDestroyed => {}
            }
        })
    }
}
