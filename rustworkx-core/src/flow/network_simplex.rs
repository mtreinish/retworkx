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

use std::hash::Hash;

use hashbrown::HashMap;
use petgraph::visit::{
    EdgeCount, EdgeRef, GraphBase, GraphProp, IntoEdgeReferences, IntoNodeReferences, NodeCount,
    NodeIndexable, NodeRef,
};

/// The return type for `network_simplex()`. It has two attributes, `cost` and `flow_edges`.
///
/// * `cost` - the found minimum cost flow that satisfies all demands
/// * `flow_edges` - A two layer `HashMap` that maps the flow edge. For
///     example, `flow_edges[u][v]` will be the flow edge `(u, v)`.
pub struct FlowReturn<G: GraphProp> {
    pub cost: i64,
    pub flow_edges: HashMap<G::NodeId, HashMap<G::NodeId, usize>>,
}

#[derive(Debug)]
struct SimplexState<G>
where
    G: GraphProp,
    G::NodeId: Eq + Hash,
{
    num_nodes: usize,
    node_map: HashMap<G::NodeId, usize>,
    edge_count: usize,
    edge_map: HashMap<G::EdgeId, usize>,
    edge_capacities: Vec<i64>,
    edge_weights: Vec<i64>,
    edge_sources: Vec<Option<G::NodeId>>,
    edge_targets: Vec<Option<G::NodeId>>,
    edge_flow: Vec<i64>,
    node_potentials: Vec<i64>,
    parents: Vec<Option<usize>>,
    parent_edges: Vec<Option<usize>>,
    subtree_size: Vec<usize>,
    next_node_dft: Vec<Option<usize>>,
    prev_node_dft: Vec<Option<usize>>,
    last_descendent_dft: Vec<usize>,
}

