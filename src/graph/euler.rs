use crate::graph::{GraphBase, Measure};
use std::collections::HashSet;

#[derive(Debug)]
pub struct EulerGraph<G, K>
where
    G: GraphBase,
    K: Measure + Copy + Default,
{
    pub length: K,
    pub edges: HashSet<(G::NodeId, G::NodeId)>,
    pub vertices: HashSet<G::NodeId>,
}

impl<G, K> EulerGraph<G, K>
where
    G: GraphBase,
    K: Copy + Measure + Default,
{
    pub fn new() -> Self {
        return Self {
            length: K::default(),
            edges: HashSet::new(),
            vertices: HashSet::new(),
        };
    }
}
