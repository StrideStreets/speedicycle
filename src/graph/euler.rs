use petgraph::{
    data::DataMap,
    stable_graph::{EdgeIndex, IndexType, NodeIndex, StableDiGraph},
    visit::{Data, NodeIndexable},
};

use crate::graph::{GraphBase, Measure};

use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt::Debug,
    hash::Hash,
    iter::Sum,
};

#[derive(Debug)]
pub struct EulerGraph<G>
where
    G: GraphBase + Data,
{
    pub length: G::EdgeWeight,
    pub edges: HashSet<(G::NodeId, G::NodeId)>,
    pub vertices: HashSet<G::NodeId>,
}

impl<G> EulerGraph<G>
where
    G: GraphBase + Data,
    G::EdgeWeight: Default,
{
    pub fn new() -> Self {
        Self {
            length: G::EdgeWeight::default(),
            edges: HashSet::new(),
            vertices: HashSet::new(),
        }
    }
}

#[derive(Debug)]
pub struct EulerCircuit<G>
where
    G: GraphBase + Data + DataMap,
{
    pub length: G::EdgeWeight,
    pub node_pair_list: Vec<(G::NodeId, G::NodeId)>,
    pub edge_list: Vec<G::EdgeId>,
    pub ordered_node_weight_list: Vec<G::NodeWeight>,
}

impl<G> EulerCircuit<G>
where
    G: GraphBase + Data + DataMap,
    G::EdgeWeight: Default + Sum,
{
    pub fn new() -> Self {
        Self {
            length: G::EdgeWeight::default(),
            node_pair_list: Vec::new(),
            edge_list: Vec::new(),
            ordered_node_weight_list: Vec::new(),
        }
    }
}

pub fn make_euler_circuit<G, Ix>(
    ref_graph: &StableDiGraph<G::NodeWeight, G::EdgeWeight, Ix>,
    egraph: &EulerGraph<G>,
    source: G::NodeId,
) -> EulerCircuit<G>
where
    G: GraphBase<NodeId = NodeIndex<Ix>, EdgeId = EdgeIndex<Ix>> + NodeIndexable + Data + DataMap,
    G::NodeId: Hash + Eq,
    G::NodeWeight: Copy,
    G::EdgeWeight: Copy + Default + Debug + Measure + PartialEq + PartialOrd + Sum,
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

    let node_order = hierholzer_new::<G, G::EdgeWeight>(&vertex_edge_mapper, source);

    let ordered_node_weight_list: Vec<<G as Data>::NodeWeight> = node_order
        .iter()
        .filter_map(|node| ref_graph.node_weight(*node))
        .copied()
        .collect();
    let node_pair_list: Vec<(G::NodeId, G::NodeId)> = node_order
        .iter()
        .zip(node_order.iter().skip(1))
        .map(|(s, t)| (*s, *t))
        .collect();

    let edge_list: Vec<G::EdgeId> = node_pair_list
        .iter()
        .filter_map(|(s, t)| ref_graph.find_edge(*s, *t))
        .collect();

    let length = edge_list
        .iter()
        .filter_map(|e| ref_graph.edge_weight(*e))
        .copied()
        .sum();

    EulerCircuit {
        length,
        node_pair_list,
        edge_list,
        ordered_node_weight_list,
    }
}

fn hierholzer_new<G, E>(
    vertex_edge_mapper: &HashMap<G::NodeId, VecDeque<G::NodeId>>,
    source: G::NodeId,
) -> VecDeque<G::NodeId>
where
    G: GraphBase,
    G::NodeId: Hash + Eq + Debug,
    E: Copy + Measure + Default,
{
    let mut v_e_mapper = vertex_edge_mapper.clone();
    let mut curr_path: VecDeque<G::NodeId> = VecDeque::new();
    let mut circuit: VecDeque<G::NodeId> = VecDeque::new();

    curr_path.push_back(source);
    let mut current_vertex = source;

    while !curr_path.is_empty() {
        if let Some(adj_list) = v_e_mapper.get_mut(&current_vertex) {
            if !adj_list.is_empty() {
                curr_path.push_back(current_vertex);
                let next_vertex = adj_list
                    .pop_back()
                    .expect("As written, we are guaranteed a value here");
                let next_adj_list = v_e_mapper
                    .get_mut(&next_vertex)
                    .expect("As written, we are guaranteed a value here");
                next_adj_list.remove(
                    next_adj_list
                        .iter()
                        .position(|&node| node == current_vertex)
                        .unwrap(),
                );

                current_vertex = next_vertex;
            } else {
                circuit.push_back(current_vertex);
                current_vertex = curr_path
                    .pop_back()
                    .expect("As written, we are guaranteed a value here");
            }
        }
    }

    let ordered_circuit: VecDeque<G::NodeId> = circuit.into_iter().rev().collect();
    println!("{:?}", &ordered_circuit);
    ordered_circuit
}
