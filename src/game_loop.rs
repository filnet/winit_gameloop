use std::{thread, time};

use winit::event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::monitor::MonitorHandle;

use crate::utility::frame;
//use crate::utility::timer;

pub trait Game {
    fn init(&mut self);
    fn start(&mut self);
    fn event<T>(&mut self, event: &Event<'_, T>);
    fn update_fixed_step(&mut self, time: time::Duration, dt: time::Duration);
    fn update(&mut self, time: time::Duration);
    fn render(&mut self);
    fn resized(&mut self);
    fn request_redraw(&self);
    fn destroy(&self);
    fn stats(&self, game_stats: &GameStats);
}

struct GameSetup {
    update_period: time::Duration,
    redraw_on_resize: bool,
    // debugging
    lag_time: Option<time::Duration>,
}

struct GameState {
    // frame
    frame_count: u64,
    last_frame_time: Option<time::Instant>,
    time: time::Duration,
    accumulator: time::Duration,
    // loop
    loop_start_time: time::Instant,
}

// split frame and globla stats
pub struct GameStats {
    frame_id: u64,
    // frame
    pub frame_duration: time::Duration,
    time: time::Duration,
    accumulator: time::Duration,
    // loop
    /// Elapsed time between NewEvents and RedrawEventsCleared
    pub loop_duration: time::Duration,
    pub event: time::Duration,
    pub update: time::Duration,
    pub render: time::Duration,
    // events
    event_count: u64,
    total_event_count: u64,
}

//#[derive(Default)]
pub struct GameLoop {
    event_loop: winit::event_loop::EventLoop<()>,
}

impl GameLoop {
    pub fn new() -> Self {
        let event_loop = winit::event_loop::EventLoop::new();
        GameLoop { event_loop }
    }
    /*
        /// Returns the list of all the monitors available on the system.
        #[inline]
    */
    pub fn available_monitors(&self) -> impl Iterator<Item = MonitorHandle> {
        self.event_loop.available_monitors()
    }

    pub fn build_window(
        &self,
        window_builder: winit::window::WindowBuilder,
    ) -> Result<winit::window::Window, winit::error::OsError> {
        window_builder.build(&self.event_loop)
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}

// FIFO mode
// there are 3 swap chain images
// - the one being rendered to
// - the one waiting to be displayed
// - the one currently displayed
//
// first frame(s?) are fast ? not always...
//
// TODO skip first 2 or 3 frames before initializing time and accumulator
// TODO what about accumulator with high value???
// TODO introduce tick ?
//
// Solutions
// - stay as is...
// - interpolate with previous frame (but introduces lag)
// - extrapolate (need to call "update" twice)
// - adaptive update_period
// - sub-stepping
//
// MAILBOX mode
impl GameLoop {
    pub fn run<G: 'static + Game>(self, mut game: G) {
        // FIXME do only when needed (i.e. WaitUntil is used, not minimized, ...)
        //let period = timer::set_timer_max_resolution();
        //println!("timer resolution set to {:?}", period);

        let mut frame_count = frame::FrameCount::new();
        let mut frame_rate_throttle = frame::FrameRateThrottle::new();
        //frame_rate_throttle.set_target_frame_rate(frame::TargetFrameRate::FramePerSeconds(60));
        frame_rate_throttle.set_target_frame_rate(frame::TargetFrameRate::Unlimited);

        // game setup
        let setup = GameSetup {
            update_period: time::Duration::from_secs_f32(1.0 / 60.0),
            redraw_on_resize: true,
            lag_time: None, //Some(time::Duration::from_millis(4)),
        };

        // game state
        let mut state = GameState {
            // frame
            frame_count: 0,
            last_frame_time: None,
            time: time::Duration::new(0, 0),
            accumulator: time::Duration::new(0, 0),
            // loop
            loop_start_time: time::Instant::now(),
        };

        // game stats
        let mut stats = GameStats {
            frame_id: 0,
            // frame time
            frame_duration: time::Duration::new(0, 0),
            time: time::Duration::new(0, 0),
            accumulator: time::Duration::new(0, 0),
            // loop time
            loop_duration: time::Duration::new(0, 0),
            event: time::Duration::new(0, 0),
            update: time::Duration::new(0, 0),
            render: time::Duration::new(0, 0),
            // stats
            event_count: 0,
            total_event_count: 0,
        };

        // loop variables
        let mut init = false;
        let mut invalidated = false;
        let mut resized = false;
        let mut minimized = false;

