use super::bhandari::{
    get_edge_disjoint_path, get_path_from_predecessors, unweave_paths, BandhariGraph,
};
use super::euler::EulerGraph;

use super::scored::MaxScored;

use super::{
    path_results_to_distance_and_predecessors, predecessors_to_successors,
};
use num::Bounded;
use petgraph::{
    algo::{bellman_ford::bellman_ford, FloatMeasure},
    data::DataMap,
    prelude::EdgeIndex,
    stable_graph::{IndexType, NodeIndex, StableDiGraph},
    visit::{Data, GraphBase, NodeIndexable, Visitable},
};
use std::collections::{BinaryHeap, HashSet};
use std::{
    fmt::Debug,
    ops::{AddAssign, Mul, Neg, RemAssign},
};

pub fn double_path<G, E, Ix>(
    source: NodeIndex<Ix>,
    rg: BandhariGraph<StableDiGraph<G::NodeWeight, E, Ix>, E, Ix>,
    target_length: E,
) -> Option<(EulerGraph<G, E>, EulerGraph<G, E>)>
where
    G: Visitable
        + Data<EdgeWeight = E>
        + DataMap
        + GraphBase<NodeId = NodeIndex<Ix>, EdgeId = EdgeIndex<Ix>>
        + NodeIndexable
        + Debug,
    G::NodeWeight: Clone + Debug,
    E: Copy + FloatMeasure + Neg<Output = E> + Mul<Output = E> + RemAssign + AddAssign + Bounded,
    Ix: IndexType,
    NodeIndex<Ix>: From<u32>,
{
    let mut _iterations = 0;

    let mut h_lower = EulerGraph::<G, E>::new();
    h_lower.length = E::min_value();

    let mut h_upper = EulerGraph::<G, E>::new();
    h_upper.length = E::max_value();

    let mut failed_nodes: HashSet<NodeIndex<Ix>> = HashSet::new();

    if let Ok(paths) = bellman_ford(&rg.graph, source) {
        let (distance_map, predecessor_map) = path_results_to_distance_and_predecessors(paths);
        let mut max_dist_heap = BinaryHeap::new();
        distance_map
            .iter()
            .for_each(|(node, weight)| max_dist_heap.push(MaxScored(weight, node)));

        let successors = predecessors_to_successors(&predecessor_map);

        while let Some(MaxScored(_node_score, node)) = max_dist_heap.pop() {
            //println!("Popped node {:?}", &node);
            if failed_nodes.get(node).is_some() {
                continue;
            }
            if let Some(p1) =
                get_path_from_predecessors::<G, E>(source, *node, &predecessor_map, &distance_map)
            {
                //println!("Path One: {:?}", &p1);
                if let Some(p2) = get_edge_disjoint_path(&rg, &p1) {
                    let mut h = unweave_paths(p1, p2);

                    h.edges.iter().for_each(|(u, v)| {
                        if let Some(e) = rg.graph.find_edge(*u, *v) {
                            h.length += *rg.graph.edge_weight(e).unwrap();
                        } else if let Some(e) = rg.graph.find_edge(*v, *u) {
                            h.length += *rg.graph.edge_weight(e).unwrap();
                        }
                    });
                    _iterations += 1;

                    if h.length < target_length {
                        h.vertices.iter().for_each(|node| {
                            failed_nodes.insert(*node);
                        });

                        if h_lower.length < h.length {
                            h_lower = h;
                        }
                    } else if h.length >= target_length && h_upper.length > h.length {
                        h_upper = h;

                        let mut targets = vec![node];
                        while let Some(next_node) = targets.pop() {
                            failed_nodes.insert(*next_node);
                            if let Some(next_targets) = successors.get(next_node) {
                                targets.extend(next_targets);
                            };
                        }
                    }

                    if h_upper.length == h_lower.length {
                        break;
                    }
                }
            }
        }
        Some((h_lower, h_upper))
    } else {
        println!("Failed to execute first instance of bellman_ford");
        None
    }
}