impl<G> SimplexState<G>
where
    G: GraphProp,
    G::NodeId: Eq + Hash,
{
    fn reduced_cost(&self, i: usize) -> i64 {
        let edge_source = self.edge_sources[i]
            .map(|x| self.node_map[&x])
            .unwrap_or_else(|| self.node_potentials.len() - 1);
        let edge_target = self.edge_targets[i]
            .map(|x| self.node_map[&x])
            .unwrap_or_else(|| self.node_potentials.len() - 1);
        let c = self.edge_weights[i] - self.node_potentials[edge_source]
            + self.node_potentials[edge_target];
        if self.edge_flow[i] == 0 {
            c
        } else {
            -c
        }
    }

    fn find_apex(&self, p: usize, q: usize) -> usize {
        let mut size_p = self.subtree_size[p];
        let mut size_q = self.subtree_size[q];
        let mut inner_p = p;
        let mut inner_q = q;
        loop {
            while size_p < size_q {
                inner_p = self.parents[inner_p].unwrap_or(self.num_nodes - 1);
                size_p = self.subtree_size[inner_p];
            }
            while size_p > size_q {
                inner_q = self.parents[inner_q].unwrap_or(self.num_nodes - 1);
                size_q = self.subtree_size[inner_q];
            }
            if size_p == size_q {
                if inner_p != inner_q {
                    inner_p = self.parents[inner_p].unwrap_or(self.num_nodes - 1);
                    size_p = self.subtree_size[inner_p];
                    inner_q = self.parents[inner_q].unwrap_or(self.num_nodes - 1);
                    size_q = self.subtree_size[inner_q];
                } else {
                    return inner_p;
                }
            }
        }
    }

    fn trace_path(&self, p: usize, w: usize) -> [Vec<usize>; 2] {
        let mut wn: Vec<usize> = vec![p; 1];
        let mut we = Vec::new();
        let mut loop_p = p;
        while loop_p != w {
            we.push(self.parent_edges[loop_p].unwrap_or(self.edge_count - 1));
            loop_p = self.parents[loop_p].unwrap_or(self.num_nodes - 1);
            wn.push(loop_p);
        }
        [wn, we]
    }

    fn find_cycle(&self, i: usize, p: usize, q: usize) -> [Vec<usize>; 2] {
        let w = self.find_apex(p, q);
        let [mut wn, mut we] = self.trace_path(p, w);
        wn.reverse();
        we.reverse();
        if we.len() != 1 || we.first() != Some(&i) {
            we.push(i);
        }
        let [mut wnr, mut wer] = self.trace_path(q, w);
        wnr.pop();
        wn.append(&mut wnr);
        we.append(&mut wer);
        [wn, we]
    }

    fn residual_capacity(&self, i: usize, p: usize) -> i64 {
        let source = match self.edge_sources[i] {
            Some(source) => self.node_map[&source],
            None => self.num_nodes - 1,
        };
        if source == p {
            self.edge_capacities[i] - self.edge_flow[i]
        } else {
            self.edge_flow[i]
        }
    }

    fn find_leaving_edge(&self, wn: &[usize], we: &[usize]) -> [usize; 3] {
        let (j, s) = we
            .iter()
            .rev()
            .zip(wn.iter().rev())
            .min_by_key(|x| self.residual_capacity(*x.0, *x.1))
            // |a_i_p, b_i_p| {
            //     let a_cap = self.residual_capacity(*a_i_p.0, *a_i_p.1);
            //     let b_cap = self.residual_capacity(*b_i_p.0, *b_i_p.1);
            //     a_cap.partial_cmp(&b_cap).unwrap()
            // })
            .unwrap();
        let source = match self.edge_sources[*j] {
            Some(source) => self.node_map[&source],
            None => self.num_nodes - 1,
        };
        let t = if source == *s {
            self.edge_targets[*j]
        } else {
            self.edge_sources[*j]
        };
        let out_t = t.map(|t| self.node_map[&t]).unwrap_or(self.num_nodes - 1);
        [*j, *s, out_t]
    }

    fn remove_edge(&mut self, s: Option<usize>, t: usize) {
        let size_t = self.subtree_size[t];
        let prev_t = match self.prev_node_dft[t] {
            Some(val) => val,
            None => self.num_nodes - 1,
        };
        let last_t = self.last_descendent_dft[t];
        let next_last_t = match self.next_node_dft[last_t] {
            Some(val) => val,
            None => self.num_nodes - 1,
        };
        self.parents[t] = None;
        self.parent_edges[t] = None;
        self.next_node_dft[prev_t] = Some(next_last_t);
        self.prev_node_dft[next_last_t] = Some(prev_t);
        self.next_node_dft[last_t] = Some(t);
        self.prev_node_dft[t] = Some(last_t);
        let mut loop_s = s;
        while loop_s.is_some() {
            let inner_s = loop_s.unwrap();
            self.subtree_size[inner_s] -= size_t;
            if self.last_descendent_dft[inner_s] == last_t {
                self.last_descendent_dft[inner_s] = prev_t;
            }
            loop_s = self.parents[inner_s];
        }
    }

    fn augment_flow(&mut self, wn: &[usize], we: &[usize], f: i64) {
        for (i, p) in we.iter().zip(wn.iter()) {
            let source = match self.edge_sources[*i] {
                Some(source) => self.node_map[&source],
                None => self.num_nodes - 1,
            };
            if source == *p {
                self.edge_flow[*i] += f;
            } else {
                self.edge_flow[*i] -= f;
            }
        }
    }

    fn make_root(&mut self, q: usize) {
        let mut ancestors: Vec<usize> = Vec::new();
        let mut loop_q = Some(q);
        while loop_q.is_some() {
            ancestors.push(loop_q.unwrap());
            loop_q = self.parents[loop_q.unwrap()];
        }
        ancestors.reverse();
        for i in 0..ancestors.len() - 1 {
            let p = ancestors[i];
            let inner_q = ancestors[i + 1];
            let size_p = self.subtree_size[p];
            let mut last_p = self.last_descendent_dft[p];
            let prev_q = self.prev_node_dft[p];
            let last_q = self.last_descendent_dft[inner_q];
            let next_last_q = self.next_node_dft[last_q];
            self.parents[p] = Some(inner_q);
            self.parents[q] = None;
            self.parent_edges[p] = self.parent_edges[inner_q];
            self.parent_edges[inner_q] = None;
            self.subtree_size[p] = size_p - self.subtree_size[inner_q];
            self.subtree_size[inner_q] = size_p;
            self.next_node_dft[prev_q.unwrap_or(self.num_nodes - 1)] = next_last_q;
            self.prev_node_dft[next_last_q.unwrap_or(self.num_nodes - 1)] = prev_q;
            self.next_node_dft[last_q] = Some(inner_q);
            self.prev_node_dft[q] = Some(last_q);
            if last_p == last_q {
                self.last_descendent_dft[p] = prev_q.unwrap_or(self.num_nodes - 1);
                last_p = prev_q.unwrap_or(self.num_nodes - 1);
            }
            self.prev_node_dft[p] = Some(last_q);
            self.next_node_dft[last_q] = Some(p);
            self.next_node_dft[last_p] = Some(q);
            self.prev_node_dft[inner_q] = Some(last_p);
            self.last_descendent_dft[inner_q] = last_p;
        }
    }

    fn add_edge(&mut self, i: usize, p: usize, q: usize) {
        let last_p = self.last_descendent_dft[p];
        let next_last_p = self.next_node_dft[last_p];
        let size_q = self.subtree_size[q];
        let last_q = self.last_descendent_dft[q];
        self.parents[q] = Some(p);
        self.parent_edges[q] = Some(i);
        self.next_node_dft[last_p] = Some(q);
        self.prev_node_dft[q] = Some(last_p);
        self.prev_node_dft[next_last_p.unwrap_or(self.num_nodes - 1)] = Some(last_q);
        self.next_node_dft[last_q] = next_last_p;
        let mut loop_p = Some(p);
        while loop_p.is_some() {
            self.subtree_size[loop_p.unwrap()] += size_q;
            if self.last_descendent_dft[loop_p.unwrap()] == last_p {
                self.last_descendent_dft[loop_p.unwrap()] = last_q;
            }
            loop_p = self.parents[loop_p.unwrap()];
        }
    }

    fn update_potentials(&mut self, i: usize, p: usize, q: usize) {
        let target = match self.edge_targets[i] {
            Some(t) => self.node_map[&t],
            None => self.num_nodes - 1,
        };
        let d = if q == target {
            self.node_potentials[p] - self.edge_weights[i] - self.node_potentials[q]
        } else {
            self.node_potentials[p] + self.edge_weights[i] - self.node_potentials[q]
        };
        self.node_potentials[p] += d;
        let l = self.last_descendent_dft[p];
        let mut loop_p = p;
        while loop_p != l {
            loop_p = self.next_node_dft[loop_p].unwrap_or(self.num_nodes - 1);
            self.node_potentials[q] += d;
        }
    }
}

