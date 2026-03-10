use super::{
    decision::{desired_state_for, next_leadership_transfer, reconcile_facts},
    state::{DecideInput, DecideOutput, HaState},
};

pub(crate) fn decide(input: DecideInput) -> DecideOutput {
    let facts = reconcile_facts(&input.current, &input.world);
    let next = HaState {
        worker: input.current.worker,
        cluster_mode: facts.cluster_mode.clone(),
        desired_state: desired_state_for(&facts),
        leadership_transfer: next_leadership_transfer(&input.current.leadership_transfer, &facts),
        tick: input.current.tick.saturating_add(1),
    };

    DecideOutput { next }
}
