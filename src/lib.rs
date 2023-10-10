pub mod graph;
pub mod io;

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, Neg, RemAssign};
use std::str::FromStr;

use anyhow::{anyhow, Error};
use clap::Parser;
use graph::{
    double_path::double_path, euler::make_euler_circuit, make_graph, trim_graph_at_max_distance,
};
use io::{read_from_dimacs, read_from_edges_json, write_solution_strings_to_file};
use num::Bounded;
use petgraph::algo::{dijkstra, FloatMeasure, Measure};
use petgraph::stable_graph::{IndexType, StableDiGraph, StableGraph};
use petgraph::Directed;

#[derive(Parser)]
pub struct CLIArgs {
    #[arg(short, long)]
    input_path: String,
    #[arg(short, long)]
    source_vertex: u32,
    #[arg(short, long)]
    target_length: f64,
}

pub struct RoutingResults<N> {
    pub upper: Vec<N>,
    pub lower: Vec<N>,
}

pub fn make_route_from_dimacs<N, E, Ix>(
    args: CLIArgs,
    return_routes: bool,
) -> Result<RoutingResults<N>, Error>
where
    Ix: IndexType + FromStr + From<u32>,
    <Ix as FromStr>::Err: Debug,
    N: 'static + FromStr + Debug + Eq + Hash + Copy + Serialize,
    E: 'static
        + From<Ix>
        + Copy
        + Debug
        + Measure
        + Bounded
        + FloatMeasure
        + AddAssign
        + RemAssign
        + Div<f64, Output = E>
        + Add<f64, Output = E>
        + From<f64>
        + Neg<Output = E>
        + Mul<Output = E>
        + Sum,
{
    if let Ok(gr) = read_from_dimacs::<N, E, Ix>(&args.input_path) {
        let max_dist = args.target_length * (0.6);
        let target_length = args.target_length;

        println!("{:?}", &gr.node_map.get(&args.source_vertex.into()));

        let (mut graph, node_index_mapper) =
            make_graph::<&'static StableGraph<N, E, Directed, Ix>, Ix>(gr);

        println!(
            "Nodes: {}, Edges: {}",
            &graph.node_count(),
            &graph.edge_count()
        );

        let starting_node = *node_index_mapper
            .get(&args.source_vertex.into())
            .expect("Invalid source vertex");

        println!("{:?}", &graph.node_weight(starting_node));
        // let (distances, predecessor_map, predecessor_tree) =
        //     get_distances(&graph, starting_node, max_dist);

        let distances = dijkstra(&graph, starting_node, None, |e| *e.weight());
        let trimmed_graph = trim_graph_at_max_distance(&mut graph, &distances, max_dist.into());

        println!(
            "Nodes: {}, Edges: {}",
            &trimmed_graph.graph.node_count(),
            &trimmed_graph.graph.edge_count()
        );
        let mut upper_ec;
        let mut lower_ec;
        let mut double_path_iterations = 1;
        loop {
            if let Some((lower_bound, upper_bound)) = double_path::<StableDiGraph<N, E, Ix>, Ix>(
                starting_node,
                &trimmed_graph,
                target_length.into(),
            ) {
                println!("{:?}", &upper_bound);

                upper_ec = make_euler_circuit::<StableDiGraph<N, E, Ix>, Ix>(
                    &graph,
                    &upper_bound,
                    starting_node,
                );
                println!("{:?}", &upper_ec);
                lower_ec = make_euler_circuit::<StableDiGraph<N, E, Ix>, Ix>(
                    &graph,
                    &lower_bound,
                    starting_node,
                );

                if upper_ec.ordered_node_weight_list.first()
                    == upper_ec.ordered_node_weight_list.last()
                    && lower_ec.ordered_node_weight_list.first()
                        == lower_ec.ordered_node_weight_list.last()
                {
                    break;
                } else {
                    double_path_iterations += 1;
                    println!("Double path iterations: {:}", double_path_iterations);
                    if double_path_iterations > 50 {
                        return Err(anyhow!(
                            "Unable to locate valid circuit within 50 iterations."
                        ));
                    }
                }
            }
        }

        let solutions_vector = vec![
            upper_ec.ordered_node_weight_list,
            lower_ec.ordered_node_weight_list,
        ];

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
            serde_json::to_string(&solutions_vector).unwrap(),
        );

        if return_routes {
            return Ok(RoutingResults {
                upper: solutions_vector[0].clone(),
                lower: solutions_vector[1].clone(),
            });
        }
    }

    Err(anyhow!(
        "Failed to produce valid circuit for provided input."
    ))
}

