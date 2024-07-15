// Licensed under the Apache License, Version 2.0 (the "License"); you may
// not use this file except in compliance with the License. You may obtain
// a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
// License for the specific language governing permissions and limitations
// under the License.

use hashbrown::HashSet;
use petgraph::visit::{IntoNeighborsDirected, NodeCount, Visitable};

/// Returns all nodes at a fixed `distance` from `source` in `G`.
/// Args:
///     `graph`:
///     `source`:
///     `distance`:
pub fn descendants_at_distance<G>(graph: G, source: G::NodeId, distance: usize) -> Vec<G::NodeId>
where
    G: Visitable + IntoNeighborsDirected + NodeCount,
    G::NodeId: std::cmp::Eq + std::hash::Hash,
{
    let mut current_layer: Vec<G::NodeId> = vec![source];
    let mut layers: usize = 0;
    let mut visited: HashSet<G::NodeId> = HashSet::with_capacity(graph.node_count());
    visited.insert(source);
    while !current_layer.is_empty() && layers < distance {
        let mut next_layer: Vec<G::NodeId> = Vec::new();
        for node in current_layer {
            for child in graph.neighbors_directed(node, petgraph::Outgoing) {
                if !visited.contains(&child) {
                    visited.insert(child);
                    next_layer.push(child);
                }
            }
        }
        current_layer = next_layer;
        layers += 1;
    }
    current_layer
}

#[cfg(test)]
mod tests {
    use super::*;
    use petgraph::graph::NodeIndex;
    use petgraph::Graph;

    #[test]
    fn test_descendants_at_distance_empty_graph() {
        let graph = Graph::<(), ()>::new();
        let source = NodeIndex::new(0);
        let distance = 1;
        let result = descendants_at_distance(&graph, source, distance);
        assert!(result.is_empty());
    }

    #[test]
    fn test_descendants_at_distance_single_node() {
        let mut graph = Graph::<(), ()>::new();
        let source = graph.add_node(());
        let distance = 1;
        let result = descendants_at_distance(&graph, source, distance);
        assert!(result.is_empty());
    }

    #[test]
    fn test_descendants_at_distance_simple_graph() {
        let mut graph = Graph::<(), ()>::new();
        let node0 = graph.add_node(());
        let node1 = graph.add_node(());
        let node2 = graph.add_node(());
        let node3 = graph.add_node(());
        graph.add_edge(node0, node1, ());
        graph.add_edge(node1, node2, ());
        graph.add_edge(node2, node3, ());

        let result = descendants_at_distance(&graph, node0, 1);
        assert_eq!(result, vec![node1]);

        let result = descendants_at_distance(&graph, node0, 2);
        assert_eq!(result, vec![node2]);

        let result = descendants_at_distance(&graph, node0, 3);
        assert_eq!(result, vec![node3]);

        let result = descendants_at_distance(&graph, node0, 4);
        assert!(result.is_empty());
    }
}
