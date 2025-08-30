use std::sync::Arc;
use domain::shared_kernel::events::{Event, EventPublisher};
use domain::event_names::ui_listeners::timer as timer_events;
use tokio::sync::mpsc;
use std::time::Duration;

/// Simulates UI behavior by responding to backend events
/// This is crucial for integration tests where the UI would normally
/// acknowledge events like timer ticks
pub struct UiSimulator {
    _event_bus: Arc<dyn EventPublisher>,
    response_channel: mpsc::UnboundedSender<UiResponse>,
    receiver: Option<mpsc::UnboundedReceiver<UiResponse>>,
}

#[derive(Debug, Clone)]
pub enum UiResponse {
    TimerTickAcknowledged,
    TimerStateUpdated,
    TaskSwitched(String),
    Custom(String),
}

impl UiSimulator {
    pub fn new(event_bus: Arc<dyn EventPublisher>) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            _event_bus: event_bus,
            response_channel: tx,
            receiver: Some(rx),
        }
    }

    /// Start listening for events and automatically respond as the UI would
    pub fn start_auto_responder(mut self) -> UiSimulatorHandle {
        let handle = UiSimulatorHandle {
            response_channel: self.response_channel.clone(),
        };

        let mut receiver = self.receiver.take().unwrap();

        tokio::spawn(async move {
            while let Some(response) = receiver.recv().await {
                // Simulate UI processing delay
                tokio::time::sleep(Duration::from_millis(10)).await;
                
                match response {
                    UiResponse::TimerTickAcknowledged => {
                        // UI would typically update its display here
                        // In tests, we just record that it happened
                    }
                    UiResponse::TimerStateUpdated => {
                        // UI would refresh the timer state display
                    }
                    UiResponse::TaskSwitched(_task_id) => {
                        // UI would update the active task display
                    }
                    UiResponse::Custom(_event_type) => {
                        // Handle custom event types
                    }
                }
            }
        });

        handle
    }

    /// Simulate UI acknowledging a timer tick event
    pub async fn acknowledge_timer_tick(&self) {
        let _ = self.response_channel.send(UiResponse::TimerTickAcknowledged);
    }

    /// Simulate UI acknowledging a state update
    pub async fn acknowledge_state_update(&self) {
        let _ = self.response_channel.send(UiResponse::TimerStateUpdated);
    }

    /// Simulate UI switching active task
    pub async fn switch_task(&self, task_id: String) {
        let _ = self.response_channel.send(UiResponse::TaskSwitched(task_id));
    }
}

/// Handle for controlling the UI simulator
pub struct UiSimulatorHandle {
    response_channel: mpsc::UnboundedSender<UiResponse>,
}

impl UiSimulatorHandle {
    /// Trigger a UI response manually
    pub fn trigger_response(&self, response: UiResponse) {
        let _ = self.response_channel.send(response);
    }

    /// Simulate the UI acknowledging multiple timer ticks
    pub async fn acknowledge_ticks(&self, count: usize) {
        for _ in 0..count {
            self.trigger_response(UiResponse::TimerTickAcknowledged);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }
}

/// Builder for creating UI simulator with specific behaviors
pub struct UiSimulatorBuilder {
    auto_acknowledge_ticks: bool,
    auto_acknowledge_state_updates: bool,
    response_delay_ms: u64,
}

impl UiSimulatorBuilder {
    pub fn new() -> Self {
        Self {
            auto_acknowledge_ticks: true,
            auto_acknowledge_state_updates: true,
            response_delay_ms: 10,
        }
    }

    pub fn with_auto_acknowledge_ticks(mut self, enabled: bool) -> Self {
        self.auto_acknowledge_ticks = enabled;
        self
    }

    pub fn with_auto_acknowledge_state_updates(mut self, enabled: bool) -> Self {
        self.auto_acknowledge_state_updates = enabled;
        self
    }

    pub fn with_response_delay(mut self, delay_ms: u64) -> Self {
        self.response_delay_ms = delay_ms;
        self
    }

    pub fn build(self, event_bus: Arc<MockEventBus>) -> UiEventInterceptor {
        UiEventInterceptor::new(
            event_bus,
            self.auto_acknowledge_ticks,
            self.auto_acknowledge_state_updates,
            self.response_delay_ms,
        )
    }
}

/// Intercepts events published by the backend and simulates UI responses
pub struct UiEventInterceptor {
    _event_bus: Arc<MockEventBus>,
    auto_acknowledge_ticks: bool,
    auto_acknowledge_state_updates: bool,
    response_delay_ms: u64,
}

impl UiEventInterceptor {
    pub fn new(
        event_bus: Arc<MockEventBus>,
        auto_acknowledge_ticks: bool,
        auto_acknowledge_state_updates: bool,
        response_delay_ms: u64,
    ) -> Self {
        Self {
            _event_bus: event_bus,
            auto_acknowledge_ticks,
            auto_acknowledge_state_updates,
            response_delay_ms,
        }
    }

    /// Process events as if the UI received them
    pub async fn process_event(&self, event: &dyn Event) {
        // Simulate network/processing delay
        tokio::time::sleep(Duration::from_millis(self.response_delay_ms)).await;

        let event_type = event.event_type();
        if event_type == timer_events::TICK && self.auto_acknowledge_ticks {
            // UI would update its timer display here
            // In real UI: invoke_command("timer:acknowledge_tick")
        } else if event_type == timer_events::STATE_UPDATED && self.auto_acknowledge_state_updates {
            // UI would refresh its state
            // In real UI: invoke_command("timer:get_state")
        }
    }

    /// Simulate the UI being disconnected (no acknowledgments)
    pub fn simulate_disconnect(&mut self) {
        self.auto_acknowledge_ticks = false;
        self.auto_acknowledge_state_updates = false;
    }

    /// Simulate the UI reconnecting
    pub fn simulate_reconnect(&mut self) {
        self.auto_acknowledge_ticks = true;
        self.auto_acknowledge_state_updates = true;
    }
}

use super::MockEventBus;

#[cfg(test)]
mod tests {
    use super::*;
    use domain::timer::events::Tick;
    use domain::timer::Phase;

    #[tokio::test]
    async fn ui_simulator_responds_to_events() {
        let event_bus = Arc::new(MockEventBus::new());
        let simulator = UiSimulator::new(event_bus.clone());
        let handle = simulator.start_auto_responder();

        // Simulate backend publishing a timer tick
        let tick_event = Tick::new(
            Some("test_task".to_string()),
            Phase::Work,
            1500,
            1,
        );
        
        event_bus.publish(Box::new(tick_event));
        
        // Give the simulator time to process
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Trigger manual acknowledgment
        handle.trigger_response(UiResponse::TimerTickAcknowledged);
    }

    #[tokio::test]
    async fn interceptor_auto_acknowledges() {
        let event_bus = Arc::new(MockEventBus::new());
        let interceptor = UiEventInterceptor::new(
            event_bus.clone(),
            true,  // auto acknowledge ticks
            true,  // auto acknowledge state updates
            5,     // 5ms delay
        );

        let tick_event = Tick::new(
            Some("test_task".to_string()),
            Phase::Work,
            1500,
            1,
        );

        interceptor.process_event(&tick_event).await;
        
        // Verify the event was processed (add assertions based on your needs)
    }
}