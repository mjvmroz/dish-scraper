use std::collections::HashMap;

use daggy::{Dag, NodeIndex};
use petgraph::{
    adj::List,
    visit::{EdgeRef, IntoEdgeReferences},
};

use crate::dish::feed::Episode;

/**
 * Takes a list of episodes and returns the minimal set of edges
 * that can be used to construct a DAG with the same topological sort.
 */
pub(crate) fn adjacency_reduced_edges(sorted_episodes: &Vec<Episode>) -> HashMap<usize, usize> {
    let mut dag = Dag::<(), usize, usize>::new();

    let mut node_indices: HashMap<usize, NodeIndex<usize>> = HashMap::new();

    sorted_episodes.iter().for_each(|episode| {
        let node_idx = dag.add_node(());
        node_indices.insert(episode.number, node_idx);
        episode.pointers.iter().for_each(|reference| {
            dag.add_edge(
                node_indices
                    .get(reference)
                    .expect("Out of order nodes")
                    .to_owned(),
                node_idx,
                1,
            )
            .expect("Failed to add edge: we assert no cycles rather than defend 🙃");
        });
    });

    let toposort: Vec<NodeIndex<usize>> = sorted_episodes
        .iter()
        .map(|e| *node_indices.get(&e.number).expect("Missing node index"))
        .collect();

    let (intermediate, _) =
        petgraph::algo::tred::dag_to_toposorted_adjacency_list::<_, usize>(&dag, &toposort);
    let (tred, _): (List<(), usize>, _) =
        petgraph::algo::tred::dag_transitive_reduction_closure(&intermediate);

    let index_numbers: HashMap<NodeIndex<usize>, usize> =
        node_indices.into_iter().map(|(k, v)| (v, k)).collect();

    tred.edge_references()
        .map(|e| {
            let source = index_numbers
                .get(&NodeIndex::new(e.source()))
                .expect("Unknown source node");
            let target = index_numbers
                .get(&NodeIndex::new(e.target()))
                .expect("Unknown target node");
            (*source, *target)
        })
        .collect()
}
