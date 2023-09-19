mod bhandari;
pub mod double_path;
pub mod euler;
mod path;
mod scored;

use self::bhandari::BandhariGraph;

use crate::io::GraphRepresentation;

use petgraph::{
    algo::{bellman_ford::Paths, FloatMeasure, Measure},
    stable_graph::{IndexType, NodeIndex, StableDiGraph},
    visit::{GraphBase, IntoEdges},
};

use std::ops::{Add, AddAssign, Div};
use std::{collections::HashMap, hash::Hash};

pub type PredecessorMap<G> = HashMap<<G as GraphBase>::NodeId, <G as GraphBase>::NodeId>;
pub type DistanceMap<G, E> = HashMap<<G as GraphBase>::NodeId, E>;

//Note that, because of weight adjustments we will make when implementing Bandhari's
//algorithm, we need to "manually" construct an undirected graph using the
//directed graph type. That is, we will need to add two edges for each
//edge in our adjacency list (one in each direction).
pub fn make_graph<G, N, E, Ix>(gr: GraphRepresentation<N, E, Ix>) -> StableDiGraph<N, E, Ix>
where
    G: GraphBase<NodeId = NodeIndex<Ix>> + IntoEdges,
    N: Eq + Hash + Copy,
    Ix: IndexType,
    E: Copy,
{
    let mut g = StableDiGraph::<N, E, Ix>::default();
    let mut node_index_mapper: HashMap<Ix, G::NodeId> = HashMap::new();

    gr.node_map.iter().for_each(|(k, v)| {
        node_index_mapper.insert(*k, g.add_node(*v));
    });

    gr.edge_list.iter().for_each(|(u, v, w)| {
        g.add_edge(
            *node_index_mapper.get(u).unwrap(),
            *node_index_mapper.get(v).unwrap(),
            *w,
        );
        g.add_edge(
            *node_index_mapper.get(v).unwrap(),
            *node_index_mapper.get(u).unwrap(),
            *w,
        );
    });

    g
}

pub fn trim_graph_at_max_distance<N, E, Ix>(
    g: &mut StableDiGraph<N, E, Ix>,
    distance_map: &HashMap<NodeIndex<Ix>, E>,
    max_dist: E,
) -> BandhariGraph<StableDiGraph<N, E, Ix>, E, Ix>
where
    E: Copy + FloatMeasure + AddAssign + Div<f64, Output = E> + Add<f64, Output = E>,
    N: Clone,
    Ix: IndexType,
{
    let local_g = g.clone();
    let node_indices = local_g.node_indices().clone();
    for node in node_indices {
        match distance_map.get(&node) {
            Some(dist) => {
                if *dist > max_dist {
                    g.remove_node(node);
                }
            }
            None => {
                g.remove_node(node);
            }
        }
    }

    //Calculate constant for Bandhari's algorithm
    let mut inf2 = E::default();
    g.edge_weights().for_each(|w| {
        inf2 += *w;
    });

    inf2 = (inf2 / 2.0) + 1.0;

    BandhariGraph {
        graph: g.clone(),
        inf_2: inf2,
    }
}

pub fn path_results_to_distance_and_predecessors<E, Ix>(
    paths: Paths<NodeIndex<Ix>, E>,
) -> (
    HashMap<NodeIndex<Ix>, E>,
    HashMap<NodeIndex<Ix>, NodeIndex<Ix>>,
)
where
    NodeIndex<Ix>: Eq + Hash + From<u32> + Copy,
    E: Copy,
{
    let mut predecessor_map: HashMap<NodeIndex<Ix>, NodeIndex<Ix>> = HashMap::new();

    (0..)
        .zip(paths.predecessors.iter())
        .map(|(i, pred)| (NodeIndex::<Ix>::from(i), pred))
        .for_each(|(node, predecessor)| {
            if let Some(pred) = predecessor {
                predecessor_map.insert(node, *pred);
            }
        });

    let mut distance_map: HashMap<NodeIndex<Ix>, E> = HashMap::new();
    (0..)
        .zip(paths.distances.iter())
        .map(|(i, cost)| (NodeIndex::<Ix>::from(i), cost))
        .for_each(|(node, cost)| {
            if predecessor_map.get(&node).is_some() {
                distance_map.insert(node, *cost);
            }
        });

    (distance_map, predecessor_map)
}

pub fn predecessors_to_successors<Ix>(
    predecessor_map: &HashMap<NodeIndex<Ix>, NodeIndex<Ix>>,
) -> HashMap<NodeIndex<Ix>, Vec<NodeIndex<Ix>>>
where
    NodeIndex<Ix>: Eq + Hash + Copy,
{
    let mut successor_map: HashMap<NodeIndex<Ix>, Vec<NodeIndex<Ix>>> = HashMap::new();
    predecessor_map
        .iter()
        .for_each(|(node, pred)| match successor_map.get_mut(pred) {
            Some(successors) => {
                successors.push(*node);
            }
            None => {
                successor_map.insert(*pred, vec![*node]);
            }
        });
    successor_map
}
