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

use crate::digraph;
use rustworkx_core::flow;

use hashbrown::HashMap;

use pyo3::prelude::*;
use pyo3::Python;

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
/// function [1], which is based on [2] and [3].
///
/// [1] https://github.com/networkx/networkx/blob/6971d845e25aa4b76879831bb85ac2fa23ae0c9e/networkx/algorithms/flow/networksimplex.py#L330
///
/// [2] Z. Kiraly, P. Kovacs.
///     Efficient implementation of minimum-cost flow algorithms.
///     Acta Universitatis Sapientiae, Informatica 4(1):67--118. 2012.
/// [3] R. Barr, F. Glover, D. Klingman.
///     Enhancement of spanning tree labeling procedures for network
///     optimization.
///     INFOR 17(1):16--34. 1979.
///
/// :param PyDiGraph graph: The input graph object to run the algorithm on
/// :param demand: A callable which will receive the node data payload
///     from ``graph`` and is expected to return the demand to indicate how
///     much flow that node wants to send (a negative value) or receive
///     (a positive value). Note that the sum of all demands in the graph
///     should be equal to 0 or the problem is not feasible.
/// :param capacity: A callable which will receive the edge data payload
///     from ``graph`` and is expected to the capacity, or how much flow
///     the edge can support.
/// * `weight` - A function which will receive the edge data payload from
///     ``graph`` and is expected to return the weight of the edge (i.e. the
///     cost incurred by sending flow on the edge).
#[pyfunction]
#[pyo3(text_signature = "(graph, demand, capacity, weight/)")]
pub fn network_simplex(
    py: Python,
    graph: &digraph::PyDiGraph,
    demand: PyObject,
    capacity: PyObject,
    weight: PyObject,
) -> PyResult<Option<(i64, HashMap<usize, HashMap<usize, i64>>)>> {
    let demand_fn = |n: &PyObject| -> PyResult<i64> {
        let res = demand.as_ref(py).call1((n,))?;
        res.extract()
    };
    let capacity_fn = |e: &PyObject| -> PyResult<i64> {
        let res = capacity.as_ref(py).call1((e,))?;
        res.extract()
    };
    let weight_fn = |e: &PyObject| -> PyResult<i64> {
        let res = weight.as_ref(py).call1((e,))?;
        res.extract()
    };

    let flow = flow::network_simplex(&graph.graph, demand_fn, capacity_fn, weight_fn)?;
    match flow {
        Some(flow) => Ok(Some((flow.cost, HashMap::new()))),
        None => Ok(None),
    }
}
