use super::{DashboardState, ReducedMotion};

pub(super) fn dashboard_state_needs_animation(state: &DashboardState) -> bool {
    if state.reduced_motion != ReducedMotion::Full {
        return false;
    }
    state.runtime_activity.active_runtime_turn || !state.live_activity_cells.is_empty()
}
