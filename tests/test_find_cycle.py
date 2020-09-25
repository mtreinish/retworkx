# Licensed under the Apache License, Version 2.0 (the "License"); you may
# not use this file except in compliance with the License. You may obtain
# a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations
# under the License.

import unittest

import retworkx


class TestFindCycle(unittest.TestCase):

    def setUp(self):
        self.edges = [(3, 0), (0, 1), (1, 0), (1, 0), (2, 1), (3, 1)]

    def test_graph_no_cycle(self):
        graph = retworkx.PyGraph()
        graph.extend_from_edge_list(self.edges)
        res = retworkx.graph_find_cycle(graph, 3)
        self.assertEqual([], res)
        
    def test_graph_cycle(self):
        graph = retworkx.PyGraph()
        graph.extend_from_edge_list(self.edges)
        graph.add_edge(2, 0, None)
        graph.extend_from_edge_list(self.edges)
        res = retworkx.graph_find_cycle(graph, 0)
        self.assertEqual([(0, 1), (1, 2), (2, 0)], res)

    def test_digraph_no_cycle(self):
        graph = retworkx.PyDiGraph()
        graph.extend_from_edge_list(self.edges)
        res = retworkx.digraph_find_cycle(graph, 3)
        self.assertEqual([], res)
        
    def test_digraph_cycle(self):
        graph = retworkx.PyDiGraph()
        graph.extend_from_edge_list(self.edges)
        graph.add_edge(2, 0, None)
        graph.extend_from_edge_list(self.edges)
        res = retworkx.digraph_find_cycle(graph, 0)
        self.assertEqual([(0, 1), (1, 2), (2, 0)], res)
