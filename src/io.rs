use anyhow::{anyhow, Error};
use itertools::Itertools;
use std::collections::HashMap;
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
        return GraphRepresentation {
            node_map: n,
            edge_list: e,
        };
    }
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

    if let Some(content) = fs::read_to_string(filepath).ok() {
        let mut lines = content.lines();

        let _size_tuple = lines.next();
        lines.for_each(|line| match line.chars().nth(0).expect("Empty line") {
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

        return Ok(GraphRepresentation::new(node_map, edge_list));
    } else {
        return Err(anyhow!("Something"));
    }
}
