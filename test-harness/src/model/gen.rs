use crate::model::event::{FleetEvent, NodeEvent};
use crate::model::{Fleet, Node};
use rand::distributions::weighted::WeightedError;
use rand::seq::SliceRandom;
use rand::Rng;

type Weighted<T> = (T, usize);

fn gen_node_choices<R: Rng>(rng: &mut R, node: &Node) -> Vec<Weighted<NodeEvent>> {
    todo!()
}

pub fn gen_fleet_event<R: Rng>(rng: &mut R, fleet: &Fleet) -> Result<FleetEvent, WeightedError> {
    let mut choices = Vec::new();
    for (node_id, node) in &fleet.nodes {
        let node_choices = gen_node_choices(rng, node);
        for (node_event, weight) in node_choices {
            choices.push((
                FleetEvent {
                    node_id: node_id.clone(),
                    node_event,
                },
                weight,
            ));
        }
    }
    choose_weighted(rng, choices)
}

fn choose_weighted<R: Rng, T: Clone>(
    rng: &mut R,
    choices: Vec<(T, usize)>,
) -> Result<T, WeightedError> {
    choices
        .choose_weighted(rng, |item| item.1)
        .map(|item| item.0.clone())
}
