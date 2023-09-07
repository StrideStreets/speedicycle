mod graph;
mod io;
use graph::{get_distances, make_graph, trim_graph_at_max_distance};
use io::read_from_dimacs;
use petgraph::stable_graph::{NodeIndex, StableUnGraph};

use crate::graph::double_path;

fn main() {
    println!("Hello, world!");
    if let Ok(gr) = read_from_dimacs("routingTopologies.txt") {
        let max_dist = 3000;
        let starting_node = NodeIndex::from(12u32);
        let graph: StableUnGraph<usize, isize, u32> = make_graph(gr);

        //println!("{:?}", &graph);

        let (distances, predecessor_map, predecessor_tree) =
            get_distances(&graph, starting_node, max_dist);

        let trimmed_graph = trim_graph_at_max_distance(graph, &distances, max_dist);

        //println!("{:?}", &trimmed_graph);

        double_path(trimmed_graph, predecessor_tree, predecessor_map, distances);
    }
}
