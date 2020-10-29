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


class TestEdges(unittest.TestCase):

    def test_get_edge_data(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        graph.add_edge(node_a, node_b, "Edgy")
        res = graph.get_edge_data(node_a, node_b)
        self.assertEqual("Edgy", res)

    def test_get_all_edge_data(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        graph.add_edge(node_a, node_b, "Edgy")
        graph.add_edge(node_a, node_b, 'b')
        res = graph.get_all_edge_data(node_a, node_b)
        self.assertIn('b', res)
        self.assertIn('Edgy', res)

    def test_no_edge(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        self.assertRaises(retworkx.NoEdgeBetweenNodes, graph.get_edge_data,
                          node_a, node_b)

    def test_no_edge_get_all_edge_data(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        self.assertRaises(retworkx.NoEdgeBetweenNodes, graph.get_all_edge_data,
                          node_a, node_b)

    def test_has_edge(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        graph.add_edge(node_a, node_b, {})
        self.assertTrue(graph.has_edge(node_a, node_b))
        self.assertTrue(graph.has_edge(node_b, node_a))

    def test_has_edge_no_edge(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        self.assertFalse(graph.has_edge(node_a, node_b))

    def test_edges(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        graph.add_edge(node_a, node_b, "Edgy")
        node_c = graph.add_node('c')
        graph.add_edge(node_b, node_c, "Super edgy")
        self.assertEqual(["Edgy", "Super edgy"], graph.edges())

    def test_edges_empty(self):
        graph = retworkx.PyGraph()
        graph.add_node('a')
        self.assertEqual([], graph.edges())

    def test_add_duplicates(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('a')
        graph.add_edge(node_a, node_b, 'a')
        graph.add_edge(node_a, node_b, 'b')
        self.assertEqual(['a', 'b'], graph.edges())

    def test_remove_no_edge(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        self.assertRaises(retworkx.NoEdgeBetweenNodes, graph.remove_edge,
                          node_a, node_b)

    def test_remove_edge_single(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        graph.add_edge(node_a, node_b, 'edgy')
        graph.remove_edge(node_a, node_b)
        self.assertEqual([], graph.edges())

    def test_remove_multiple(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        graph.add_edge(node_a, node_b, 'edgy')
        graph.add_edge(node_a, node_b, 'super_edgy')
        graph.remove_edge_from_index(0)
        self.assertEqual(['super_edgy'], graph.edges())

    def test_remove_edge_from_index(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        graph.add_edge(node_a, node_b, 'edgy')
        graph.remove_edge_from_index(0)
        self.assertEqual([], graph.edges())

    def test_remove_edge_no_edge(self):
        graph = retworkx.PyGraph()
        graph.add_node('a')
        graph.remove_edge_from_index(0)
        self.assertEqual([], graph.edges())

    def test_remove_edges_from(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        node_c = graph.add_node('c')
        graph.add_edge(node_a, node_b, 'edgy')
        graph.add_edge(node_a, node_c, 'super_edgy')
        graph.remove_edges_from([(node_a, node_b), (node_a, node_c)])
        self.assertEqual([], graph.edges())

    def test_remove_edges_from_invalid(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        node_c = graph.add_node('c')
        graph.add_edge(node_a, node_b, 'edgy')
        graph.add_edge(node_a, node_c, 'super_edgy')
        with self.assertRaises(retworkx.NoEdgeBetweenNodes):
            graph.remove_edges_from([(node_b, node_c), (node_a, node_c)])

    def test_degree(self):
        graph = retworkx.PyGraph()
        node_a = graph.add_node('a')
        node_b = graph.add_node('b')
        graph.add_edge(node_a, node_b, "Edgy")
        node_c = graph.add_node('c')
        graph.add_edge(node_b, node_c, "Super edgy")
        self.assertEqual(2, graph.degree(node_b))

    def test_add_edge_from(self):
        graph = retworkx.PyGraph()
        nodes = list(range(4))
        graph.add_nodes_from(nodes)
        edge_list = [(0, 1, 'a'), (1, 2, 'b'), (0, 2, 'c'), (2, 3, 'd'),
                     (0, 3, 'e')]
        res = graph.add_edges_from(edge_list)
        self.assertEqual(len(res), 5)
        self.assertEqual(['a', 'b', 'c', 'd', 'e'], graph.edges())
        self.assertEqual(3, graph.degree(0))
        self.assertEqual(2, graph.degree(1))
        self.assertEqual(3, graph.degree(2))
        self.assertEqual(2, graph.degree(3))

    def test_add_edge_from_empty(self):
        graph = retworkx.PyGraph()
        res = graph.add_edges_from([])
        self.assertEqual([], res)

    def test_add_edge_from_no_data(self):
        graph = retworkx.PyGraph()
        nodes = list(range(4))
        graph.add_nodes_from(nodes)
        edge_list = [(0, 1), (1, 2), (0, 2), (2, 3),
                     (0, 3)]
        res = graph.add_edges_from_no_data(edge_list)
        self.assertEqual(len(res), 5)
        self.assertEqual([None, None, None, None, None], graph.edges())
        self.assertEqual(3, graph.degree(0))
        self.assertEqual(2, graph.degree(1))
        self.assertEqual(3, graph.degree(2))
        self.assertEqual(2, graph.degree(3))

    def test_add_edge_from_empty_no_data(self):
        graph = retworkx.PyGraph()
        res = graph.add_edges_from_no_data([])
        self.assertEqual([], res)

    def test_edge_list(self):
        graph = retworkx.PyGraph()
        graph.add_nodes_from(list(range(4)))
        edge_list = [(0, 1, 'a'), (1, 2, 'b'), (0, 2, 'c'), (2, 3, 'd'),
                     (0, 3, 'e')]
        graph.add_edges_from(edge_list)
        self.assertEqual([(x[0], x[1]) for x in edge_list], graph.edge_list())

    def test_edge_list_empty(self):
        graph = retworkx.PyGraph()
        self.assertEqual([], graph.edge_list())

    def test_weighted_edge_list(self):
        graph = retworkx.PyGraph()
        graph.add_nodes_from(list(range(4)))
        edge_list = [(0, 1, 'a'), (1, 2, 'b'), (0, 2, 'c'), (2, 3, 'd'),
                     (0, 3, 'e')]
        graph.add_edges_from(edge_list)
        self.assertEqual(edge_list, graph.weighted_edge_list())

    def test_weighted_edge_list_empty(self):
        graph = retworkx.PyGraph()
        self.assertEqual([], graph.weighted_edge_list())

    def test_extend_from_edge_list(self):
        graph = retworkx.PyGraph()
        edge_list = [(0, 1), (1, 2), (0, 2), (2, 3),
                     (0, 3)]
        graph.extend_from_edge_list(edge_list)
        self.assertEqual(len(graph), 4)
        self.assertEqual([None] * 5, graph.edges())
        self.assertEqual(3, graph.degree(0))
        self.assertEqual(2, graph.degree(1))
        self.assertEqual(3, graph.degree(2))
        self.assertEqual(2, graph.degree(3))

    def test_extend_from_edge_list_empty(self):
        graph = retworkx.PyGraph()
        graph.extend_from_edge_list([])
        self.assertEqual(0, len(graph))

    def test_extend_from_edge_list_nodes_exist(self):
        graph = retworkx.PyGraph()
        graph.add_nodes_from(list(range(4)))
        edge_list = [(0, 1), (1, 2), (0, 2), (2, 3),
                     (0, 3)]
        graph.extend_from_edge_list(edge_list)
        self.assertEqual(len(graph), 4)
        self.assertEqual([None] * 5, graph.edges())
        self.assertEqual(3, graph.degree(0))
        self.assertEqual(2, graph.degree(1))
        self.assertEqual(3, graph.degree(2))
        self.assertEqual(2, graph.degree(3))

    def test_extend_from_weighted_edge_list(self):
        graph = retworkx.PyGraph()
        edge_list = [(0, 1, 'a'), (1, 2, 'b'), (0, 2, 'c'), (2, 3, 'd'),
                     (0, 3, 'e')]
        graph.extend_from_weighted_edge_list(edge_list)
        self.assertEqual(len(graph), 4)
        self.assertEqual(['a', 'b', 'c', 'd', 'e'], graph.edges())
        self.assertEqual(3, graph.degree(0))
        self.assertEqual(2, graph.degree(1))
        self.assertEqual(3, graph.degree(2))
        self.assertEqual(2, graph.degree(3))

    def test_extend_from_weighted_edge_list_empty(self):
        graph = retworkx.PyGraph()
        graph.extend_from_weighted_edge_list([])
        self.assertEqual(0, len(graph))

    def test_extend_from_weighted_edge_list_nodes_exist(self):
        graph = retworkx.PyGraph()
        graph.add_nodes_from(list(range(4)))
        edge_list = [(0, 1, 'a'), (1, 2, 'b'), (0, 2, 'c'), (2, 3, 'd'),
                     (0, 3, 'e')]
        graph.extend_from_weighted_edge_list(edge_list)
        self.assertEqual(len(graph), 4)
        self.assertEqual(['a', 'b', 'c', 'd', 'e'], graph.edges())
        self.assertEqual(3, graph.degree(0))
        self.assertEqual(2, graph.degree(1))
        self.assertEqual(3, graph.degree(2))
        self.assertEqual(2, graph.degree(3))