/// Find a minimum cost flow satisfying all the demands in a directed graph
///
/// This is a primal network simplex algorithm that uses the leaving arc rule
/// to prevent cycling
///
/// ``graph`` is treated as a digraph (the trait bound don't force it but
/// for undirected inputs it will be treated as one) where edges have cost and
/// capacity and noes have demand, in other words nodes want to send or receive
/// some flow. A negative demands means that the node wants to send flow,
/// a positive demand means the node wants to receive flow. A flow on the
/// digraph ``graph`` satisfies all demand if the net flow into each node is
/// equal
///
/// Note this function uses signed integers for edge weights, capacity, and
/// node demands. This is to avoid floating point errors preventing causing
/// the algorithm to fail as it is particularly sensitive to this. If you need
/// to use a floating point number it is recommend to multiply the float by
/// a constant factor and cast it to an int (such as
/// ``demand=lambda x: int(x * 100)``) to approximate the floating point value.
///
/// This implementation is based on the NetworkX ``network_simplex()``
/// function [^networkx_simplex], which is based on [^kiraly] and [^barr].
///
/// [^networkx_simplex]: <https://github.com/networkx/networkx/blob/6971d845e25aa4b76879831bb85ac2fa23ae0c9e/networkx/algorithms/flow/networksimplex.py#L330>
///
/// [^kiraly]: Z. Kiraly, P. Kovacs.
///     Efficient implementation of minimum-cost flow algorithms.
///     Acta Universitatis Sapientiae, Informatica 4(1):67--118. 2012.
///
/// [^barr]: R. Barr, F. Glover, D. Klingman.
///     Enhancement of spanning tree labeling procedures for network
///     optimization.
///     INFOR 17(1):16--34. 1979.
///
/// Arguments:
///
/// * `graph` - The input graph object to run the algorithm on
/// * `demand` - A function which will receive a borrowed ``NodeWeight``
///     from ``graph`` and is expected to return the demand to indicate how
///     much flow that node wants to send (a negative value) or receive
///     (a positive value). Note that the sum of all demands in the graph
///     should be equal to 0 or the problem is not feasible.
/// * `capacity` - A function which will receive a borrowed ``EdgeWeight``
///     from ``graph`` and is expected to the capacity, or how much flow
///     the edge can support.
/// * `weight` - A function which will receive a borrowed ``EdgeWeight`` from
///     ``graph`` and is expected to return the weight of the edge (i.e. the
///     cost incurred by sending flow on the edge).
///
/// # Example
/// ```rust
/// use std::convert::Infallible;
/// use rustworkx_core::petgraph;
/// use rustworkx_core::flow::network_simplex;
///
/// let mut graph = petgraph::graph::DiGraph::<i64, [i64; 2]>::new();
/// // Node A
/// let A = graph.add_node(-5);
/// // Node D
/// let D = graph.add_node(5);//
/// // Node B
/// let B = graph.add_node(0);
/// // Node C
/// let C = graph.add_node(0);
///
/// graph.add_edge(A, B, [3, 4]);
/// graph.add_edge(A, C, [6, 10]);
/// graph.add_edge(B, D, [1, 9]);
/// graph.add_edge(C, D, [2, 5]);
/// let result = network_simplex(
///     &graph,
///     |n| Ok::<i64, Infallible>(*n as i64),
///     |e: &[i64; 2]| Ok(e[1] as i64),
///     |e: &[i64; 2]| Ok(e[0] as i64),
/// )
/// .unwrap()
/// .unwrap();
///
/// assert_eq!(result.cost, 24);
/// ```
pub fn network_simplex<G, F, C, W, E>(
    graph: G,
    mut demand: F,
    mut capacity: C,
    mut weight: W,
) -> Result<Option<FlowReturn<G>>, E>
where
    G: NodeIndexable
        + IntoNodeReferences
        + IntoEdgeReferences
        + NodeCount
        + EdgeCount
        + GraphProp
        + GraphBase
        + std::fmt::Debug,
    <G as GraphBase>::NodeId: Eq + Hash + std::fmt::Debug,
    <G as GraphBase>::EdgeId: Eq + Hash + std::fmt::Debug,
    F: FnMut(&G::NodeWeight) -> Result<i64, E>,
    C: FnMut(&G::EdgeWeight) -> Result<i64, E>,
    W: FnMut(&G::EdgeWeight) -> Result<i64, E>,
{
    let num_nodes = graph.node_count();
    let edge_count = graph.edge_count();
    if num_nodes == 0 || edge_count == 0 {
        return Ok(None);
    }
    let node_indices: Vec<G::NodeId> = graph.node_identifiers().collect();
    let node_map: HashMap<G::NodeId, usize> = node_indices
        .iter()
        .enumerate()
        .map(|(index, val)| (*val, index))
        .collect();
    let mut demands: Vec<i64> = Vec::with_capacity(num_nodes);
    let mut demand_abs_max = 0;
    for n in graph.node_references() {
        let demand_val = demand(n.weight())?;
        //        if demand_val.is_infinite() {
        //            return Ok(None);
        //        }
        demands.push(demand_val);
        let abs_demand = demand_val.abs();
        if abs_demand > demand_abs_max {
            demand_abs_max = abs_demand;
        }
    }
    let mut edge_indices: Vec<G::EdgeId> = Vec::with_capacity(edge_count);
    let mut edge_capacities: Vec<i64> = Vec::with_capacity(edge_count);
    let mut edge_weights: Vec<i64> = Vec::with_capacity(edge_count);
    let mut edge_sources: Vec<Option<G::NodeId>> = Vec::with_capacity(edge_count + num_nodes);
    let mut edge_targets: Vec<Option<G::NodeId>> = Vec::with_capacity(edge_count + num_nodes);
    let mut capacity_sum: i64 = 0;
    let mut weight_sum: i64 = 0;
    for edge in graph.edge_references() {
        let capacity = capacity(edge.weight())?;
        let source = edge.source();
        let target = edge.target();
        if source != target && capacity != 0 {
            edge_indices.push(edge.id());
            if capacity < 0 {
                return Ok(None);
            }
            capacity_sum += capacity;
            //        if !capacity.is_infinite() {
            //            capacity_sum += capacity;
            //        }
            edge_capacities.push(capacity);
            let weight = weight(edge.weight())?;
            weight_sum += weight.abs();
            edge_weights.push(weight);
            edge_sources.push(Some(source));
            edge_targets.push(Some(target));
        }
    }
    let edge_map: HashMap<G::EdgeId, usize> = edge_indices
        .iter()
        .enumerate()
        .map(|(index, val)| (*val, index))
        .collect();
    // Initialize
    for (index, demand) in demands.iter().enumerate() {
        if *demand > 0 {
            edge_sources.push(None);
            edge_targets.push(Some(node_indices[index]));
        } else {
            edge_sources.push(Some(node_indices[index]));
            edge_targets.push(None);
        }
    }
    let max_value = [capacity_sum, weight_sum, demand_abs_max]
        .into_iter()
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let faux_infite = if max_value == 0 { 1 } else { max_value * 3 };
    edge_weights.append(&mut vec![faux_infite; num_nodes]);
    edge_capacities.append(&mut vec![faux_infite; num_nodes]);
    // Pivot Loop
    let mut edge_flow = vec![0; edge_count];
    edge_flow.extend(demands.iter().map(|x| x.abs()));
    let node_potentials: Vec<i64> = demands
        .iter()
        .map(|d| if *d <= 0 { faux_infite } else { -faux_infite })
        .collect();
    let mut parents: Vec<Option<usize>> = vec![Some(num_nodes); num_nodes];
    parents.push(None);
    let parent_edges: Vec<Option<usize>> = (edge_count..edge_count + num_nodes).map(Some).collect();
    let mut subtree_size: Vec<usize> = vec![1; num_nodes];
    subtree_size.push(num_nodes + 1);
    let mut next_node_dft: Vec<Option<usize>> = (1..num_nodes).map(Some).collect();
    next_node_dft.extend([None, Some(0)].into_iter());
    let mut tmp: Vec<Option<usize>> = (0..num_nodes).map(Some).collect();
    let mut prev_node_dft: Vec<Option<usize>> = vec![None; 1];
    prev_node_dft.append(&mut tmp);
    let mut last_descendent_dft: Vec<usize> = (0..num_nodes).collect();
    last_descendent_dft.push(num_nodes - 1);
    let mut state: SimplexState<G> = SimplexState {
        num_nodes,
        node_map,
        edge_count,
        edge_map,
        edge_capacities,
        edge_weights,
        edge_sources,
        edge_targets,
        edge_flow,
        node_potentials,
        parents,
        parent_edges,
        subtree_size,
        next_node_dft,
        prev_node_dft,
        last_descendent_dft,
    };

    // piot loop
    #[allow(non_snake_case)]
    let B: usize = (edge_count as f64).sqrt().ceil() as usize;
    #[allow(non_snake_case)]
    let M: usize = (edge_count + B - 1) / B;
    let mut m = 0;
    let mut f = 0;
    while m < M {
        let mut l = f + B;
        let edges: Vec<usize> = if l <= edge_count {
            (f..l).collect()
        } else {
            l -= edge_count;
            let mut tmp: Vec<usize> = (f..edge_count).collect();
            tmp.extend(0..l);
            tmp
        };
        f = l;
        let i = edges
            .into_iter()
            .min_by_key(|x| state.reduced_cost(*x))
            .unwrap();
        let c = state.reduced_cost(i);
        if c >= 0 {
            m += 1;
        } else {
            let mut p: usize;
            let mut q: usize;
            if state.edge_flow[i] == 0 {
                p = match state.edge_sources[i] {
                    Some(source) => state.node_map[&source],
                    None => num_nodes - 1,
                };
                q = match state.edge_targets[i] {
                    Some(target) => state.node_map[&target],
                    None => num_nodes - 1,
                };
            } else {
                p = match state.edge_targets[i] {
                    Some(target) => state.node_map[&target],
                    None => num_nodes - 1,
                };
                q = match state.edge_sources[i] {
                    Some(source) => state.node_map[&source],
                    None => num_nodes - 1,
                };
            }
            let [wn, we] = state.find_cycle(i, p, q);
            let [j, mut s, mut t] = state.find_leaving_edge(&wn, &we);
            state.augment_flow(&wn, &we, state.residual_capacity(j, s));
            if i != j {
                if state.parents[t] != Some(s) {
                    std::mem::swap(&mut t, &mut s);
                }
                if we.iter().position(|x| *x == i).unwrap()
                    > we.iter().position(|x| *x == j).unwrap()
                {
                    std::mem::swap(&mut p, &mut q);
                }
                state.remove_edge(Some(s), t);
                state.make_root(q);
                state.add_edge(i, p, q);
                state.update_potentials(i, p, q);
            }
            m = 0;
        }
    }
    for flow in &state.edge_flow[state.edge_flow.len() - num_nodes..] {
        if *flow != 0 {
            return Ok(None);
        }
    }
    for i in 0..edge_count {
        if state.edge_flow[i] * 2 >= faux_infite {
            return Ok(None);
        }
    }
    state.edge_flow.drain(edge_count..);
    let cost = state
        .edge_weights
        .iter()
        .zip(state.edge_flow.iter())
        .map(|(w, x)| w * x)
        .sum();
    let flow_edges = HashMap::with_capacity(num_nodes);
    Ok(Some(FlowReturn { cost, flow_edges }))
}

