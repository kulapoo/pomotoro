use std::sync::Arc;
use serde_json::Value;
use domain::shared_kernel::events::EventPublisher;
use super::simulator::UiSimulator;

/// Builder for creating UI simulator with specific behaviors
pub struct UiSimulatorBuilder {
    auto_acknowledge_ticks: bool,
    auto_acknowledge_state_updates: bool,
    response_delay_ms: u64,
    initial_config: Option<Value>,
}

impl UiSimulatorBuilder {
    pub fn new() -> Self {
        Self {
            auto_acknowledge_ticks: true,
            auto_acknowledge_state_updates: true,
            response_delay_ms: 10,
            initial_config: None,
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

    pub fn with_initial_config(mut self, config: Value) -> Self {
        self.initial_config = Some(config);
        self
    }

    pub fn build(self, event_bus: Arc<dyn EventPublisher>) -> UiSimulator {
        let mut simulator = UiSimulator::new(event_bus);

        // Apply builder configuration to the simulator
        simulator.set_auto_acknowledge_ticks(self.auto_acknowledge_ticks);
        simulator.set_auto_acknowledge_state_updates(self.auto_acknowledge_state_updates);
        simulator.set_response_delay_ms(self.response_delay_ms);

        // Apply initial configuration if provided
        if let Some(config) = self.initial_config {
            simulator.apply_initial_config(config);
        }

        simulator
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use crate::core::mocks::MockEventBus;

    #[tokio::test]
    async fn test_builder_pattern() {
        let event_bus = Arc::new(MockEventBus::new());

        let simulator = UiSimulatorBuilder::new()
            .with_auto_acknowledge_ticks(false)
            .with_auto_acknowledge_state_updates(true)
            .with_response_delay(20)
            .with_initial_config(json!({
                "theme": "dark",
                "work_duration": 30
            }))
            .build(event_bus);

        // Test that the simulator was built with custom configuration
        let config = simulator.config.get_config().await;
        assert!(config.is_object());
    }
}