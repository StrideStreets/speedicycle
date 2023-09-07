use crate::graph::path::FindEdge;
use crate::graph::path::Path;
use num::Bounded;
use petgraph::algo::Measure;
use petgraph::stable_graph::{IndexType, NodeIndex, StableGraph, WalkNeighbors};
use petgraph::visit::{Data, GraphBase, IntoEdges, NodeIndexable, Visitable};
use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::hash::Hash;

pub trait WalkableNeighbors<G>
where
    G: GraphBase + IndexType,
{
    fn detach(&self) -> WalkNeighbors<G>;
}

pub fn moore_shortest_s_t_path<G, K>(
    graph: &StableGraph<G::NodeWeight, G::EdgeWeight>,
    source: <G as GraphBase>::NodeId,
    target: <G as GraphBase>::NodeId,
) -> (HashMap<G::NodeId, K>, HashMap<G::NodeId, G::NodeId>)
where
    G: IntoEdges
        + Visitable
        + Data<EdgeWeight = K>
        + NodeIndexable
        + FindEdge<G>
        + IndexType
        + GraphBase<NodeId = NodeIndex>,
    G::NodeId: Eq + Hash + IndexType,
    G::EdgeId: IndexType,
    G::Neighbors: WalkableNeighbors<G>,
    K: Measure + Copy + Bounded,
{
    let mut predecessor_map = HashMap::<<G as GraphBase>::NodeId, <G as GraphBase>::NodeId>::new();
    let mut distance = HashMap::<<G as GraphBase>::NodeId, K>::new();

    let mut b = HashSet::<<G as GraphBase>::NodeId>::new();
    let mut a = HashSet::<<G as GraphBase>::NodeId>::new();

    distance.insert(source, K::default());
    distance.insert(target, K::max_value());

    b.insert(source);

    while !b.is_empty() {
        a.clear();
        b.iter().for_each(|u| {
            while let Some((e, v)) = graph.neighbors(*u).detach().next(&graph) {
                if let Some(w) = graph.edge_weight(e) {
                    let u_dist = *distance
                        .get(u)
                        .expect("Node should be present in distance map");

                    let v_dist = *distance.get(&v).unwrap_or(&K::max_value());

                    let t_dist = *distance.get(&target).unwrap_or(&K::max_value());

                    if *w < K::max_value() {
                        if (u_dist + *w < v_dist) && (u_dist + *w < t_dist) {
                            distance.insert(v, u_dist + *w);
                            predecessor_map.insert(v, *u);
                            a.insert(v);
                        }
                    }
                }
            }
        });
        b = a;
        b.remove(&target);
    }

    return (distance, predecessor_map);
}
