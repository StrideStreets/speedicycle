mod bhandari;
mod dijkstra_src;
mod modified_dijkstra;
mod moore;
mod path;
mod scored;

use self::{
    bhandari::{get_edge_disjoint_path, get_path_from_previous, unweave_paths},
    dijkstra_src::simplified_dijkstra,
    moore::WalkableNeighbors,
    path::FindEdge,
};
use crate::io::GraphRepresentation;
use graphalgs::shortest_path::spfa;
use num::Bounded;
use petgraph::{
    algo::{
        bellman_ford::{bellman_ford, Paths},
        FloatMeasure, Measure,
    },
    data::DataMap,
    prelude::EdgeIndex,
    stable_graph::{IndexType, NodeIndex, StableDiGraph, WalkNeighbors},
    visit::{
        Data, GraphBase, IntoEdges, IntoNeighbors, IntoNodeIdentifiers, NodeIndexable, Visitable,
    },
};
use scored::MaxScored;
use std::{
    collections::{BinaryHeap, HashMap, HashSet},
    hash::Hash,
    ops::RangeFrom,
};
use std::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, Mul, Neg, RemAssign},
};

// pub trait CalculableGraphWeight<K>
// where
//     K: Copy + Ord + Measure + Neg<Output = K> + Mul<Output = K> + RemAssign + Bounded,
// {
// }
// pub trait RoutableGraph<G, K>
// where
//     G: IntoEdges
//         + Visitable
//         + Data<EdgeWeight = K>
//         + Data<NodeWeight = K>
//         + DataMap
//         + GraphBase<NodeId = NodeIndex>
//         + NodeIndexable
//         + IndexType
//         + FindEdge<G>,
//     G::EdgeId: IndexType,
//     G::Neighbors: WalkableNeighbors<G>,
//     K: CalculableGraphWeight<K>,
// {
// }
#[derive(Debug)]
pub struct BandhariGraph<G, E, Ix>
where
    G: Visitable + Data<EdgeWeight = E> + GraphBase<NodeId = NodeIndex<Ix>>,
    E: Measure + Copy,
{
    pub graph: G,
    pub inf_2: E,
}

#[derive(Debug)]
pub struct EulerGraph<G, K>
where
    G: GraphBase,
    K: Measure + Copy + Default,
{
    pub length: K,
    pub edges: HashSet<(G::NodeId, G::NodeId)>,
    pub vertices: HashSet<G::NodeId>,
}

impl<G, K> EulerGraph<G, K>
where
    G: GraphBase,
    K: Copy + Measure + Default,
{
    fn new() -> Self {
        return Self {
            length: K::default(),
            edges: HashSet::new(),
            vertices: HashSet::new(),
        };
    }
}
//Note that, because of weight adjustments we will make when implementing Bandhari's
//algorithm, we need to "manually" construct an undirected graph using the
//directed graph type. That is, we will need to add two edges for each
//edge in our adjacency list (one in each direction).
pub fn make_graph<G, N, E, Ix>(gr: GraphRepresentation<N, E, Ix>) -> StableDiGraph<N, E, Ix>
where
    G: GraphBase<NodeId = NodeIndex<Ix>> + IntoEdges,
    N: Eq + Hash,
    Ix: IndexType,
    E: Copy,
{
    let mut g = StableDiGraph::<N, E, Ix>::default();
    let mut node_index_mapper: HashMap<Ix, G::NodeId> = HashMap::new();

    gr.node_map.into_iter().for_each(|(k, v)| {
        node_index_mapper.insert(k, g.add_node(v));
    });

    gr.edge_list.into_iter().for_each(|(u, v, w)| {
        g.add_edge(
            *node_index_mapper.get(&u).unwrap(),
            *node_index_mapper.get(&v).unwrap(),
            w,
        );
        g.add_edge(
            *node_index_mapper.get(&v).unwrap(),
            *node_index_mapper.get(&u).unwrap(),
            w,
        );
    });

    return g;
}

// pub fn get_distances<G, F, K>(
//     g: G,
//     starting_node: &G::NodeId,
//     max_dist: K,
// ) -> HashMap<<G as GraphBase>::NodeId, K>
// where
//     G: IntoEdges + Visitable + Data<EdgeWeight = K>,
//     G::NodeId: Eq + Hash,
//     K: Copy + Measure,
// {
//     let edge_cost_fn = |e: <G as IntoEdgeReferences>::EdgeRef| {
//         let weight = *e.weight();
//         return weight;
//     };

//     let dists = modified_dijkstra(g, *starting_node, edge_cost_fn, max_dist);

//     return dists;
// }

