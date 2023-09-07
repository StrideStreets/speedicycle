use petgraph::{
    algo::Measure,
    visit::{Data, GraphBase, IntoEdges, NodeIndexable, Visitable},
};

pub trait FindEdge<G>
where
    G: NodeIndexable + Data,
{
    fn find_edge(
        &self,
        a: <G as GraphBase>::NodeId,
        b: <G as GraphBase>::NodeId,
    ) -> Option<<G as GraphBase>::EdgeId>;

    fn remove_edge(&self, e: <G as GraphBase>::EdgeId) -> Option<<G as Data>::EdgeWeight>;

    fn add_edge(
        &self,
        a: <G as GraphBase>::NodeId,
        b: <G as GraphBase>::NodeId,
        w: <G as Data>::EdgeWeight,
    ) -> <G as GraphBase>::EdgeId;
}

pub struct Path<G, K>
where
    G: IntoEdges + Visitable + Data<EdgeWeight = K> + NodeIndexable + FindEdge<G>,
    K: Measure + Copy,
{
    pub length: K,
    pub sequence: Vec<<G as GraphBase>::NodeId>,
}

impl<G, K> Path<G, K>
where
    G: IntoEdges + Visitable + Data<EdgeWeight = K> + NodeIndexable + FindEdge<G>,
    K: Measure + Copy,
{
    pub fn new() -> Self {
        Self {
            length: K::default(),
            sequence: Vec::new(),
        }
    }
}