        self.event_loop.run(move |event, _, control_flow| {
            {
                //let now = time::Instant::now();
                //let _delta = now - state.last_event_time.unwrap_or(now);
                //state.last_event_time = Some(now);
                stats.event_count += 1;
                stats.total_event_count += 1;
                /*println!(
                    "EVENT {} {:?} - {:?} ({:?})",
                    event_count, event, now, _delta
                );*/
            }
            //*control_flow = ControlFlow::Wait;
            //resized = false;
            match event {
                Event::DeviceEvent { .. } | Event::WindowEvent { .. } => {
                    let start_time = time::Instant::now();
                    game.event(&event);
                    stats.event += time::Instant::now() - start_time;
                }
                _ => (),
            }
            match event {
                Event::DeviceEvent { .. } => {}
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => {
                        let KeyboardInput {
                            virtual_keycode,
                            state,
                            ..
                        } = input;
                        if let (Some(VirtualKeyCode::Escape), ElementState::Pressed) =
                            (virtual_keycode, state)
                        {
                            *control_flow = ControlFlow::Exit
                        }
                    }
                    WindowEvent::Resized(new_size) => {
                        minimized = new_size.width == 0 && new_size.height == 0;
                        // hack to ignore initial (and spurious) resize events
                        if init {
                            println!("Ignored : {:?}", event);
                        } else {
                            println!("Resized : {:?}", event);
                            /*use backtrace::Backtrace;
                            let bt = Backtrace::new();
                            println!("{:?}", bt);*/
                            resized = true;
                            game.resized();
                        }
                    }
                    _ => {}
                },
                Event::UserEvent { .. } => {}
                Event::Suspended => {}
                Event::Resumed => {}
                Event::NewEvents(start_cause) => {
                    //println!("NewEvents : {:?}", start_cause);
                    // initialize state
                    init = false;
                    invalidated = true;
                    resized = false;
                    state.loop_start_time = time::Instant::now();
                    // reset stats
                    stats.event = time::Duration::new(0, 0);
                    // handle event
                    match start_cause {
                        StartCause::Init => {
                            //last_time = time::Instant::now();
                            game.init();
                            // hack to ignore initial (and spurious) resize events
                            init = true;
                            state.time = time::Duration::new(0, 0);
                            state.accumulator = time::Duration::new(0, 0);
                        }
                        StartCause::Poll => {}
                        StartCause::ResumeTimeReached {
                            start: _start,
                            requested_resume: _requested_resume,
                        } => {
                            //println!("START : {:?}", start_cause);
                            /*if !minimized {
                                println!("RESUME TIME REACHED");
                            }*/
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
                    game.start();
                }
                Event::MainEventsCleared => {
                    // Application update code.
                    // TODO now must be measured after the sleep (or as close as possible)?
                    let now = time::Instant::now();
                    let start_time = now;
                    state.frame_count += 1;
                    if invalidated {
                        match state.last_frame_time {
                            None => {
                                game.update_fixed_step(state.time, setup.update_period);
                                game.update(state.time);
                            }
                            Some(_last_time) => {
                                //let frame_duration = time::Duration::from_secs_f32(1.0 / 60.0); //now - last_time;
                                let frame_duration = now - state.last_frame_time.unwrap_or(now);
                                state.accumulator += frame_duration;
                                //println!("{:?} {:?}", frame_duration, accumulator);
                                // TODO cap the number of iterations to avoid spiral of death...
                                //let mut update_count = 0;
                                while state.accumulator >= setup.update_period {
                                    // this is pointless unless we have a physics engine that prefers fixed time step (say 10ms)
                                    // currently we don't have a physics engine (and why is update_period equals to 1/60 s?)
                                    game.update_fixed_step(state.time, setup.update_period);
                                    state.time += setup.update_period;
                                    state.accumulator -= setup.update_period;
                                    //update_count += 1;
                                }
                                /*if frame_duration > setup.update_period {
                                    println!(
                                        "!!! @{} ({:?}, {:?})",
                                        state.frame_count, frame_duration, state.accumulator
                                    );
                                }*/
                                /*println!(
                                    "@{} ({:?}, {:?})",
                                    state.frame_count, frame_duration, state.accumulator
                                );*/
                                /*if update_count == 0 {
                                    println!(
                                        "*** skipped @{} ({:?}, {:?})",
                                        state.frame_count, frame_duration, state.accumulator
                                    );
                                } else if update_count >= 2 {
                                    println!(
                                        "*** lagging {} @{} ({:?}, {:?})",
                                        update_count - 1,
                                        state.frame_count,
                                        frame_duration,
                                        state.accumulator
                                    )
                                }*/
                                game.update(state.time + state.accumulator);
                            }
                        };
                    }

                    // update stats
                    stats.frame_id = state.frame_count;
                    stats.frame_duration = now - state.last_frame_time.unwrap_or(now);
                    stats.time = state.time;
                    stats.accumulator = state.accumulator;

                    // update state
                    state.last_frame_time = Some(now);

                    //println!("{} {} {}", redraw, resized, minimized);
                    let redraw = invalidated || (setup.redraw_on_resize && resized);
                    if redraw && !minimized {
                        // Queue a RedrawRequested event.
                        //println!("REDRAW REQUESTED");
                        game.request_redraw();
                    }

                    // emulate lag
                    if let Some(lag_time) = setup.lag_time {
                        thread::sleep(lag_time);
                    }

                    stats.update = time::Instant::now() - start_time;
                }
                Event::RedrawRequested(_) => {
                    //println!("RedrawRequested");
                    let start_time = time::Instant::now();
                    let redraw = invalidated || (setup.redraw_on_resize && resized);
                    if redraw && !minimized {
                        //println!("REDRAW");
                        game.render();
                    }
                    stats.render = time::Instant::now() - start_time;
                }
                Event::RedrawEventsCleared => {
                    let now = time::Instant::now();
                    // update and send stats
                    stats.loop_duration = now - state.loop_start_time;
                    // TODO emit warning if loop duration is bigger than some value
                    // TODO these stats will be seen in the next frame
                    // while "probes" will be seen in the current frame
                    // !!!
                    game.stats(&stats);

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
