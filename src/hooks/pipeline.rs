use crate::error::BtcResult;

use super::handler::{HookAction, HookHandler};
use super::types::HookEvent;

pub struct HookPipeline {
    handlers: Vec<Box<dyn HookHandler>>,
}

impl HookPipeline {
    pub fn new() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }

    /// Register a handler to the pipeline.
    pub fn register(&mut self, handler: Box<dyn HookHandler>) {
        self.handlers.push(handler);
    }

    /// Process an event through all matching handlers in order.
    ///
    /// Returns the first `Deny` action encountered, or `Allow` if all handlers allow.
    /// A `Modify` action is returned immediately as well.
    pub fn process(&self, event: &HookEvent) -> BtcResult<HookAction> {
        for handler in &self.handlers {
            if handler.handles(&event.hook_type) {
                let action = handler.handle(event)?;
                match &action {
                    HookAction::Allow => continue,
                    HookAction::Deny(_) | HookAction::Modify(_) => return Ok(action),
                }
            }
        }
        Ok(HookAction::Allow)
    }
}

impl Default for HookPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;
    use crate::error::BtcResult;
    use crate::hooks::types::{HookEvent, HookType};

    struct AllowHandler;
    impl HookHandler for AllowHandler {
        fn name(&self) -> &str {
            "allow"
        }
        fn handles(&self, _hook_type: &HookType) -> bool {
            true
        }
        fn handle(&self, _event: &HookEvent) -> BtcResult<HookAction> {
            Ok(HookAction::Allow)
        }
    }

    struct DenyHandler {
        reason: String,
    }
    impl HookHandler for DenyHandler {
        fn name(&self) -> &str {
            "deny"
        }
        fn handles(&self, _hook_type: &HookType) -> bool {
            true
        }
        fn handle(&self, _event: &HookEvent) -> BtcResult<HookAction> {
            Ok(HookAction::Deny(self.reason.clone()))
        }
    }

    fn test_event() -> HookEvent {
        HookEvent {
            hook_type: HookType::PreToolUse,
            agent_id: None,
            tool_name: Some("Bash".into()),
            tool_input: None,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn pipeline_all_allow() {
        let mut pipeline = HookPipeline::new();
        pipeline.register(Box::new(AllowHandler));
        pipeline.register(Box::new(AllowHandler));

        let result = pipeline.process(&test_event()).unwrap();
        assert_eq!(result, HookAction::Allow);
    }

    #[test]
    fn pipeline_deny_stops_processing() {
        let mut pipeline = HookPipeline::new();
        pipeline.register(Box::new(AllowHandler));
        pipeline.register(Box::new(DenyHandler {
            reason: "blocked".into(),
        }));
        pipeline.register(Box::new(AllowHandler));

        let result = pipeline.process(&test_event()).unwrap();
        assert_eq!(result, HookAction::Deny("blocked".into()));
    }

    #[test]
    fn pipeline_empty_allows() {
        let pipeline = HookPipeline::new();
        let result = pipeline.process(&test_event()).unwrap();
        assert_eq!(result, HookAction::Allow);
    }
}
