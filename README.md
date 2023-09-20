### A lightning-fast, general purpose module for finding fixed-cost circuits in weighted, undirected graphs.

## Usage

The speedicycle binary (compiled on Linux, but expected to run on most Unix-derived systems) is usable via the command line with the syntax:

```shell
speedicycle -i ['path_to_input_file.txt'] -s ['index of source node'] -t ['target path cost']
```

Currently, the input file must contain the description of an undirected, weighted graph in DIMACS format. Specifically, the file should consist solely of plaintext, consisting of:

- A header line beginning with the character `p` and containing the number of nodes/vertices and edges in the graph, e.g. `p 4000 5000` for a graph containing 4000 nodes and 5000 edges
- One line for each vertex in the graph, beginning with the character `v` and containing a numerical label, e.g. `v 11213`. These vertices will later be referred to _by their position in this list_, indexed starting at zero.
- One line for each edge in the graph, beginning with the character `e` and containing a starting vertex, an ending vertex, and a numerical weight. Vertices here are referred to _by their index position in the above vertex list_. For example, `e 0 1 25`, signifying an edge connecting the 0th and 1st vertex in the list with a weight of 25.

An example of this format is contained in `DIMACS_sample.txt`. **Support for additional formats is in the works, and contributions on that front are welcome!**

## Background

This tool was initially designed for the purpose of locating fixed-distance, closed-circuit walking paths in street grid data (and, by extension, walk routes of a predetermined time). The problem of locating circuits of specified cost, however, is more generally applicable.

The central algorithm here implements Lewis and Corcoran’s [“double-path heuristic”](https://ideas.repec.org/a/spr/joheur/v28y2022i3d10.1007_s10732-022-09493-5.html) to locate viable circuits comprising two edge-disjoint paths from a source node _s_ to a target node _t_, each with a cost of approximately _k_/2, where _k_ is the desired total cost of traversing the circuit. By strategically selecting target nodes (and eliminating non-viable paths after each calculation), a suitable path (if one exists) can be located with impressive efficiency.

## Implementation

Speedicycle is built atop [petgraph](https://github.com/petgraph/petgraph), which provides both the underlying graph structure upon which the double-path routine operates and implementations of essential graph traversal algorithms, including Dijkstra's and Moore's shortest path algorithms, both of which are used here.

Also contained here is an implementation of [Ramesh Bhandari's method](https://worldcat.org/en/title/493194936) for finding two edge-disjoint paths between _s_ and _t_. In brief, once the shortest path between the two nodes is located, directions are added along the path and weights adjusted to heavily penalize re-traversal in the original direction. The new shortest path is located, and then the two are "unwoven" to produce the final circuit.

Finally, an implementation of Hierholzer's algorithm for locating an Eulerian circuit within an undirected graph is used to properly order the vertices in the resulting paths.

## Final Notes

Your feedback, suggestions, and contributions are all welcome! If you spot an error, please open an issue, and feel free (but not compelled) to submit a PR with a fix. If you're just interested in discussing, please do reach out via [LinkedIn](https://www.linkedin.com/in/kyle-slugg/) or on my [personal website](https://kyleslugg.co/). Talk soon!
