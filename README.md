

https://randomascii.wordpress.com/2013/07/08/windows-timer-resolution-megawatts-wasted/

https://www.belshe.com/2010/06/04/chrome-cranking-up-the-clock/

https://stackoverflow.com/questions/7685762/windows-7-timing-functions-how-to-use-getsystemtimeadjustment-correctly/11743614#11743614

http://www.windowstimestamp.com/description

# Game Loop

https://gafferongames.com/post/fix_your_timestep/

https://gameprogrammingpatterns.com/game-loop.html

https://medium.com/@tglaiel/how-to-make-your-game-run-at-60fps-24c61210fe75

# Game Loop Code

https://github.com/amethyst/amethyst/blob/master/amethyst_core/src/frame_limiter.rs

http://gameprogrammingpatterns.com/game-loop.html

# MsgWaitForMultipleObjectsEx

Used here : https://github.com/rust-windowing/winit/blob/master/src/platform_impl/windows/event_loop.rs#L330

MsgWaitForMultipleObjectsEx : https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-msgwaitformultipleobjectsex

Wait functions https://docs.microsoft.com/en-us/windows/win32/sync/wait-functions

Wait Functions and Time-out Intervals

The accuracy of the specified time-out interval depends on the resolution of the system clock. The system clock "ticks" at a constant rate. If the time-out interval is less than the resolution of the system clock, the wait may time out in less than the specified length of time. If the time-out interval is greater than one tick but less than two, the wait can be anywhere between one and two ticks, and so on.

To increase the accuracy of the time-out interval for the wait functions, call the timeGetDevCaps function to determine the supported minimum timer resolution and the timeBeginPeriod function to set the timer resolution to its minimum. Use caution when calling timeBeginPeriod, as frequent calls can significantly affect the system clock, system power usage, and the scheduler. If you call timeBeginPeriod, call it one time early in the application and be sure to call the timeEndPeriod function at the very end of the application.

https://docs.microsoft.com/fr-fr/windows/win32/api/timeapi/ns-timeapi-timecaps

 
