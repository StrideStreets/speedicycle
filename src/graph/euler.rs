use petgraph::{
    data::DataMap,
    stable_graph::{EdgeIndex, IndexType, NodeIndex, StableDiGraph},
    visit::{Data, NodeIndexable},
};

use crate::graph::{GraphBase, Measure};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
    iter::Sum,
};

#[derive(Debug)]
pub struct EulerGraph<G, E>
where
    G: GraphBase,
    E: Measure + Copy + Default,
{
    pub length: E,
    pub edges: HashSet<(G::NodeId, G::NodeId)>,
    pub vertices: HashSet<G::NodeId>,
}

impl<G, E> EulerGraph<G, E>
where
    G: GraphBase,
    E: Copy + Measure + Default,
{
    pub fn new() -> Self {
        Self {
            length: E::default(),
            edges: HashSet::new(),
            vertices: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct EulerCircuit<G, E>
where
    G: GraphBase,
    E: Measure + Copy + Default,
{
    pub length: E,
    pub node_pair_list: Vec<(G::NodeId, G::NodeId)>,
    pub edge_list: Vec<G::EdgeId>,
}

impl<G, E> EulerCircuit<G, E>
where
    G: GraphBase,
    E: Copy + Measure + Default,
{
    pub fn new() -> Self {
        Self {
            length: E::default(),
            node_pair_list: Vec::new(),
            edge_list: Vec::new(),
        }
    }
}

pub fn make_euler_circuit<G, E, Ix>(
    ref_graph: &StableDiGraph<G::NodeWeight, E, Ix>,
    egraph: &EulerGraph<G, E>,
    source: G::NodeId,
) -> EulerCircuit<G, E>
where
    G: GraphBase<NodeId = NodeIndex<Ix>, EdgeId = EdgeIndex<Ix>>
        + NodeIndexable
        + Data<EdgeWeight = E>
        + DataMap,
    G::NodeId: Hash + Eq,
    E: Measure + Copy + Default + Sum,
    Ix: IndexType,
{
    let mut vertex_edge_mapper: HashMap<G::NodeId, VecDeque<G::NodeId>> = HashMap::new();

    egraph.edges.iter().for_each(|(u, v)| {
        match vertex_edge_mapper.get_mut(u) {
            Some(vec) => {
                vec.push_back(*v);
            }
            None => {
                let mut bag = VecDeque::new();
                bag.push_back(*v);
                vertex_edge_mapper.insert(*u, bag);
            }
        };

        match vertex_edge_mapper.get_mut(v) {
            Some(vec) => {
                vec.push_back(*u);
            }
            None => {
                let mut bag = VecDeque::new();
                bag.push_back(*u);
                vertex_edge_mapper.insert(*v, bag);
            }
        };
    });

    let node_order = hierholzer::<G, E>(vertex_edge_mapper, source);
    let node_pair_list: Vec<(G::NodeId, G::NodeId)> = node_order
        .iter()
        .zip(node_order.iter().skip(1))
        .map(|(s, t)| (*s, *t))
        .collect();

    let edge_list: Vec<G::EdgeId> = node_pair_list
        .iter()
        .filter_map(|(s, t)| ref_graph.find_edge(*s, *t))
        .collect();

    let length: E = edge_list
        .iter()
        .filter_map(|e| ref_graph.edge_weight(*e)).copied()
        .sum();

    EulerCircuit {
        length,
        node_pair_list,
        edge_list,
    }
}

fn hierholzer<G, E>(
    mut v_e_mapper: HashMap<G::NodeId, VecDeque<G::NodeId>>,
    source: G::NodeId,
) -> VecDeque<G::NodeId>
where
    G: GraphBase,
    G::NodeId: Hash + Eq,
    E: Copy + Measure + Default,
{
    let mut ordered_nodes = VecDeque::new();
    ordered_nodes.push_back(source);

    let mut temp_ordered_nodes = VecDeque::new();

    extract_circuit::<G, E>(&mut v_e_mapper, &mut ordered_nodes, source);

    //Pick up on implementation here, starting with EulerCircuit.cpp line 34
    loop {
        let circuit_start_node: G::NodeId;
        let mut next_iter_start_ind = 0usize;
        let mut current_ordered_nodes = ordered_nodes.clone();
        let mut iter = current_ordered_nodes.iter().enumerate().skip(0).peekable();

        loop {
            match iter.next() {
                Some((_i, node)) => {
                    if !&v_e_mapper
                        .get(node)
                        .expect("All nodes should be in mapper")
                        .is_empty()
                    {
                        circuit_start_node = *node;
                        break;
                    }
                }
                None => {
                    ordered_nodes.push_back(source);
                    return ordered_nodes;
                }
            }
        }

        extract_circuit::<G, E>(&mut v_e_mapper, &mut temp_ordered_nodes, circuit_start_node);
        temp_ordered_nodes.push_back(circuit_start_node);

        if let Some((next_ind, _next_node)) = &mut iter.peek() {
            next_iter_start_ind = *next_ind;
            while let Some(back_node) = temp_ordered_nodes.pop_back() {
                ordered_nodes.insert(*next_ind, back_node);
            }
            current_ordered_nodes = ordered_nodes.clone();
        }
        iter = current_ordered_nodes
            .iter()
            .enumerate()
            .skip(next_iter_start_ind)
            .peekable();
    }
}

fn extract_circuit<G, E>(
    v_e_mapper: &mut HashMap<G::NodeId, VecDeque<G::NodeId>>,
    ordered_nodes: &mut VecDeque<G::NodeId>,
    source: G::NodeId,
) where
    G: GraphBase,
    G::NodeId: Hash + Eq,
    E: Copy + Measure + Default,
{
    let mut u = source;

    while let Some(node) = v_e_mapper
        .get_mut(&u)
        .expect("All nodes should be in mapper")
        .pop_front()
    {
        let v = node;
        if let Some(vec) = v_e_mapper.get_mut(&u) {
            if let Some(pos) = vec.iter().position(|x| *x == v) {
                vec.swap_remove_back(pos);
            }
        }
        if let Some(vec) = v_e_mapper.get_mut(&v) {
            if let Some(pos) = vec.iter().position(|x| *x == u) {
                vec.swap_remove_back(pos);
            }
        }
        ordered_nodes.push_back(v);
        u = v;
    }

    ordered_nodes.pop_back();
}
