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

import rustworkx


class TestDescendantsAtDistance(unittest.TestCase):
    def test_distance_zero(self):
        graph = rustworkx.PyDiGraph()
        graph.add_nodes_from(list(range(6)))
        graph.add_edges_from_no_data([(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 5)])
        result = rustworkx.descendants_at_distance(graph, 0, 0)
        expected = [0]
        self.assertEqual(result, expected)

    def test_distance_one(self):
        graph = rustworkx.PyDiGraph()
        graph.add_nodes_from(list(range(6)))
        graph.add_edges_from_no_data([(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 5)])
        result = rustworkx.descendants_at_distance(graph, 0, 1)
        expected = [1, 2]
        self.assertEqual(len(result), len(expected))

    def test_distance_two(self):
        graph = rustworkx.PyDiGraph()
        graph.add_nodes_from(list(range(6)))
        graph.add_edges_from_no_data([(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 5)])
        result = rustworkx.descendants_at_distance(graph, 0, 2)
        expected = [3]
        self.assertEqual(result, expected)

    def test_distance_three(self):
        graph = rustworkx.PyDiGraph()
        graph.add_nodes_from(list(range(6)))
        graph.add_edges_from_no_data([(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 5)])
        result = rustworkx.descendants_at_distance(graph, 0, 3)
        expected = [4]
        self.assertEqual(result, expected)

    def test_distance_four(self):
        graph = rustworkx.PyDiGraph()
        graph.add_nodes_from(list(range(6)))
        graph.add_edges_from_no_data([(0, 1), (0, 2), (1, 3), (2, 3), (3, 4), (4, 5)])
        result = rustworkx.descendants_at_distance(graph, 0, 4)
        expected = [5]
        self.assertEqual(result, expected)