pub fn make_route_from_edges_json<N, E, Ix>(
    json_string: String,
    source_vertex_id: N,
    target_length: E,
) -> Result<RoutingResults<N>, Error>
where
    Ix: IndexType + FromStr + From<u32>,
    <Ix as FromStr>::Err: Debug,
    for<'de> N: Deserialize<'de>,
    for<'de> E: Deserialize<'de>,
    N: 'static + FromStr + Debug + Eq + Hash + Copy + PartialOrd,
    E: 'static
        + From<Ix>
        + Copy
        + Debug
        + Measure
        + Bounded
        + FloatMeasure
        + AddAssign
        + RemAssign
        + Div<f64, Output = E>
        + Add<f64, Output = E>
        + From<f64>
        + Neg<Output = E>
        + Mul<Output = E>
        + Sum,
{
    println!("Source vertex ID: {:?}", &source_vertex_id);
    println!("Target distance: {:?}", &target_length);
    if let Ok((gr, weight_to_node_id)) = read_from_edges_json::<N, E, Ix>(json_string) {
        println!("Made graph from provided JSON");
        let max_dist = target_length * (0.6.into());

        let (mut graph, node_index_mapper) =
            make_graph::<&'static StableGraph<N, E, Directed, Ix>, Ix>(gr);

        let starting_node = match weight_to_node_id.get(&source_vertex_id) {
            Some(idx) => {
                println!("{:?}", &idx);
                match node_index_mapper.get(idx) {
                    Some(node_idx) => *node_idx,
                    None => return Err(anyhow!("Node index not found")),
                }
            }
            None => return Err(anyhow!("Invalid source vertex")),
        };

        let distances = dijkstra(&graph, starting_node, None, |e| *e.weight());
        let trimmed_graph = trim_graph_at_max_distance(&mut graph, &distances, max_dist.into());

        let mut upper_ec;
        let mut lower_ec;
        let mut double_path_iterations = 1;
        loop {
            if let Some((lower_bound, upper_bound)) = double_path::<StableDiGraph<N, E, Ix>, Ix>(
                starting_node,
                &trimmed_graph,
                target_length.into(),
            ) {
                println!("{:?}", &upper_bound);

                upper_ec = make_euler_circuit::<StableDiGraph<N, E, Ix>, Ix>(
                    &graph,
                    &upper_bound,
                    starting_node,
                );
                println!("{:?}", &upper_ec);
                lower_ec = make_euler_circuit::<StableDiGraph<N, E, Ix>, Ix>(
                    &graph,
                    &lower_bound,
                    starting_node,
                );

                if upper_ec.ordered_node_weight_list.first()
                    == upper_ec.ordered_node_weight_list.last()
                    && lower_ec.ordered_node_weight_list.first()
                        == lower_ec.ordered_node_weight_list.last()
                {
                    break;
                } else {
                    double_path_iterations += 1;
                    println!("Double path iterations: {:}", double_path_iterations);
                    if double_path_iterations > 50 {
                        return Err(anyhow!(
                            "Unable to locate valid circuit within 50 iterations."
                        ));
                    }
                }
            }
        }

        let solutions_vector = vec![
            upper_ec.ordered_node_weight_list,
            lower_ec.ordered_node_weight_list,
        ];

        return Ok(RoutingResults {
            upper: solutions_vector[0].clone(),
            lower: solutions_vector[1].clone(),
        });
    } else {
        Err(anyhow!(
            "Failed to produce valid circuit for provided input."
        ))
    }
}
