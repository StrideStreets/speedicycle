mod graph;
mod io;

use graph::{double_path::double_path, make_graph, trim_graph_at_max_distance};
use io::read_from_dimacs;
use petgraph::algo::dijkstra;
use petgraph::stable_graph::{NodeIndex, StableDiGraph, StableGraph};
use petgraph::Directed;

fn main() {
    println!("Hello, world!");
    if let Ok(gr) = read_from_dimacs::<u32, f64, u32>("routingTopologies.txt") {
        let max_dist = 3000.0;
        let starting_node = NodeIndex::from(12u32);
        let graph: StableGraph<u32, f64> =
            make_graph::<&'static StableGraph<u32, f64, Directed, u32>, u32, f64, u32>(gr);

        println!(
            "Nodes: {}, Edges: {}",
            &graph.node_count(),
            &graph.edge_count()
        );

        // let (distances, predecessor_map, predecessor_tree) =
        //     get_distances(&graph, starting_node, max_dist);

        let distances = dijkstra(&graph, starting_node, None, |e| *e.weight());
        let trimmed_graph = trim_graph_at_max_distance(graph, &distances, max_dist);

        println!(
            "Nodes: {}, Edges: {}",
            &trimmed_graph.graph.node_count(),
            &trimmed_graph.graph.edge_count()
        );

        if let Some((lower_bound, upper_bound)) =
            double_path::<StableDiGraph<u32, f64, u32>, f64, u32>(
                NodeIndex::new(12),
                trimmed_graph,
                4000.0,
            )
        {
            println!("Upper bound: {:?}", upper_bound);
            println!("Lower bound: {:?}", lower_bound);
        }
    }
}
