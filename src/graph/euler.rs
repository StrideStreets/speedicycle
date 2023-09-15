use crate::graph::{GraphBase, Measure};
use hashbag::HashBag;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    hash::Hash,
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
        return Self {
            length: E::default(),
            edges: HashSet::new(),
            vertices: HashSet::new(),
        };
    }
}

pub struct EulerCircuit<G, E>
where
    G: GraphBase,
    E: Measure + Copy + Default,
{
    pub length: E,
    pub pair_list: Vec<(G::NodeId, G::NodeId)>,
    pub edge_list: Vec<G::EdgeId>,
}

impl<G, E> EulerCircuit<G, E>
where
    G: GraphBase,
    E: Copy + Measure + Default,
{
    pub fn new() -> Self {
        return Self {
            length: E::default(),
            pair_list: Vec::new(),
            edge_list: Vec::new(),
        };
    }
}

pub fn make_euler_circuit<G, E>(
    egraph: &EulerGraph<G, E>,
    source: G::NodeId,
) -> Option<EulerCircuit<G, E>>
where
    G: GraphBase,
    G::NodeId: Hash + Eq,
    E: Measure + Copy + Default,
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

    return hierholzer::<G, E>(vertex_edge_mapper, source);
}

fn hierholzer<G, E>(
    mut v_e_mapper: HashMap<G::NodeId, VecDeque<G::NodeId>>,
    source: G::NodeId,
) -> Option<EulerCircuit<G, E>>
where
    G: GraphBase,
    G::NodeId: Hash + Eq,
    E: Copy + Measure + Default,
{
    let mut ordered_nodes = VecDeque::new();
    ordered_nodes.push_back(source);

    (v_e_mapper, ordered_nodes) = extract_circuit::<G, E>(v_e_mapper, ordered_nodes, source);

    //Pick up on implementation here, starting with EulerCircuit.cpp line 34
    return None;
}

fn extract_circuit<G, E>(
    mut v_e_mapper: HashMap<G::NodeId, VecDeque<G::NodeId>>,
    mut ordered_nodes: VecDeque<G::NodeId>,
    source: G::NodeId,
) -> (HashMap<G::NodeId, VecDeque<G::NodeId>>, VecDeque<G::NodeId>)
where
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

    return (v_e_mapper, ordered_nodes);
}
