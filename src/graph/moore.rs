use crate::graph::path::FindEdge;
use crate::graph::path::Path;
use num::Bounded;
use petgraph::algo::Measure;
use petgraph::stable_graph::StableDiGraph;
use petgraph::stable_graph::{IndexType, NodeIndex, StableGraph, WalkNeighbors};
use petgraph::visit::IntoNeighbors;
use petgraph::visit::{Data, GraphBase, IntoEdges, NodeIndexable, Visitable};
use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::hash::Hash;

pub trait WalkableNeighbors<G, E, Ix>
where
    G: GraphBase<NodeId = NodeIndex<Ix>>,
    Ix: IndexType,
{
    fn detach(&self) -> WalkNeighbors<G>;
}

pub fn moore_shortest_s_t_path<G, E, Ix>(
    graph: &StableDiGraph<G::NodeWeight, E, Ix>,
    source: <G as GraphBase>::NodeId,
    target: <G as GraphBase>::NodeId,
) -> (HashMap<G::NodeId, E>, HashMap<G::NodeId, G::NodeId>)
where
    G: Visitable + Data<EdgeWeight = E> + NodeIndexable + GraphBase<NodeId = NodeIndex<Ix>>,
    //+ IntoNeighbors<Neighbors = WalkNeighbors<Ix>>,
    G::NodeId: Eq + Hash + IndexType,
    E: Measure + Copy + Bounded,
    Ix: IndexType,
{
    let mut predecessor_map = HashMap::<<G as GraphBase>::NodeId, <G as GraphBase>::NodeId>::new();
    let mut distance = HashMap::<G::NodeId, E>::new();

    let mut b = HashSet::<G::NodeId>::new();
    let mut a = HashSet::<G::NodeId>::new();

    distance.insert(source, E::default());
    distance.insert(target, E::max_value());

    b.insert(source);
    println!("Inside Moore's; printing b: {:?}", &b);
    while !b.is_empty() {
        a.clear();
        b.iter().for_each(|u| {
            println!("Inside loop. Operating on node {:?}", &u);
            let mut neighbors = graph.neighbors(*u);
            while let Some(v) = neighbors.next() {
                //.detach().next(&graph) {
                println!("Inside neighbors loop. Operating on node {:?}", &v);
                if let Some(edge) = graph.find_edge(*u, v) {
                    let w = *graph.edge_weight(edge).unwrap();
                    let u_dist = *distance
                        .get(u)
                        .expect("Node should be present in distance map");

                    let v_dist = *distance.get(&v).unwrap_or(&E::max_value());

                    let t_dist = *distance.get(&target).unwrap_or(&E::max_value());

                    if w < E::max_value() {
                        if (u_dist + w < v_dist) && (u_dist + w < t_dist) {
                            distance.insert(v, u_dist + w);
                            predecessor_map.insert(v, *u);
                            a.insert(v);
                        }
                    }
                }
            }
        });
        println!("Original a: {:?}", &a);
        println!("Original b: {:?}", &b);

        b = a.clone();
        println!("Overwritten b: {:?}", &b);
        b.remove(&target);

        println!(
            "Target in predecessor set? {:?}",
            predecessor_map.entry(target)
        );
    }

    return (distance, predecessor_map);
}
