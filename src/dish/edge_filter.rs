use std::collections::{HashMap, HashSet};

use daggy::{Dag, NodeIndex};
use petgraph::{
    adj::List,
    visit::{EdgeRef, IntoEdgeReferences},
};
use serde::{Deserialize, Serialize};

use crate::dish::feed::Episode;

/**
 * Takes a list of episodes and returns the minimal set of edges
 * that can be used to construct a DAG with the same topological sort.
 */
pub(crate) fn adjacency_reduced_edges(
    sorted_episodes: &Vec<(usize, Vec<usize>)>,
) -> Vec<(usize, usize)> {
    let mut dag = Dag::<(), usize, usize>::new();

    let mut node_indices: HashMap<usize, NodeIndex<usize>> = HashMap::new();

    sorted_episodes.iter().for_each(|(number, pointers)| {
        let node_idx = dag.add_node(());
        node_indices.insert(*number, node_idx);
        pointers.iter().for_each(|reference| {
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
        .map(|(number, _)| *node_indices.get(number).expect("Missing node index"))
        .collect();

    let (intermediate, _) =
        petgraph::algo::tred::dag_to_toposorted_adjacency_list::<_, usize>(&dag, &toposort);
    let (tred, _tclos): (List<(), usize>, List<(), usize>) =
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CongressionalGraph {
    episodes: Vec<Episode>,
    adjacency_reduced_edges: Vec<(usize, usize)>,
    networks: Vec<Network>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Network {
    nodes: HashSet<usize>,
    edges: HashSet<(usize, usize)>,
}

impl Network {
    pub(crate) fn new() -> Self {
        Self {
            nodes: HashSet::new(),
            edges: HashSet::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Subnetworks {
    networks: Vec<Network>,
}

impl Subnetworks {
    pub(crate) fn new(networks: Vec<Network>) -> Self {
        Self { networks }
    }
}

impl FromIterator<(usize, usize)> for Subnetworks {
    fn from_iter<T: IntoIterator<Item = (usize, usize)>>(iter: T) -> Self {
        let mut networks: Vec<Network> = Vec::new();
        let mut visited: HashMap<usize, usize> = HashMap::new();

        for (source, target) in iter {
            let idx = visited
                .get(&source)
                .or_else(|| visited.get(&target))
                .map(|i| *i);
            if let Some(i) = idx {
                let network = &mut networks[i];
                network.nodes.insert(source);
                network.nodes.insert(target);
                network.edges.insert((source, target));
                visited.insert(source, i);
                visited.insert(target, i);
            } else {
                let mut network = Network::new();
                network.nodes.insert(source);
                network.nodes.insert(target);
                network.edges.insert((source, target));
                networks.push(network);
                visited.insert(source, networks.len() - 1);
            }
        }
        Subnetworks::new(networks)
    }
}

fn networks(tred: Vec<(usize, usize)>) -> Vec<Network> {
    Subnetworks::from_iter(tred.into_iter()).networks
}

pub(crate) fn analyze(
    episodes: Vec<Episode>,
    links: Vec<(usize, Vec<usize>)>,
) -> CongressionalGraph {
    let episodes = {
        let mut sorted = episodes;
        sorted.sort_by_key(|ep| ep.number);
        sorted
    };
    let tred = adjacency_reduced_edges(&links);
    let networks = networks(tred.clone());

    CongressionalGraph {
        episodes,
        networks,
        adjacency_reduced_edges: tred,
    }
}