#[cfg(test)]
mod tests {

    use crate::flow::{network_simplex, FlowReturn};
    use petgraph::graph::{DiGraph, NodeIndex};
    use std::convert::Infallible;

    #[test]
    fn test_google_example_1() {
        // Taken from https://developers.google.com/optimization/flow/mincostflow
        let start_nodes = [0, 0, 1, 1, 1, 2, 2, 3, 4];
        let end_nodes = [1, 2, 2, 3, 4, 3, 4, 4, 2];
        let capacities = [15, 8, 20, 4, 10, 15, 4, 20, 5];
        let unit_costs = [4, 4, 2, 2, 6, 1, 3, 2, 3];
        let supplies = [20, 0, 0, -5, -15];
        let mut graph: DiGraph<isize, [usize; 2]> =
            DiGraph::with_capacity(supplies.len(), start_nodes.len());
        for i in 0..supplies.len() {
            graph.add_node(-1 * supplies[i]);
        }
        for i in 0..start_nodes.len() {
            graph.add_edge(
                NodeIndex::new(start_nodes[i]),
                NodeIndex::new(end_nodes[i]),
                [unit_costs[i], capacities[i]],
            );
        }
        let res = network_simplex(
            &graph,
            |n| Ok::<i64, Infallible>(*n as i64),
            |e: &[usize; 2]| Ok(e[1] as i64),
            |e: &[usize; 2]| Ok(e[0] as i64),
        )
        .unwrap()
        .unwrap();
        assert_eq!(res.cost, 150);
    }
}
