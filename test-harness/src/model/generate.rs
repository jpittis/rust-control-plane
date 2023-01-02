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

impl Default for Generator {
    fn default() -> Self {
        let node_ids = vec!["node1".to_string(), "node2".to_string()];
        Self {
            template: Fleet::default(),
            current: Fleet::new(&node_ids),
            weights: NodeEventWeights::default(),
            max_attempts: 100,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
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

    pub fn current(&self) -> &Fleet {
        &self.current
    }
}

#[cfg(test)]
mod test {
    use super::Generator;
    use rand_pcg::Pcg64;
    use rand_seeder::Seeder;

    #[test]
    fn generate_multiple_events() {
        let mut rng: Pcg64 = Seeder::from("generate_multiple_events").make_rng();
        let mut gen = Generator::default();
        for _ in 0..100 {
            gen.next(&mut rng).unwrap();
        }
    }
}
