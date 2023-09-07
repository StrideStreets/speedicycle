mod bhandari;
mod modified_dijkstra;
mod moore;
mod path;
mod scored;

use crate::io::GraphRepresentation;
use modified_dijkstra::modified_dijkstra;
use petgraph::{
    algo::Measure,
    stable_graph::{EdgeIndex, NodeIndex, StableUnGraph},
    visit::{Data, IntoEdgeReferences, Visitable},
};
use scored::MaxScored;
use std::collections::{BinaryHeap, HashMap};

pub struct RoutableGraph<G, K>
where
    G: Visitable + Data<EdgeWeight = K>,
    K: Measure + Copy,
{
    graph: G,
    inf_2: K,
}

//Note that, because of weight adjustments we will make when implementing Bandhari's
//algorithm, we need to "manually" construct an undirected graph using the
//directed graph type. That is, we will need to add two edges for each
//edge in our adjacency list (one in each direction).
pub fn make_graph(gr: GraphRepresentation) -> StableUnGraph<usize, isize, u32> {
    let mut g = StableUnGraph::<usize, isize, u32>::default();
    let mut node_index_mapper: HashMap<usize, NodeIndex> = HashMap::new();

    gr.node_map.into_iter().for_each(|(k, v)| {
        node_index_mapper.insert(k, g.add_node(v));
    });

    gr.edge_list.into_iter().for_each(|(u, v, w)| {
        g.add_edge(
            *node_index_mapper.get(&u).unwrap(),
            *node_index_mapper.get(&v).unwrap(),
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

pub fn get_distances(
    g: &StableUnGraph<usize, isize, u32>,
    starting_node: NodeIndex,
    max_dist: isize,
) -> (
    HashMap<NodeIndex, isize>,
    HashMap<NodeIndex, NodeIndex>,
    HashMap<NodeIndex, Vec<NodeIndex>>,
) {
    let edge_cost_fn = |e: <&StableUnGraph<usize, isize, u32> as IntoEdgeReferences>::EdgeRef| {
        let weight = *e.weight();
        return weight;
    };

    let (dists, predecessor_map, predecessor_tree) =
        modified_dijkstra(g, starting_node, edge_cost_fn, max_dist);

    return (dists, predecessor_map, predecessor_tree);
}

pub fn trim_graph_at_max_distance<G, K>(
    g: StableUnGraph<usize, isize>,
    distance_map: &HashMap<NodeIndex, isize>,
    max_dist: isize,
) -> RoutableGraph<StableUnGraph<usize, isize>, isize> {
    let node_mapper = |node_id: NodeIndex, node_weight: &usize| match distance_map.get(&node_id) {
        Some(val) => {
            if (val > &max_dist) {
                return None;
            } else {
                return Some(*node_weight);
            }
        }
        None => return None,
    };

    let edge_mapper = |_edge_id: EdgeIndex, edge_weight: &isize| {
        return Some(*edge_weight);
    };

    let trimmed_graph = g.filter_map(node_mapper, edge_mapper);

    //Calculate constant for Bandhari's algorithm
    let mut INF2 = 0;
    &trimmed_graph.edge_weights().for_each(|w| {
        INF2 += *w;
    });

    INF2 = (INF2 / 2) + 1;

    return RoutableGraph {
        graph: trimmed_graph,
        inf_2: INF2,
    };
}

pub fn double_path(
    rg: RoutableGraph<StableUnGraph<usize, isize>, isize>,
    predecessor_tree: HashMap<NodeIndex, Vec<NodeIndex>>,
    predecessor_map: HashMap<NodeIndex, NodeIndex>,
    distance_map: HashMap<NodeIndex, usize>,
) {
    //Need to define Euler graph with upper and lower performance bounds
    //to effectively implement while loop, below.

    //The Eulerian graph will be the result of "unweaving" paths one and two,
    //per Bandhari
    let mut max_dist_heap = BinaryHeap::new();
    distance_map
        .into_iter()
        .for_each(|(node, weight)| max_dist_heap.push(MaxScored(weight, node)));

    while !distance_map.is_empty() {}
}
