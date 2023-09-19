#![warn(unused_crate_dependencies)]
mod graph;
mod io;

use clap::Parser;
use graph::{
    double_path::double_path, euler::make_euler_circuit, make_graph, trim_graph_at_max_distance,
};
use io::{read_from_dimacs, write_solution_strings_to_file};
use petgraph::algo::dijkstra;
use petgraph::stable_graph::{NodeIndex, StableDiGraph, StableGraph};
use petgraph::Directed;
use serde_json::json;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    input_path: String,
    #[arg(short, long)]
    source_vertex: u32,
    #[arg(short, long)]
    target_length: f64,
}
fn main() {
    let args: Args = Args::parse();

    if let Ok(gr) = read_from_dimacs::<u32, f64, u32>(&args.input_path) {
        let max_dist = args.target_length * (0.6);
        let target_length = args.target_length;

        println!("{:?}", &gr.node_map.get(&args.source_vertex));

        let (mut graph, node_index_mapper) =
            make_graph::<&'static StableGraph<u32, f64, Directed, u32>, u32, f64, u32>(gr);

        println!(
            "Nodes: {}, Edges: {}",
            &graph.node_count(),
            &graph.edge_count()
        );

        let starting_node = *node_index_mapper
            .get(&args.source_vertex)
            .expect("Invalid source vertex");

        println!("{:?}", &graph.node_weight(starting_node));
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
                starting_node,
                trimmed_graph,
                target_length,
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
                    &args
                        .input_path
                        .split('.')
                        .next()
                        .expect("Filepath should contain one or more parts after splitting"),
                    "sols.txt",
                ]
                .join("_"),
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
