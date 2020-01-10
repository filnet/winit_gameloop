use std::assert;
use std::cmp;
use std::fmt;
use std::mem;
use std::time;
use std::convert::TryFrom;

use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, StartCause, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

//use winapi::shared::basetsd::{DWORD_PTR, UINT_PTR};

use winapi::um::mmsystem;
use winapi::um::timeapi;

fn main() {
    println!("Hello, world!");
    let game_loop = GameLoop::new();
    let _game_app = GameApp::new(&game_loop.event_loop);

    unsafe {
        let mut time_caps = mmsystem::TIMECAPS {
            wPeriodMin: 0,
            wPeriodMax: 0,
        };
        match timeapi::timeGetDevCaps(&mut time_caps, u32::try_from(mem::size_of::<mmsystem::TIMECAPS>()).unwrap()) {
            mmsystem::MMSYSERR_NOERROR => {
                println!(
                    "Time caps: min {}ms, max {}ms",
                    time_caps.wPeriodMin, time_caps.wPeriodMax
                );
            }
            mmsystem::MMSYSERR_ERROR => {
                panic!("general error");
            }
            mmsystem::TIMERR_NOCANDO => {
                panic!("invalid arguments or some other error");
            }
            _ => {
                panic!("unexpected result");
            }
        }
        const TARGET_RESOLUTION : u32 = 1;
        let period = cmp::min(cmp::max(time_caps.wPeriodMin, TARGET_RESOLUTION), time_caps.wPeriodMax);
        match timeapi::timeBeginPeriod(period) {
            mmsystem::MMSYSERR_NOERROR => {
                println!("Time begin period set to {}ms", period);
            }
            mmsystem::TIMERR_NOCANDO => {
                panic!("period out of range");
            }
            _ => {
                panic!("unexpected result");
            }
        }
    }

    game_loop.run();
}

// Utilities
pub fn init_window(
    event_loop: &EventLoop<()>,
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

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_SEC_F32: f32 = NANOS_PER_SEC as f32;

// FrameCount

const SAMPLE_COUNT: usize = 5;
const SAMPLE_COUNT_F32: f32 = SAMPLE_COUNT as f32;

pub struct FrameCount {
    //start_time: time::Instant,
    last_frame_time: time::Instant,
    frame_count: u64,
    fps: f32,
    samples: [time::Duration; SAMPLE_COUNT],
    started: bool,
}

impl FrameCount {
    pub fn new() -> FrameCount {
        FrameCount {
            //start_time: time::Instant::now(),
            last_frame_time: time::Instant::now(),
            frame_count: 0,
            fps: 0.,
            samples: [time::Duration::new(0, 0); SAMPLE_COUNT],
            started: false,
        }
    }

    fn start(&mut self) {
        assert!(!self.started, "frame count already started!");
        //self.start_time = time::Instant::now();
        self.last_frame_time = time::Instant::now();
        self.frame_count = 0;
        self.fps = 0.;
        self.started = true;
    }

    pub fn frame(&mut self) {
        if !self.started {
            self.start();
            return;
        }
        let frame_duration = self.last_frame_time.elapsed();

        self.frame_count += 1;
        self.samples[self.frame_count as usize % SAMPLE_COUNT] = frame_duration;

        // sliding window average
        let total: time::Duration = self.samples.iter().sum();
        self.fps = NANOS_PER_SEC_F32 / (total.as_nanos() as f32 / SAMPLE_COUNT_F32);
        // total average
        //TODO

        if self.frame_count % 100 == 0 {
            println!("{}", self);
        }

        // make sure to add frame duration
        // and not time::Instant::now() as time moves on...
        self.last_frame_time = self.last_frame_time + frame_duration;
    }

    pub fn fps(&self) -> f32 {
        self.fps
    }
}

impl fmt::Display for FrameCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FPS {:7.2} ({} frames)", self.fps, self.frame_count)
    }
}

// FrameRateThrottle

const DEFAULT_TARGET_FPS: u32 = 60;
//const DURATION_ZERO: time::Duration = time::Duration::from_nanos(0);

pub struct FrameRateThrottle {
    _target_fps: u32,
    target_frame_duration: time::Duration,
    last_frame_time: time::Instant,
    started: bool,
    wait: bool,
}

impl FrameRateThrottle {
    pub fn new() -> FrameRateThrottle {
        FrameRateThrottle {
            _target_fps: DEFAULT_TARGET_FPS,
            target_frame_duration: time::Duration::from_nanos(
                (NANOS_PER_SEC as f32 / DEFAULT_TARGET_FPS as f32) as u64,
            ),
            last_frame_time: time::Instant::now(),
            started: false,
            wait: false,
        }
    }

    fn start(&mut self) {
        assert!(!self.started, "frame rate throttle already started!");
        self.last_frame_time = time::Instant::now() + self.target_frame_duration;
        self.started = true;
        self.wait = true;
    }

