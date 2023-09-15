use crate::graph::{euler::EulerGraph, path::Path, path_results_to_distance_and_predecessors};
use num::Bounded;
use petgraph::{
    algo::{bellman_ford, FloatMeasure, Measure},
    data::DataMap,
    stable_graph::{IndexType, NodeIndex, StableDiGraph},
    visit::{Data, GraphBase, NodeIndexable, Visitable},
};
use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    ops::{Mul, Neg, RemAssign},
};

#[derive(Debug)]
pub struct BandhariGraph<G, E, Ix>
where
    G: Visitable + Data<EdgeWeight = E> + GraphBase<NodeId = NodeIndex<Ix>>,
    E: Measure + Copy,
{
    pub graph: G,
    pub inf_2: E,
}

pub fn get_path_from_predecessors<G, K>(
    source: <G as GraphBase>::NodeId,
    target: <G as GraphBase>::NodeId,
    predecessor_map: &HashMap<<G as GraphBase>::NodeId, <G as GraphBase>::NodeId>,
    distance_map: &HashMap<<G as GraphBase>::NodeId, K>,
) -> Option<Path<G, K>>
where
    G: Visitable + Data<EdgeWeight = K> + NodeIndexable,
    G::NodeId: Eq + Hash + Debug,
    K: Measure + Copy,
{
    let mut p = Path::<G, K>::new();
    if let Some(length) = distance_map.get(&target) {
        p.length = length.to_owned();
        let mut farthest_node = &target;
        //println!("Source: {:?}", &source);
        //println!("Target: {:?}", &target);
        while *farthest_node != source {
            //println!("Current farthest node: {:?}", &farthest_node);
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

pub fn get_edge_disjoint_path<G, E, Ix>(
    rg: &BandhariGraph<StableDiGraph<<G as Data>::NodeWeight, E, Ix>, E, Ix>,
    p: &Path<G, E>,
) -> Option<Path<G, E>>
where
    G: Visitable
        + Data<EdgeWeight = E>
        + GraphBase<NodeId = NodeIndex<Ix>>
        + NodeIndexable
        + DataMap
        + Debug,
    G::NodeId: Eq + Hash,
    G::NodeWeight: Clone + Debug,
    E: FloatMeasure + Copy + Neg<Output = E> + Mul<Output = E> + RemAssign + Bounded,
    Ix: IndexType,
    NodeIndex<Ix>: From<u32>,
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
        let mut w: <G as Data>::EdgeWeight;

        //Remove edges
        if let Some(edge_id) = g.find_edge(*u, *v) {
            //println!("Removing edge {:?}", &edge_id);
            if let Some(weight) = g.remove_edge(edge_id) {
                //print!(" with weight {:?}", &weight);
                w = weight;
                let _temp = g.add_edge(*u, *v, w * rg.inf_2);
                //println!("Added edge {:?}", temp);
            }
        }

        if let Some(edge_id) = g.find_edge(*v, *u) {
            //println!("Removing edge {:?}", &edge_id);
            if let Some(weight) = g.remove_edge(edge_id) {
                //print!(" with weight {:?}", &weight);
                w = weight;
                let _temp = g.add_edge(*v, *u, -w);
                //println!("Added edge {:?}", temp);
            }
        }

        //Add dummy directed edges
    }

    //Now, we need to get the second path, using our modified graph
    if let Ok(paths) = bellman_ford(&g, source) {
        let (mod_distance_map, mod_predecessor_map) =
            path_results_to_distance_and_predecessors(paths);

        if let Some(mut reverse_path) = get_path_from_predecessors::<G, E>(
            source,
            *target,
            &mod_predecessor_map,
            &mod_distance_map,
        ) {
            reverse_path.length %= rg.inf_2;
            println!("{:?}", &reverse_path);
            return Some(reverse_path);
        } else {
            println!("Failed to get reverse path");
            return None;
        }
    } else {
        println!("Failed to execute bellman_ford");
        return None;
    }
    // let (mod_distance_map, mod_predecessor_map) =
    //     moore_shortest_s_t_path::<G, E, Ix>(&g, source, *target);
}

//In the future, think about implementing this as an associated method on EulerGraph
pub fn unweave_paths<G, E, Ix>(p1: Path<G, E>, p2: Path<G, E>) -> EulerGraph<G, E>
where
    G: Visitable + NodeIndexable + Data<EdgeWeight = E> + GraphBase<NodeId = NodeIndex<Ix>>,
    G::NodeId: Copy,
    G::NodeId: Eq + Hash,
    E: Copy + Measure + Bounded,
{
    let mut circuit_set = EulerGraph::<G, E>::new();

    for i in 0..(p1.sequence.len() - 1) {
        circuit_set
            .edges
            .insert((p1.sequence[i], p1.sequence[i + 1]));
    }

    for i in 0..(p2.sequence.len() - 1) {
        match circuit_set.edges.get(&(p2.sequence[i + 1], p2.sequence[i])) {
            None => {
                circuit_set
                    .edges
                    .insert((p2.sequence[i], p2.sequence[i + 1]));
            }
            Some(_) => {
                circuit_set
                    .edges
                    .remove(&(p2.sequence[i + 1], p2.sequence[i]));
            }
        }
    }

    circuit_set.edges.iter().for_each(|(u, v)| {
        circuit_set.vertices.insert(*u);
        circuit_set.vertices.insert(*v);
    });

    return circuit_set;
}
