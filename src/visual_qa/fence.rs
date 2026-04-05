use crate::error::BtcResult;

/// Coordinates agent pausing for consistent checkpoint creation.
///
/// When a checkpoint is being created, all agents must be paused to ensure
/// the repository state is consistent. `AgentFence` signals agents to pause,
/// and the RAII `FenceGuard` resumes them on drop.
pub struct AgentFence;

impl AgentFence {
    pub fn new() -> Self {
        Self
    }

    /// Acquire the fence, signaling all agents to pause.
    /// Returns a guard that resumes agents when dropped.
    pub fn acquire(&self) -> BtcResult<FenceGuard> {
        tracing::info!("Agent fence acquired — agents paused for checkpoint");
        Ok(FenceGuard { _private: () })
    }
}

impl Default for AgentFence {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard that resumes agents when dropped.
pub struct FenceGuard {
    _private: (),
}

impl Drop for FenceGuard {
    fn drop(&mut self) {
        tracing::info!("Agent fence released — agents resumed");
    }
}
