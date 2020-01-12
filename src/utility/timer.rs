use std::cmp;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::mem;
use std::time;

use winapi::um::mmsystem;
use winapi::um::timeapi;

static mut CURRENT_PERIOD: Option<time::Duration> = None;

pub fn timer_resolution() -> (time::Duration, time::Duration) {
    unsafe {
        let mut time_caps = mmsystem::TIMECAPS {
            wPeriodMin: 0,
            wPeriodMax: 0,
        };
        match timeapi::timeGetDevCaps(
            &mut time_caps,
            u32::try_from(mem::size_of::<mmsystem::TIMECAPS>()).unwrap(),
        ) {
            mmsystem::MMSYSERR_NOERROR => {
                /*println!(
                    "Time caps: min {}ms, max {}ms",
                    time_caps.wPeriodMin, time_caps.wPeriodMax
                );*/
                (
                    time::Duration::from_millis(time_caps.wPeriodMin.into()),
                    time::Duration::from_millis(time_caps.wPeriodMax.into()),
                )
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
    }
}

pub fn set_timer_max_resolution() -> time::Duration {
    let (min_period, _) = timer_resolution();
    set_timer_resolution(min_period)
}

pub fn set_timer_resolution(target_period: time::Duration) -> time::Duration {
    unsafe {
        if CURRENT_PERIOD != None {
            panic!("unbalanced timer resolution change");
        }
        let (min_period, max_period) = timer_resolution();
        let period = cmp::min(cmp::max(min_period, target_period), max_period);
        match timeapi::timeBeginPeriod(period.as_millis().try_into().unwrap()) {
            mmsystem::MMSYSERR_NOERROR => {
                //println!("Time begin period set to {}ms", period);
                CURRENT_PERIOD = Some(period);
                period
            }
            mmsystem::TIMERR_NOCANDO => {
                panic!("period out of range");
            }
            _ => {
                panic!("unexpected result");
            }
        }
    }
}

pub fn reset_timer_resolution() {
    unsafe {
        let period = CURRENT_PERIOD.expect("unbalanced timer resolution change");
        match timeapi::timeEndPeriod(period.as_millis().try_into().unwrap()) {
            mmsystem::MMSYSERR_NOERROR => {
                CURRENT_PERIOD = None;
            }
            mmsystem::TIMERR_NOCANDO => {
                panic!("period out of range");
            }
            _ => {
                panic!("unexpected result");
            }
        }
    }
}
