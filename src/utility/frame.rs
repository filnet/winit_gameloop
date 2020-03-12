use std::assert;
use std::fmt;
use std::time;

const NANOS_PER_SEC: u32 = 1_000_000_000;
const NANOS_PER_SEC_F32: f32 = NANOS_PER_SEC as f32;

// FrameCount

const SAMPLE_COUNT: usize = 5;
const SAMPLE_COUNT_F32: f32 = SAMPLE_COUNT as f32;

pub struct FrameCount {
    //start_time: time::Instant,
    last_frame_time: time::Instant,
    frame_count: u64,
    frame_per_seconds: f32,
    samples: [time::Duration; SAMPLE_COUNT],
    started: bool,
}

impl FrameCount {
    pub fn new() -> FrameCount {
        FrameCount {
            //start_time: time::Instant::now(),
            last_frame_time: time::Instant::now(),
            frame_count: 0,
            frame_per_seconds: 0.,
            samples: [time::Duration::new(0, 0); SAMPLE_COUNT],
            started: false,
        }
    }

    fn start(&mut self) {
        assert!(!self.started, "frame count already started!");
        //self.start_time = time::Instant::now();
        self.last_frame_time = time::Instant::now();
        self.frame_count = 0;
        self.frame_per_seconds = 0.;
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
        self.frame_per_seconds = NANOS_PER_SEC_F32 / (total.as_nanos() as f32 / SAMPLE_COUNT_F32);
        // total average
        //TODO

        if self.frame_count % 100 == 0 {
            println!("{}", self);
        }

        // make sure to add frame duration
        // and not time::Instant::now() as time moves on...
        self.last_frame_time = self.last_frame_time + frame_duration;
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }
    pub fn frame_per_seconds(&self) -> f32 {
        self.frame_per_seconds
    }
}

impl fmt::Display for FrameCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "FPS {:7.2} ({} frames)",
            self.frame_per_seconds(),
            self.frame_count()
        )
    }
}

// FrameRateThrottle

//const DEFAULT_TARGET_FPS: u32 = 240;

pub enum TargetFrameRate {
    Unlimited,
    FramePerSeconds(u32),
}

impl TargetFrameRate {
    pub fn target_frame_duration(&self) -> time::Duration {
        match self {
            TargetFrameRate::Unlimited => time::Duration::from_millis(0),
            TargetFrameRate::FramePerSeconds(fps) => {
                time::Duration::from_nanos((NANOS_PER_SEC as f32 / *fps as f32) as u64)
            }
        }
    }
}

/*trait TFrameRate
{
    fn target_frame_duration();
}

trait FrameRateT<T: TFrameRate> {
    fn frame(&mut self);
    fn wait_until(&self) -> Option<time::Instant>;
}*/

pub struct FrameRateThrottle {
    target_frame_rate: TargetFrameRate,
    started: bool,
    //last_frame_time: time::Instant,
    next_frame_time: time::Instant,
    wait: bool,
}

impl FrameRateThrottle {
    pub fn new() -> FrameRateThrottle {
        let target_frame_rate = TargetFrameRate::Unlimited;        
        FrameRateThrottle {
            target_frame_rate,
            started: false,
            //last_frame_time: time::Instant::now(),
            next_frame_time: time::Instant::now(),
            wait: false,
        }
    }

    pub fn set_target_frame_rate(&mut self, target_frame_rate: TargetFrameRate) {
        self.target_frame_rate = target_frame_rate;
    }

    fn start(&mut self) {
        assert!(!self.started, "frame rate throttle already started!");
        self.started = true;
        //self.last_frame_time = time::Instant::now();
        self.next_frame_time = time::Instant::now();
        self.wait = true;
    }

    pub fn frame(&mut self) {
        match self.target_frame_rate {
            TargetFrameRate::Unlimited => return,
            _ => {}
        }

        if !self.started {
            self.start();
            return;
        }
        let now = time::Instant::now();
        if now < self.next_frame_time {
            let dt = self.next_frame_time - now;
            println!("!!! TOO EARLY ({:?})", dt);
            return;
        }
        let frame_duration = now - self.next_frame_time;
        let target_frame_duration = self.target_frame_rate.target_frame_duration();
        if frame_duration > target_frame_duration {
            let dt = frame_duration - target_frame_duration;
            println!("*** LATE (lag={:?})", dt);
            self.wait = false;
            self.next_frame_time = now;
        } else {
            /*println!(
                "*** EARLY (duration={:?}, target={:?}, sleep={:?})",
                frame_duration,
                target_frame_duration,
                target_frame_duration - frame_duration
            );*/
            self.wait = true;
            self.next_frame_time = self.next_frame_time + target_frame_duration;
        }
    }

    pub fn wait_until(&self) -> Option<time::Instant> {
        match self.target_frame_rate {
            TargetFrameRate::Unlimited => return None,
            _ => {}
        }
        if self.wait {
            Some(self.next_frame_time)
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
