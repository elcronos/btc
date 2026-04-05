use crate::error::BtcResult;

use super::types::{HookEvent, HookType};

#[derive(Debug, Clone, PartialEq)]
pub enum HookAction {
    Allow,
    Deny(String),
    Modify(serde_json::Value),
}

pub trait HookHandler: Send + Sync {
    fn name(&self) -> &str;
    fn handles(&self, hook_type: &HookType) -> bool;
    fn handle(&self, event: &HookEvent) -> BtcResult<HookAction>;
}
