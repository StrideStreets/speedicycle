use anyhow::{anyhow, Error};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::fs;
use std::hash::Hash;

use std::str::FromStr;

#[derive(Debug)]
pub struct GraphRepresentation<N, E, Ix> {
    pub node_map: HashMap<Ix, N>,
    pub edge_list: Vec<(Ix, Ix, E)>,
}

impl<N, E, Ix> GraphRepresentation<N, E, Ix> {
    fn new(n: HashMap<Ix, N>, e: Vec<(Ix, Ix, E)>) -> GraphRepresentation<N, E, Ix> {
        GraphRepresentation {
            node_map: n,
            edge_list: e,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct EdgeRepresentation<N, E> {
    pub edge_id: N,
    pub start_node: N,
    pub end_node: N,
    pub weight: E,
}

pub fn read_from_dimacs<N, E, Ix>(filepath: &str) -> Result<GraphRepresentation<N, E, Ix>, Error>
where
    Ix: FromStr + Eq + Hash + TryFrom<u32>,
    Ix::Err: Debug,
    <Ix as TryFrom<u32>>::Error: Debug,
    N: FromStr,
    E: From<Ix>,
{
    let mut node_map = HashMap::<Ix, N>::new();
    let mut edge_list = Vec::<(Ix, Ix, E)>::new();
    let mut nodes = Vec::<Vec<&str>>::new();
    let mut edges = Vec::<Vec<&str>>::new();

    if let Ok(content) = fs::read_to_string(filepath) {
        let mut lines = content.lines();

        let _size_tuple = lines.next();
        lines.for_each(|line| match line.chars().next().expect("Empty line") {
            'e' => edges.push(line.split_whitespace().skip(1).take(3).collect()),
            'v' => nodes.push(line.split_whitespace().skip(1).collect()),
            _ => {}
        });

        (0..).zip(nodes.into_iter()).for_each(|(i, vals)| {
            if let Some(val) = vals.first().and_then(|val| val.parse::<N>().ok()) {
                node_map.insert(Ix::try_from(i).unwrap(), val);
            } else {
                panic!("Missing node identifier")
            }
        });

        edges.into_iter().for_each(|els| {
            if let Some((u, v, w)) = els
                .iter()
                .map(|val| val.parse::<Ix>().unwrap())
                .collect_tuple()
            {
                let w: E = w
                    .try_into()
                    .expect("Could not convert edge weight to signed int");
                edge_list.push((u, v, w));
            };
        });

        Ok(GraphRepresentation::new(node_map, edge_list))
    } else {
        return Err(anyhow!("Something"));
    }
}

pub fn read_from_edges_json<N, E, Ix>(
    json_string: String,
) -> Result<(GraphRepresentation<N, E, Ix>, HashMap<N, Ix>), Error>
where
    for<'de> N: Deserialize<'de>,
    for<'de> E: Deserialize<'de>,
    N: PartialEq + PartialOrd + Eq + Hash + Copy,
    (E, N, N): PartialEq + PartialOrd,
    Ix: Eq + PartialEq + Hash + Copy + TryFrom<u32>,
    <Ix as TryFrom<u32>>::Error: Debug,
{
    println!("Beginning JSON parsing");
    //println!("{}", &json_string);
    if let Ok(edges_list) = serde_json::from_str::<Vec<EdgeRepresentation<N, E>>>(&json_string) {
        println!("Deserialized JSON");
        let mut node_weight_to_index = HashMap::<N, Ix>::new();
        let mut edge_list = Vec::<(Ix, Ix, E)>::new();
        let mut nodes = HashSet::<N>::new();
        let mut edges = Vec::<(E, N, N)>::new();

        edges_list.into_iter().for_each(|edge| {
            nodes.insert(edge.start_node);
            nodes.insert(edge.end_node);
            if edge.start_node < edge.end_node {
                edges.push((edge.weight, edge.start_node, edge.end_node));
            } else {
                edges.push((edge.weight, edge.start_node, edge.end_node));
            }
        });

        (0..).zip(nodes.iter()).for_each(|(i, node_id)| {
            if let Ok(ind) = Ix::try_from(i) {
                node_weight_to_index.insert(*node_id, ind);
            }
        });

        edges.into_iter().for_each(|(w, u, v)| {
            if let (Some(n1), Some(n2)) =
                (node_weight_to_index.get(&u), node_weight_to_index.get(&v))
            {
                edge_list.push((*n1, *n2, w));
            }
        });

        let node_map: HashMap<Ix, N> = node_weight_to_index.iter().map(|(k, v)| (*v, *k)).collect();

        return Ok((
            GraphRepresentation::new(node_map, edge_list),
            node_weight_to_index,
        ));
    } else {
        return Err(anyhow!("something"));
    }
}
pub fn write_solution_strings_to_file(
    path: &str,
    solution_string: String,
) -> Result<(), std::io::Error> {
    fs::write(path, solution_string)
}
