use crate::graph::{
    moore::{moore_shortest_s_t_path, WalkableNeighbors},
    path::{FindEdge, Path},
    RoutableGraph,
};
use num::Bounded;
use petgraph::{
    algo::Measure,
    data::DataMap,
    stable_graph::{IndexType, NodeIndex, StableGraph},
    visit::{Data, GraphBase, IntoEdges, NodeIndexable, Visitable},
    Undirected,
};
use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Mul, Neg, RemAssign},
    process::Output,
};

pub fn get_path_from_previous<G, K>(
    source: <G as GraphBase>::NodeId,
    target: <G as GraphBase>::NodeId,
    predecessor_map: &HashMap<<G as GraphBase>::NodeId, <G as GraphBase>::NodeId>,
    distance_map: HashMap<<G as GraphBase>::NodeId, K>,
) -> Option<Path<G, K>>
where
    G: IntoEdges + Visitable + Data<EdgeWeight = K> + NodeIndexable + FindEdge<G>,
    G::NodeId: Eq + Hash,
    K: Measure + Copy,
{
    let mut p = Path::<G, K>::new();
    if let Some(length) = distance_map.get(&target) {
        p.length = length.to_owned();
        let mut farthest_node = &target;
        while *farthest_node != source {
            p.sequence.push(*farthest_node);
            farthest_node = predecessor_map.get(farthest_node).unwrap();
        }
        p.sequence.push(source);
        p.sequence.reverse();
    } else {
        return None;
    }

    return Some(p);
}

pub fn get_edge_disjoint_path<G, K>(
    rg: &RoutableGraph<StableGraph<<G as Data>::NodeWeight, K>, K>,
    p: Path<G, K>,
) -> Option<Path<G, K>>
where
    G: IntoEdges
        + Visitable
        + Data<EdgeWeight = K>
        + GraphBase<NodeId = NodeIndex>
        + NodeIndexable
        + FindEdge<G>
        + DataMap
        + IndexType,
    G::Neighbors: WalkableNeighbors<G>,
    G::NodeId: Eq + Hash,
    G::EdgeId: IndexType,
    G::NodeWeight: Clone,
    K: Measure + Copy + Neg<Output = K> + Mul<Output = K> + RemAssign + Bounded,
{
    let source = p.sequence[0];
    let target = p
        .sequence
        .last()
        .expect("Something went wrong: empty path sequence");

    let mut g = rg.graph.clone();
    for i in 0..(p.sequence.len() - 1) {
        //Continue here translating from Bhandari.cpp line 26
        let u = &p.sequence[i];
        let v = &p.sequence[(i + 1)];

        if let Some(edge_id) = g.find_edge(*u, *v) {
            if let Some(w) = g.remove_edge(edge_id) {
                g.add_edge(*u, *v, -w);
                g.add_edge(*v, *u, w * rg.inf_2);
            }
        }
    }

    //Now, we need to get the second path, using our modified graph
    let (mod_distance_map, mod_predecessor_map) =
        moore_shortest_s_t_path::<G, K>(&g, source, *target);

    if let Some(mut reverse_path) =
        get_path_from_previous::<G, K>(source, *target, &mod_predecessor_map, mod_distance_map)
    {
        reverse_path.length %= rg.inf_2;
        return Some(reverse_path);
    } else {
        return None;
    }
}
