use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::{BinaryHeap, HashMap};

use std::hash::Hash;

use petgraph::{
    algo::Measure,
    visit::{Data, EdgeRef, GraphBase, IntoEdgeReferences, IntoEdges, VisitMap, Visitable},
};

use super::scored::MinScored;

//NOTE: This algo has been modified to stop considering nodes after distance reaches a provided maximum
//It has also had the provision for a target node removed

/// \[Generic\] Dijkstra's shortest path algorithm.
///
/// Compute the length of the shortest path from `start` to every reachable
/// node.
///
/// The graph should be `Visitable` and implement `IntoEdges`. The function
/// `edge_cost` should return the cost for a particular edge, which is used
/// to compute path costs. Edge costs must be non-negative.
///
/// If `goal` is not `None`, then the algorithm terminates once the `goal` node's
/// cost is calculated.
///
/// Returns a `HashMap` that maps `NodeId` to path cost.
/// # Example
/// ```rust
/// use petgraph::Graph;
/// use petgraph::algo::dijkstra;
/// use petgraph::prelude::*;
/// use std::collections::HashMap;
///
/// let mut graph: Graph<(), (), Directed> = Graph::new();
/// let a = graph.add_node(()); // node with no weight
/// let b = graph.add_node(());
/// let c = graph.add_node(());
/// let d = graph.add_node(());
/// let e = graph.add_node(());
/// let f = graph.add_node(());
/// let g = graph.add_node(());
/// let h = graph.add_node(());
/// // z will be in another connected component
/// let z = graph.add_node(());
///
/// graph.extend_with_edges(&[
///     (a, b),
///     (b, c),
///     (c, d),
///     (d, a),
///     (e, f),
///     (b, e),
///     (f, g),
///     (g, h),
///     (h, e),
/// ]);
/// // a ----> b ----> e ----> f
/// // ^       |       ^       |
/// // |       v       |       v
/// // d <---- c       h <---- g
///
/// let expected_res: HashMap<NodeIndex, usize> = [
///     (a, 3),
///     (b, 0),
///     (c, 1),
///     (d, 2),
///     (e, 1),
///     (f, 2),
///     (g, 3),
///     (h, 4),
/// ].iter().cloned().collect();
/// let res = dijkstra(&graph, b, None, |_| 1);
/// assert_eq!(res, expected_res);
/// // z is not inside res because there is not path from b to z.
/// ```
pub fn modified_dijkstra<G, F, K>(
    graph: G,
    start: G::NodeId,
    mut edge_cost: F,
    max_dist: K,
) -> (
    HashMap<G::NodeId, K>,
    HashMap<G::NodeId, G::NodeId>,
    HashMap<G::NodeId, Vec<G::NodeId>>,
)
where
    G: IntoEdges + Visitable + Data<EdgeWeight = K>,
    G::NodeId: Eq + Hash,
    F: FnMut(G::EdgeRef) -> K,
    K: Measure + Copy,
{
    let mut visited = graph.visit_map();
    let mut scores = HashMap::new();
    let mut predecessor = HashMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score = K::default();
    scores.insert(start, zero_score);
    visit_next.push(MinScored(zero_score, start));
    while let Some(MinScored(node_score, node)) = visit_next.pop() {
        //Experimenting with removing conditional here. That is, if we get to a point where the
        //distance is greater than the max, we remove the node from the predecessor list and
        //stop the iteration.
        if node_score > max_dist {
            predecessor.remove(&node);
            visited.visit(node);
        }
        if visited.is_visited(&node) {
            continue;
        }
        for edge in graph.edges(node) {
            let next = edge.target();
            if visited.is_visited(&next) {
                continue;
            }

            let next_score = node_score + edge_cost(edge);

            match scores.entry(next) {
                Occupied(ent) => {
                    if next_score < *ent.get() {
                        *ent.into_mut() = next_score;
                        visit_next.push(MinScored(next_score, next));
                        predecessor.insert(next.clone(), node.clone());
                    }
                }
                Vacant(ent) => {
                    ent.insert(next_score);
                    visit_next.push(MinScored(next_score, next));
                    predecessor.insert(next.clone(), node.clone());
                }
            }
        }
        visited.visit(node);
    }

    let mut predecessor_tree: HashMap<<G as GraphBase>::NodeId, Vec<<G as GraphBase>::NodeId>> =
        HashMap::new();

    predecessor
        .clone()
        .into_iter()
        .for_each(|(node, prev)| match predecessor_tree.entry(prev) {
            Occupied(entry) => {
                entry.into_mut().push(node.clone());
            }
            Vacant(entry) => {
                entry.insert(vec![node.clone()]);
            }
        });

    return (scores, predecessor, predecessor_tree);
}
