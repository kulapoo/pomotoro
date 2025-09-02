use crate::AppContext;

#[allow(dead_code)]
pub fn assert_event_published(_ctx: &AppContext, _event_name: &str) {
    // TODO: Implement when event_bus has get_published_events method
    // let events = ctx.event_bus.get_published_events();
    // assert!(events.iter().any(|e| e.name == event_name));
}
