use anyhow::{anyhow, Error};
use itertools::Itertools;
use std::collections::HashMap;
use std::fs;

#[derive(Debug)]
pub struct GraphRepresentation {
    pub node_map: HashMap<usize, usize>,
    pub edge_list: Vec<(usize, usize, isize)>,
}

impl GraphRepresentation {
    fn new(n: HashMap<usize, usize>, e: Vec<(usize, usize, isize)>) -> GraphRepresentation {
        return GraphRepresentation {
            node_map: n,
            edge_list: e,
        };
    }
}

pub fn read_from_dimacs(filepath: &str) -> Result<GraphRepresentation, Error> {
    let mut node_map = HashMap::<usize, usize>::new();
    let mut edge_list = Vec::<(usize, usize, isize)>::new();
    let mut nodes = Vec::<Vec<&str>>::new();
    let mut edges = Vec::<Vec<&str>>::new();

    if let Some(content) = fs::read_to_string(filepath).ok() {
        let mut lines = content.lines();

        let size_tuple = lines.next();
        lines.for_each(|line| match line.chars().nth(0).expect("Empty line") {
            'e' => edges.push(line.split_whitespace().skip(1).take(3).collect()),
            'v' => nodes.push(line.split_whitespace().skip(1).collect()),
            _ => {}
        });

        nodes.into_iter().enumerate().for_each(|(i, vals)| {
            if let Some(val) = vals.first().and_then(|val| val.parse::<usize>().ok()) {
                node_map.insert(i, val);
            } else {
                panic!("Missing node identifier")
            }
        });

        edges.into_iter().for_each(|els| {
            if let Some((u, v, w)) = els
                .iter()
                .map(|val| val.parse::<usize>().unwrap())
                .collect_tuple()
            {
                let w: isize = w
                    .try_into()
                    .expect("Could not convert edge weight to signed int");
                edge_list.push((u, v, w));
            };
        });

        return Ok(GraphRepresentation::new(node_map, edge_list));
    } else {
        return Err(anyhow!("Something"));
    }
}
