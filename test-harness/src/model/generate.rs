use crate::model::choose::{choose_fleet_event, ChooseError, NodeEventWeights};
use crate::model::event::{apply_fleet_event, FleetEvent};
use crate::model::Fleet;
use rand::Rng;

struct Generator {
    template: Fleet,
    current: Fleet,
    weights: NodeEventWeights,
    max_attempts: usize,
}

pub enum GeneratorError {
    ChooseError(ChooseError),
    MaxAttempts,
}

impl Generator {
    pub fn new(
        template: Fleet,
        current: Fleet,
        weights: NodeEventWeights,
        max_attempts: usize,
    ) -> Self {
        Self {
            template,
            current,
            weights,
            max_attempts,
        }
    }

    pub fn next<R: Rng>(&mut self, rng: &mut R) -> Result<FleetEvent, GeneratorError> {
        for _ in 0..self.max_attempts {
            let fleet_event = choose_fleet_event(rng, &self.template, &self.weights)
                .map_err(GeneratorError::ChooseError)?;
            if apply_fleet_event(&mut self.current, &fleet_event).is_ok() {
                return Ok(fleet_event);
            } else {
                continue;
            }
        }
        Err(GeneratorError::MaxAttempts)
    }
}
