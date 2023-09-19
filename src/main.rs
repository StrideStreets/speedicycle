#![warn(unused_crate_dependencies)]
mod graph;
mod io;

use graph::{
    double_path::double_path, euler::make_euler_circuit, make_graph, trim_graph_at_max_distance,
};
use io::{read_from_dimacs, write_solution_strings_to_file};
use petgraph::algo::dijkstra;
use petgraph::stable_graph::{NodeIndex, StableDiGraph, StableGraph};
use petgraph::Directed;
use serde_json::json;

fn main() {
    let filepath = std::env::args().nth(1).expect("No filepath provided.");
    let source_ind: u32 = std::env::args()
        .nth(2)
        .expect("No source index given")
        .parse()
        .expect("Source index not a valid integer.");

    if let Ok(gr) = read_from_dimacs::<u32, f64, u32>(&filepath) {
        let max_dist = 3000.0;
        let starting_node = NodeIndex::from(source_ind);
        let mut graph: StableGraph<u32, f64> =
            make_graph::<&'static StableGraph<u32, f64, Directed, u32>, u32, f64, u32>(gr);

        println!(
            "Nodes: {}, Edges: {}",
            &graph.node_count(),
            &graph.edge_count()
        );

        // let (distances, predecessor_map, predecessor_tree) =
        //     get_distances(&graph, starting_node, max_dist);

        let distances = dijkstra(&graph, starting_node, None, |e| *e.weight());
        let trimmed_graph = trim_graph_at_max_distance(&mut graph, &distances, max_dist);

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
            let upper_ec = make_euler_circuit::<StableDiGraph<u32, f64, u32>, f64, u32>(
                &graph,
                &upper_bound,
                starting_node,
            );
            let lower_ec = make_euler_circuit::<StableDiGraph<u32, f64, u32>, f64, u32>(
                &graph,
                &lower_bound,
                starting_node,
            );

            let _ = write_solution_strings_to_file(
                &[
                    &filepath
                        .split('.')
                        .next()
                        .expect("Filepath should contain one or more parts after splitting"),
                    "sols.txt",
                ]
                .join(""),
                serde_json::to_string(&vec![
                    &upper_ec.ordered_node_weight_list,
                    &lower_ec.ordered_node_weight_list,
                ])
                .unwrap(),
            );

            println!(
                "{}",
                json!(&vec![
                    &upper_ec.ordered_node_weight_list,
                    &lower_ec.ordered_node_weight_list
                ])
            )
        }
    }
}
