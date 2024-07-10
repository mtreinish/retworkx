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


class TestDescendants_at_distance(unittest.TestCase):
    def setup(self):
        self.G = rustworkx.PyDiGraph()
        edges = [(0,1),(0,2),(1,3),(2,3),(3,4)]
        self.G.add_edges_from(edges)

    def test_distance_at_zero(self):
        result = rustworkx.descendants_at_distance(self.G,0,0)
        self.assertEqual(result, {0})
    
    def test_distance_one(self):
        result = rustworkx.descendants_at_distance(self.G,0,1)
        self.assertEqual(result, {1,2})
    
    def test_distance_two(self):
        result = rustworkx.descendants_at_distance(self.G,0,2)
        self.assertEqual(result, {3})

    def test_distance_three(self):
        result = rustworkx.descendants_at_distance(self.G,0,3)
        self.assertEqual(result, {4})

    def test_distance_four(self):
        result = rustworkx.descendants_at_distance(self.G,0,4)
        self.assertEqual(result, {5})