pub fn get_distances<N, E, Ix>(
    g: &StableDiGraph<N, E, Ix>,
    starting_node: NodeIndex<Ix>,
) -> HashMap<NodeIndex<Ix>, E>
where
    E: Copy + Measure,

    Ix: IndexType,
{
    // let edge_cost_fn = |e: <&StableDiGraph<usize, isize, u32> as IntoEdgeReferences>::EdgeRef| {
    //     let weight = *e.weight();
    //     return weight;
    // };

    return simplified_dijkstra(g, starting_node);
    // modified_dijkstra(g, starting_node, edge_cost_fn, max_dist);
}

pub fn trim_graph_at_max_distance<N, E, Ix>(
    mut g: StableDiGraph<N, E, Ix>,
    distance_map: &HashMap<NodeIndex<Ix>, E>,
    max_dist: E,
) -> BandhariGraph<StableDiGraph<N, E, Ix>, E, Ix>
where
    E: Copy + FloatMeasure + AddAssign + Div<f64, Output = E> + Add<f64, Output = E>,
    N: Clone,
    Ix: IndexType,
{
    //let mut local_g = g.clone();
    // let node_mapper = |node_id: NodeIndex, node_weight: &usize| match distance_map.get(&node_id) {
    //     Some(val) => {
    //         if (val > &max_dist) {
    //             return None;
    //         } else {
    //             return Some(*node_weight);
    //         }
    //     }
    //     None => return None,
    // };

    // let edge_mapper = |_edge_id: EdgeIndex, edge_weight: &isize| {
    //     return Some(*edge_weight);
    // };

    // let trimmed_graph = g.filter_map(node_mapper, edge_mapper);
    let local_g = g.clone();
    let mut node_indices = local_g.node_indices().clone();
    while let Some(node) = node_indices.next() {
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
    let mut INF2 = E::default();
    g.edge_weights().for_each(|w| {
        INF2 += *w;
    });

    INF2 = (INF2 / 2.0) + 1.0;

    return BandhariGraph {
        graph: g,
        inf_2: INF2,
    };
}

pub fn path_results_to_distance_and_predecessors<E, Ix>(
    paths: Paths<NodeIndex<Ix>, E>,
) -> (
    HashMap<NodeIndex<Ix>, E>,
    HashMap<NodeIndex<Ix>, NodeIndex<Ix>>,
)
where
    NodeIndex<Ix>: Eq + Hash + From<u32>,
{
    let mut predecessor_map: HashMap<NodeIndex<Ix>, NodeIndex<Ix>> = HashMap::new();
    (0..)
        .zip(paths.predecessors.into_iter())
        .map(|(i, pred)| (NodeIndex::<Ix>::from(i), pred))
        .for_each(|(node, predecessor)| {
            if let Some(pred) = predecessor {
                predecessor_map.insert(node, pred);
            }
        });

    let mut distance_map: HashMap<NodeIndex<Ix>, E> = HashMap::new();
    (0..)
        .zip(paths.distances.into_iter())
        .map(|(i, cost)| (NodeIndex::<Ix>::from(i), cost))
        .for_each(|(node, cost)| {
            if let Some(pred) = predecessor_map.get(&node) {
                distance_map.insert(node, cost);
            }
        });

    return (distance_map, predecessor_map);
}

pub fn double_path<G, E, Ix>(
    source: NodeIndex<Ix>,
    rg: BandhariGraph<StableDiGraph<G::NodeWeight, E, Ix>, E, Ix>,
) where
    G: Visitable
        + Data<EdgeWeight = E>
        + DataMap
        + GraphBase<NodeId = NodeIndex<Ix>, EdgeId = EdgeIndex<Ix>>
        + NodeIndexable
        + Debug,
    //+ IntoNeighbors<Neighbors = WalkNeighbors<Ix>>,
    G::NodeWeight: Clone + Debug,
    E: Copy + FloatMeasure + Neg<Output = E> + Mul<Output = E> + RemAssign + Bounded,
    Ix: IndexType,
    NodeIndex<Ix>: From<u32>,
{
    if let Ok(paths) = bellman_ford(&rg.graph, source) {
        let (distance_map, predecessor_map) = path_results_to_distance_and_predecessors(paths);
        let mut max_dist_heap = BinaryHeap::new();
        distance_map
            .clone()
            .into_iter()
            .for_each(|(node, weight)| max_dist_heap.push(MaxScored(weight, node)));

        while let Some(MaxScored(node_score, node)) = max_dist_heap.pop() {
            println!("Popped node {:?}", &node);
            if let Some(p1) =
                get_path_from_previous::<G, E>(source, node, &predecessor_map, &distance_map)
            {
                println!("Path One: {:?}", &p1);
                if let Some(p2) = get_edge_disjoint_path(&rg, &p1) {
                    println!("Path Two: {:?}", &p2);
                    println!("{:?}", unweave_paths(p1, p2));
                }
            }
        }
    } else {
        println!("Failed to execute first instance of bellman_ford");
    }

    //Need to define Euler graph with upper and lower performance bounds
    //to effectively implement while loop, below.

    //The Eulerian graph will be the result of "unweaving" paths one and two,
    //per Bandhari
}
