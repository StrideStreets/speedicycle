mod graph;
mod io;

use graph::{
    double_path::double_path, euler::make_euler_circuit, make_graph, trim_graph_at_max_distance,
};
use io::read_from_dimacs;
use petgraph::algo::dijkstra;
use petgraph::stable_graph::{NodeIndex, StableDiGraph, StableGraph};
use petgraph::Directed;

fn main() {
    println!("Hello, world!");
    if let Ok(gr) = read_from_dimacs::<u32, f64, u32>("routingTopologies.txt") {
        let max_dist = 3000.0;
        let starting_node = NodeIndex::from(12u32);
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

        if let Some((_lower_bound, upper_bound)) =
            double_path::<StableDiGraph<u32, f64, u32>, f64, u32>(
                NodeIndex::new(12),
                trimmed_graph,
                4000.0,
            )
        {
            println!("Upper bound edge length: {:?}", &upper_bound.edges.len());
            println!("Upper bound edges: {:?}", &upper_bound.edges);
            let ec = make_euler_circuit::<StableDiGraph<u32, f64, u32>, f64, u32>(
                &graph,
                &upper_bound,
                starting_node,
            );

            println!("Euler Edge Length: {:?}", &ec.edge_list.len());
            println!("Euler Node Pair Length: {:?}", &ec.node_pair_list.len());
            println!("Euler Node Pairs: {:?}", &ec.node_pair_list);
        }
    }
}