    pub fn frame(&mut self) {
        if !self.started {
            self.start();
            return;
        }
        let now = time::Instant::now();
        let frame_duration = now - self.last_frame_time;
        //self.last_frame_time = self.last_frame_time + self.target_frame_duration;

        //let now = time::Instant::now();
        if frame_duration > self.target_frame_duration {
            println!(
                "*** SLOW (lag={:?})",
                frame_duration - self.target_frame_duration
            );
            self.last_frame_time = self.last_frame_time + frame_duration;
            self.wait = false;
        } else {
            //println!("FAST (duration={:?}, target={:?}, sleep={:?})", frame_duration, self.target_frame_duration, self.target_frame_duration - frame_duration);
            self.last_frame_time = self.last_frame_time + self.target_frame_duration;
            self.wait = true;
        }
    }

    pub fn wait_until(&self) -> Option<time::Instant> {
        if self.wait {
            Some(self.last_frame_time)
        } else {
            None
        }
    }
}

impl fmt::Display for FrameRateThrottle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "")
    }
}

pub struct GameApp {
    _window: winit::window::Window,
}

impl GameApp {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> GameApp {
        let _window = init_window(&event_loop, "Title", 640, 480);
        GameApp { _window }
    }
}

pub struct GameLoop {
    event_loop: EventLoop<()>,
}

impl GameLoop {
    pub fn new() -> GameLoop {
        // init window stuff
        let event_loop = EventLoop::new();

        GameLoop { event_loop }
    }

    pub fn run(self) {
        let mut frame_count = FrameCount::new();
        let mut frame_rate_throttle = FrameRateThrottle::new();
        let mut wait_cancelled = false;
        let mut event_count = 0;
        let mut last_event_time = time::Instant::now();

        self.event_loop.run(move |event, _, control_flow| {
            let now = time::Instant::now();
            let delta = now - last_event_time;
            last_event_time = now;
            event_count += 1;

            //println!("EVENT {} {:?} - {:?} ({:?})", event_count, event, now, delta);

            //*control_flow = ControlFlow::Wait;
            match event {
                Event::DeviceEvent { .. } => {}
                Event::WindowEvent { event, .. } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            //vulkan_app.wait_device_idle();
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
                                            //vulkan_app.wait_device_idle();
                                            *control_flow = ControlFlow::Exit
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        WindowEvent::Resized(_new_size) => {
                            print!("RESIZE : {:?}\n", _new_size);
                            // TODO why do we wait ?
                            //vulkan_app.wait_device_idle();
                            //vulkan_app.resize_framebuffer();
                        }
                        _ => {}
                    }
                }
                Event::UserEvent { .. } => {}
                Event::Suspended => {}
                Event::Resumed => {}
                Event::NewEvents(start_cause) => {
                    //print!("START : {:?}\n", start_cause);
                    wait_cancelled = false;
                    match start_cause {
                        StartCause::Init => {
                            //tick_counter.start();
                            //vulkan_app.request_redraw();
                        }
                        StartCause::ResumeTimeReached {
                            start,
                            requested_resume,
                        } => {
                            /*println!(
                                "ResumeTimeReached (requested={:?}, actual={:?}, lag={:?})",
                                requested_resume - start,
                                last_event_time - start,
                                last_event_time - requested_resume
                            );*/
                            //vulkan_app.request_redraw();
                        }
                        StartCause::Poll => {
                            //vulkan_app.request_redraw();
                        }
                        StartCause::WaitCancelled { .. } => {
                            wait_cancelled = true;
                        }
                    }
                }
                Event::MainEventsCleared => {
                    // Application update code.

                    // Queue a RedrawRequested event.
                    //vulkan_app.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    // Redraw the application.

                    // It's preferrable to render in this event rather than in MainEventsCleared, since
                    // rendering in here allows the program to gracefully handle redraws requested
                    // by the OS.

                    // https://gafferongames.com/post/fix_your_timestep/

                    //let accum = tick_counter.elapsed();
                    //let accum_ns = accum.as_secs() * 1_000_000_000 + accum.subsec_nanos() as u64;

                    //let d = accum_ns as f32 / 1_000_000_000.;

                    //vulkan_app.draw_frame(d);

                    //if IS_PAINT_FPS_COUNTER {
                    //    print!("FPS: {} {} ({})\r", tick_counter.fps(), d, tick_counter.frame_count());
                    //}

                    //print!("TICK\n");
                    //tick_counter.tick();
                    //print!("TOCK\n");

                    // TODO should be done in RedrawEventsCleared
                    //*control_flow = ControlFlow::WaitUntil(tick_counter.sleep_until());
                }
                Event::RedrawEventsCleared => {
                    if !wait_cancelled {
                        frame_count.frame();
                        // keep last
                        frame_rate_throttle.frame();
                    }
                    //println!("***** {:?}", frame_rate_throttle.wait_until());
                    match frame_rate_throttle.wait_until() {
                        Some(instant) => *control_flow = ControlFlow::WaitUntil(instant),
                        None => *control_flow = ControlFlow::Poll,
                    }
                }
                Event::LoopDestroyed => {
                    //vulkan_app.wait_device_idle();
                }
            }
        })
    }
}
