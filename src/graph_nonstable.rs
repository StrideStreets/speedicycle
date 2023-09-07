use crate::io::GraphRepresentation;
use petgraph::{
    algo::Measure,
    data::DataMap,
    graph::{EdgeIndex, EdgeReference, EdgeWeightsMut, Node, NodeIndex},
    visit::{Data, EdgeRef, GraphBase, IntoEdgeReferences, IntoEdges, Visitable},
    Graph, Undirected,
};
use std::collections::{hash_map::Entry, HashMap};
use std::hash::Hash;
mod modified_dijkstra;
mod scored;
use modified_dijkstra::modified_dijkstra;

pub fn make_graph(gr: GraphRepresentation) -> Graph<usize, usize, Undirected, u32> {
    let mut g = Graph::<usize, usize, Undirected, u32>::new_undirected();
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
    g: &Graph<usize, usize, Undirected, u32>,
    starting_node: NodeIndex,
    max_dist: usize,
) -> HashMap<NodeIndex, usize> {
    let edge_cost_fn =
        |e: <&Graph<usize, usize, Undirected, u32> as IntoEdgeReferences>::EdgeRef| {
            let weight = *e.weight();
            return weight;
        };

    let dists = modified_dijkstra(g, starting_node, edge_cost_fn, max_dist);

    return dists;
}

pub fn trim_graph_at_max_distance(
    g: Graph<usize, usize, Undirected>,
    distance_map: HashMap<NodeIndex, usize>,
    max_dist: usize,
) -> Graph<usize, usize, Undirected> {
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

    let edge_mapper = |_edge_id: EdgeIndex, edge_weight: &usize| {
        return Some(*edge_weight);
    };

    return g.filter_map(node_mapper, edge_mapper);
}
