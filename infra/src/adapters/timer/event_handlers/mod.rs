mod timer_started;
mod timer_tick;
mod registry;

pub(super) use timer_started::TimerStartedHandler;
pub(super) use timer_tick::TimerTickHandler;

pub use registry::register_timer_handlers;
pub use registry::unregister_timer_handlers;