//! `Graph<N, E, Ty, Ix>` is a graph datastructure using an adjacency list representation.

use std::cmp;
use std::fmt;
use std::iter;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut, Range};
use std::slice;

use {
    EdgeDirection, Outgoing, Incoming,
    Undirected,
    Directed,
    EdgeType,
    IntoWeightedEdge,
};

/// The default integer type for node and edge indices in `Graph`.
/// `u32` is the default to reduce the size of the graph's data and improve
/// performance in the common case.
pub type DefIndex = u32;

/// Trait for the unsigned integer type used for node and edge indices.
///
/// Marked `unsafe` because: the trait must faithfully preseve
/// and convert index values.
pub unsafe trait IndexType : Copy + Ord + fmt::Debug + 'static
{
    fn new(x: usize) -> Self;
    fn index(&self) -> usize;
    fn max() -> Self;
}

unsafe impl IndexType for usize {
    #[inline(always)]
    fn new(x: usize) -> Self { x }
    #[inline(always)]
    fn index(&self) -> Self { *self }
    #[inline(always)]
    fn max() -> Self { ::std::usize::MAX }
}

unsafe impl IndexType for u32 {
    #[inline(always)]
    fn new(x: usize) -> Self { x as u32 }
    #[inline(always)]
    fn index(&self) -> usize { *self as usize }
    #[inline(always)]
    fn max() -> Self { ::std::u32::MAX }
}

unsafe impl IndexType for u16 {
    #[inline(always)]
    fn new(x: usize) -> Self { x as u16 }
    #[inline(always)]
    fn index(&self) -> usize { *self as usize }
    #[inline(always)]
    fn max() -> Self { ::std::u16::MAX }
}

unsafe impl IndexType for u8 {
    #[inline(always)]
    fn new(x: usize) -> Self { x as u8 }
    #[inline(always)]
    fn index(&self) -> usize { *self as usize }
    #[inline(always)]
    fn max() -> Self { ::std::u8::MAX }
}

/// Node identifier.
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct NodeIndex<Ix=DefIndex>(Ix);

impl<Ix: IndexType> NodeIndex<Ix>
{
    #[inline]
    pub fn new(x: usize) -> Self {
        NodeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize
    {
        self.0.index()
    }

    #[inline]
    pub fn end() -> Self
    {
        NodeIndex(IndexType::max())
    }

    fn _into_edge(self) -> EdgeIndex<Ix> {
        EdgeIndex(self.0)
    }
}

impl<Ix: IndexType> From<Ix> for NodeIndex<Ix> {
    fn from(ix: Ix) -> Self { NodeIndex(ix) }
}

/// Short version of `NodeIndex::new`
pub fn node_index<Ix: IndexType>(index: usize) -> NodeIndex<Ix> { NodeIndex::new(index) }

/// Short version of `EdgeIndex::new`
pub fn edge_index<Ix: IndexType>(index: usize) -> EdgeIndex<Ix> { EdgeIndex::new(index) }

/// Edge identifier.
#[derive(Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct EdgeIndex<Ix=DefIndex>(Ix);

impl<Ix: IndexType> EdgeIndex<Ix>
{
    #[inline]
    pub fn new(x: usize) -> Self {
        EdgeIndex(IndexType::new(x))
    }

    #[inline]
    pub fn index(self) -> usize
    {
        self.0.index()
    }

    /// An invalid `EdgeIndex` used to denote absence of an edge, for example
    /// to end an adjacency list.
    #[inline]
    pub fn end() -> Self {
        EdgeIndex(IndexType::max())
    }

    fn _into_node(self) -> NodeIndex<Ix> {
        NodeIndex(self.0)
    }
}

impl<Ix: IndexType> fmt::Debug for EdgeIndex<Ix>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "EdgeIndex("));
        if *self == EdgeIndex::end() {
            try!(write!(f, "End"));
        } else {
            try!(write!(f, "{}", self.index()));
        }
        write!(f, ")")
    }
}

const DIRECTIONS: [EdgeDirection; 2] = [Outgoing, Incoming];

/// The graph's node type.
#[derive(Debug, Clone)]
pub struct Node<N, Ix: IndexType = DefIndex> {
    /// Associated node data.
    pub weight: N,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex<Ix>; 2],
}

impl<N, Ix: IndexType> Node<N, Ix>
{
    /// Accessor for data structure internals: the first edge in the given direction.
    pub fn next_edge(&self, dir: EdgeDirection) -> EdgeIndex<Ix>
    {
        self.next[dir as usize]
    }
}

/// The graph's edge type.
#[derive(Debug, Clone)]
pub struct Edge<E, Ix: IndexType = DefIndex> {
    /// Associated edge data.
    pub weight: E,
    /// Next edge in outgoing and incoming edge lists.
    next: [EdgeIndex<Ix>; 2],
    /// Start and End node index
    node: [NodeIndex<Ix>; 2],
}

impl<E, Ix: IndexType> Edge<E, Ix>
{
    /// Accessor for data structure internals: the next edge for the given direction.
    pub fn next_edge(&self, dir: EdgeDirection) -> EdgeIndex<Ix>
    {
        self.next[dir as usize]
    }

    /// Return the source node index.
    pub fn source(&self) -> NodeIndex<Ix>
    {
        self.node[0]
    }

    /// Return the target node index.
    pub fn target(&self) -> NodeIndex<Ix>
    {
        self.node[1]
    }
}

/// `Graph<N, E, Ty, Ix>` is a graph datastructure using an adjacency list representation.
///
/// `Graph` is parameterized over:
///
/// - Associated data `N` for nodes and `E` for edges, called *weights*.
///   The associated data can be of arbitrary type.
/// - Edge type `Ty` that determines whether the graph edges are directed or undirected.
/// - Index type `Ix`, which determines the maximum size of the graph.
///
/// The graph uses **O(|V| + |E|)** space, and allows fast node and edge insert,
/// efficient graph search and graph algorithms.
/// It implements **O(e')** edge lookup and edge and node removals, where **e'**
/// is some local measure of edge count.
/// Based on the graph datastructure used in rustc.
///
/// Here's an example of building a graph with directed edges, and below
/// an illustration of how it could be rendered with graphviz (graphviz rendering
/// is not a part of petgraph):
///
/// ```
/// use petgraph::Graph;
///
/// let mut deps = Graph::<&str, &str>::new();
/// let pg = deps.add_node("petgraph");
/// let fb = deps.add_node("fixedbitset");
/// let qc = deps.add_node("quickcheck");
/// let rand = deps.add_node("rand");
/// let libc = deps.add_node("libc");
/// deps.extend_with_edges(&[
///     (pg, fb), (pg, qc),
///     (qc, rand), (rand, libc), (qc, libc),
/// ]);
/// ```
///
/// ![graph-example](graph-example.svg)
///
/// ### Graph Indices
///
/// The graph maintains indices for nodes and edges, and node and edge
/// weights may be accessed mutably. Indices range in a compact interval, for
/// example for *n* nodes indices are 0 to *n* - 1 inclusive.
///
/// `NodeIndex` and `EdgeIndex` are types that act as references to nodes and edges,
/// but these are only stable across certain operations.
/// **Adding nodes or edges keeps indices stable.
/// Removing nodes or edges may shift other indices.**
/// Removing a node will force the last node to shift its index to
/// take its place. Similarly, removing an edge shifts the index of the last edge.
///
/// The `Ix` parameter is `u32` by default. The goal is that you can ignore this parameter
/// completely unless you need a very big graph -- then you can use `usize`.
///
/// ### Pros and Cons of Indices
///
/// * The fact that the node and edge indices in the graph each are numbered in compact
/// intervals (from 0 to *n* - 1 for *n* nodes) simplifies some graph algorithms.
///
/// * You can select graph index integer type after the size of the graph. A smaller
/// size may have better performance.
///
/// * Using indices allows mutation while traversing the graph, see `Dfs`,
/// and `.neighbors(a).detach()`.
///
/// * You can create several graphs using the equal node indices but with
/// differing weights or differing edges.
///
/// * The `Graph` is a regular rust collection and is `Send` and `Sync` (as long
/// as associated data `N` and `E` are).
///
/// * Some indices shift during node or edge removal, so that is a drawback
/// of removing elements. Indices don't allow as much compile time checking as
/// references.
///
pub struct Graph<N, E, Ty = Directed, Ix: IndexType = DefIndex> {
    nodes: Vec<Node<N, Ix>>,
    edges: Vec<Edge<E, Ix>>,
    ty: PhantomData<Ty>,
}

/// The resulting cloned graph has the same graph indices as `self`.
impl<N, E, Ty, Ix: IndexType> Clone for Graph<N, E, Ty, Ix>
    where N: Clone, E: Clone,
{
    fn clone(&self) -> Self {
        Graph {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            ty: self.ty,
        }
    }

    fn clone_from(&mut self, rhs: &Self) {
        self.nodes.clone_from(&rhs.nodes);
        self.edges.clone_from(&rhs.edges);
        self.ty = rhs.ty;
    }
}

impl<N, E, Ty, Ix> fmt::Debug for Graph<N, E, Ty, Ix> where
    N: fmt::Debug, E: fmt::Debug, Ty: EdgeType, Ix: IndexType
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let indent = "    ";
        let etype = if self.is_directed() { "Directed" } else { "Undirected" };
        try!(write!(f, "Graph<{}> {{", etype));
        if self.node_count() == 0 {
            return write!(f, "}}");
        }
        try!(writeln!(f, ""));
        for (index, n) in self.nodes.iter().enumerate() {
            try!(writeln!(f, "{}{}: Node({:?}),", indent, index, n.weight));
        }
        for (index, e) in self.edges.iter().enumerate() {
            try!(writeln!(f, "{}{}: Edge(from={}, to={}, weight={:?}),",
                          indent, index,
                          e.source().index(),
                          e.target().index(),
                          e.weight));
        }
        try!(write!(f, "}}"));
        Ok(())
    }
}

enum Pair<T> {
    Both(T, T),
    One(T),
    None,
}

fn index_twice<T>(slc: &mut [T], a: usize, b: usize) -> Pair<&mut T>
{
    if a == b {
        slc.get_mut(a).map_or(Pair::None, Pair::One)
    } else if a >= slc.len() || b >= slc.len() {
        Pair::None
    } else {
        // safe because a, b are in bounds and distinct
        unsafe {
            let ar = &mut *(slc.get_unchecked_mut(a) as *mut _);
            let br = &mut *(slc.get_unchecked_mut(b) as *mut _);
            Pair::Both(ar, br)
        }
    }
}

impl<N, E> Graph<N, E, Directed>
{
    /// Create a new `Graph` with directed edges.
    ///
    /// This is a convenience method. Use `Graph::with_capacity` or `Graph::default` for
    /// a constructor that is generic in all the type parameters of `Graph`.
    pub fn new() -> Self
    {
        Graph{nodes: Vec::new(), edges: Vec::new(),
              ty: PhantomData}
    }
}

impl<N, E> Graph<N, E, Undirected>
{
    /// Create a new `Graph` with undirected edges.
    ///
    /// This is a convenience method. Use `Graph::with_capacity` or `Graph::default` for
    /// a constructor that is generic in all the type parameters of `Graph`.
    pub fn new_undirected() -> Self
    {
        Graph{nodes: Vec::new(), edges: Vec::new(),
              ty: PhantomData}
    }
}

impl<N, E, Ty, Ix> Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    /// Create a new `Graph` with estimated capacity.
    pub fn with_capacity(nodes: usize, edges: usize) -> Self
    {
        Graph{nodes: Vec::with_capacity(nodes), edges: Vec::with_capacity(edges),
              ty: PhantomData}
    }

    /// Return the number of nodes (vertices) in the graph.
    ///
    /// Computes in **O(1)** time.
    pub fn node_count(&self) -> usize
    {
        self.nodes.len()
    }

    /// Return the number of edges in the graph.
    ///
    /// Computes in **O(1)** time.
    pub fn edge_count(&self) -> usize
    {
        self.edges.len()
    }

    /// Whether the graph has directed edges or not.
    #[inline]
    pub fn is_directed(&self) -> bool
    {
        Ty::is_directed()
    }

    /// Add a node (also called vertex) with associated data `weight` to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Return the index of the new node.
    ///
    /// **Panics** if the Graph is at the maximum number of nodes for its index
    /// type (N/A if usize).
    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix>
    {
        let node = Node{weight: weight, next: [EdgeIndex::end(), EdgeIndex::end()]};
        let node_idx = NodeIndex::new(self.nodes.len());
        // check for max capacity, except if we use usize
        assert!(Ix::max().index() == !0 || NodeIndex::end() != node_idx);
        self.nodes.push(node);
        node_idx
    }

    /// Access the weight for node `a`.
    ///
    /// Also available with indexing syntax: `&graph[a]`.
    pub fn node_weight(&self, a: NodeIndex<Ix>) -> Option<&N>
    {
        self.nodes.get(a.index()).map(|n| &n.weight)
    }

    /// Access the weight for node `a`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[a]`.
    pub fn node_weight_mut(&mut self, a: NodeIndex<Ix>) -> Option<&mut N>
    {
        self.nodes.get_mut(a.index()).map(|n| &mut n.weight)
    }

    /// Add an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(1)** time.
    ///
    /// **Panics** if any of the nodes don't exist.<br>
    /// **Panics** if the Graph is at the maximum number of edges for its index
    /// type (N/A if usize).
    ///
    /// **Note:** `Graph` allows adding parallel (“duplicate”) edges. If you want
    /// to avoid this, use [`.update_edge(a, b, weight)`](#method.update_edge) instead.
    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix>
    {
        let edge_idx = EdgeIndex::new(self.edges.len());
        assert!(Ix::max().index() == !0 || EdgeIndex::end() != edge_idx);
        let mut edge = Edge {
            weight: weight,
            node: [a, b],
            next: [EdgeIndex::end(); 2],
        };
        match index_twice(&mut self.nodes, a.index(), b.index()) {
            Pair::None => panic!("Graph::add_edge: node indices out of bounds"),
            Pair::One(an) => {
                edge.next = an.next;
                an.next[0] = edge_idx;
                an.next[1] = edge_idx;
            }
            Pair::Both(an, bn) => {
                // a and b are different indices
                edge.next = [an.next[0], bn.next[1]];
                an.next[0] = edge_idx;
                bn.next[1] = edge_idx;
            }
        }
        self.edges.push(edge);
        edge_idx
    }

    /// Add or update an edge from `a` to `b`.
    /// If the edge already exists, its weight is updated.
    ///
    /// Return the index of the affected edge.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// connected to `a` (and `b`, if the graph edges are undirected).
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix>
    {
        if let Some(ix) = self.find_edge(a, b) {
            if let Some(ed) = self.edge_weight_mut(ix) {
                *ed = weight;
                return ix;
            }
        }
        self.add_edge(a, b, weight)
    }

    /// Access the weight for edge `e`.
    ///
    /// Also available with indexing syntax: `&graph[e]`.
    pub fn edge_weight(&self, e: EdgeIndex<Ix>) -> Option<&E>
    {
        self.edges.get(e.index()).map(|ed| &ed.weight)
    }

    /// Access the weight for edge `e`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[e]`.
    pub fn edge_weight_mut(&mut self, e: EdgeIndex<Ix>) -> Option<&mut E>
    {
        self.edges.get_mut(e.index()).map(|ed| &mut ed.weight)
    }

    /// Access the source and target nodes for `e`.
    pub fn edge_endpoints(&self, e: EdgeIndex<Ix>)
        -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)>
    {
        self.edges.get(e.index()).map(|ed| (ed.source(), ed.target()))
    }

    /// Remove `a` from the graph if it exists, and return its weight.
    /// If it doesn't exist in the graph, return `None`.
    ///
    /// Apart from `a`, this invalidates the last node index in the graph
    /// (that node will adopt the removed node index). Edge indices are
    /// invalidated as they would be following the removal of each edge
    /// with an endpoint in `a`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of affected
    /// edges, including *n* calls to `.remove_edge()` where *n* is the number
    /// of edges with an endpoint in `a`, and including the edges with an
    /// endpoint in the displaced node.
    pub fn remove_node(&mut self, a: NodeIndex<Ix>) -> Option<N>
    {
        if self.nodes.get(a.index()).is_none() {
            return None
        }
        for d in &DIRECTIONS {
            let k = *d as usize;

            // Remove all edges from and to this node.
            loop {
                let next = self.nodes[a.index()].next[k];
                if next == EdgeIndex::end() {
                    break
                }
                let ret = self.remove_edge(next);
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }

        // Use swap_remove -- only the swapped-in node is going to change
        // NodeIndex<Ix>, so we only have to walk its edges and update them.

        let node = self.nodes.swap_remove(a.index());

        // Find the edge lists of the node that had to relocate.
        // It may be that no node had to relocate, then we are done already.
        let swap_edges = match self.nodes.get(a.index()) {
            None => return Some(node.weight),
            Some(ed) => ed.next,
        };

        // The swapped element's old index
        let old_index = NodeIndex::new(self.nodes.len());
        let new_index = a;

        // Adjust the starts of the out edges, and ends of the in edges.
        for &d in &DIRECTIONS {
            let k = d as usize;
            for (_, curedge) in EdgesMut::new(&mut self.edges, swap_edges[k], d) {
                debug_assert!(curedge.node[k] == old_index);
                curedge.node[k] = new_index;
            }
        }
        Some(node.weight)
    }

    /// For edge `e` with endpoints `edge_node`, replace links to it,
    /// with links to `edge_next`.
    fn change_edge_links(&mut self, edge_node: [NodeIndex<Ix>; 2], e: EdgeIndex<Ix>,
                         edge_next: [EdgeIndex<Ix>; 2])
    {
        for &d in &DIRECTIONS {
            let k = d as usize;
            let node = match self.nodes.get_mut(edge_node[k].index()) {
                Some(r) => r,
                None => {
                    debug_assert!(false, "Edge's endpoint dir={:?} index={:?} not found",
                                  d, edge_node[k]);
                    return
                }
            };
            let fst = node.next[k];
            if fst == e {
                //println!("Updating first edge 0 for node {}, set to {}", edge_node[0], edge_next[0]);
                node.next[k] = edge_next[k];
            } else {
                for (_i, curedge) in EdgesMut::new(&mut self.edges, fst, d) {
                    if curedge.next[k] == e {
                        curedge.next[k] = edge_next[k];
                        break; // the edge can only be present once in the list.
                    }
                }
            }
        }
    }

    /// Remove an edge and return its edge weight, or `None` if it didn't exist.
    ///
    /// Apart from `e`, this invalidates the last edge index in the graph
    /// (that edge will adopt the removed edge index).
    ///
    /// Computes in **O(e')** time, where **e'** is the size of four particular edge lists, for
    /// the vertices of `e` and the vertices of another affected edge.
    pub fn remove_edge(&mut self, e: EdgeIndex<Ix>) -> Option<E>
    {
        // every edge is part of two lists,
        // outgoing and incoming edges.
        // Remove it from both
        let (edge_node, edge_next) = match self.edges.get(e.index()) {
            None => return None,
            Some(x) => (x.node, x.next),
        };
        // Remove the edge from its in and out lists by replacing it with
        // a link to the next in the list.
        self.change_edge_links(edge_node, e, edge_next);
        self.remove_edge_adjust_indices(e)
    }

    fn remove_edge_adjust_indices(&mut self, e: EdgeIndex<Ix>) -> Option<E>
    {
        // swap_remove the edge -- only the removed edge
        // and the edge swapped into place are affected and need updating
        // indices.
        let edge = self.edges.swap_remove(e.index());
        let swap = match self.edges.get(e.index()) {
            // no elment needed to be swapped.
            None => return Some(edge.weight),
            Some(ed) => ed.node,
        };
        let swapped_e = EdgeIndex::new(self.edges.len());

        // Update the edge lists by replacing links to the old index by references to the new
        // edge index.
        self.change_edge_links(swap, swapped_e, [e, e]);
        Some(edge.weight)
    }

    /// Return an iterator of all nodes with an edge starting from `a`.
    ///
    /// - `Undirected`: All edges from or to `a`.
    /// - `Directed`: Outgoing edges from `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `NodeIndex<Ix>`.
    ///
    /// Use [`.neighbors(a).detach()`][1] to get a neighbor walker that does
    /// not borrow from the graph.
    ///
    /// [1]: struct.Neighbors.html#method.detach
    pub fn neighbors(&self, a: NodeIndex<Ix>) -> Neighbors<E, Ix>
    {
        self.neighbors_directed(a, Outgoing)
    }

    /// Return an iterator of all neighbors that have an edge between them and `a`,
    /// in the specified direction.
    /// If the graph's edges are undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// - `Undirected`: All edges from or to `a`.
    /// - `Directed`, `Outgoing`: All edges from `a`.
    /// - `Directed`, `Incoming`: All edges to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `NodeIndex<Ix>`.
    ///
    /// For a `Directed` graph, neighbors are listed in reverse order of their
    /// addition to the graph, so the most recently added edge's neighbor is
    /// listed first. The order in an `Undirected` graph is arbitrary.
    ///
    /// Use [`.neighbors_directed(a, dir).detach()`][1] to get a neighbor walker that does
    /// not borrow from the graph.
    ///
    /// [1]: struct.Neighbors.html#method.detach
    pub fn neighbors_directed(&self, a: NodeIndex<Ix>, dir: EdgeDirection) -> Neighbors<E, Ix>
    {
        Neighbors {
            iter: self.edges_directed(a, dir),
        }
    }

    /// Return an iterator of all neighbors that have an edge between them and `a`,
    /// in either direction.
    /// If the graph's edges are undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// - `Undirected` and `Directed`: All edges from or to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `NodeIndex<Ix>`.
    ///
    /// Use [`.neighbors_undirected(a).detach()`][1] to get a neighbor walker that does
    /// not borrow from the graph.
    ///
    /// [1]: struct.Neighbors.html#method.detach
    ///
    pub fn neighbors_undirected(&self, a: NodeIndex<Ix>) -> Neighbors<E, Ix>
    {
        Neighbors {
            iter: self.edges_undirected(a),
        }
    }

    /// Return an iterator over the neighbors of node `a`, paired with their respective edge
    /// weights.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `(NodeIndex<Ix>, &E)`.
    pub fn edges(&self, a: NodeIndex<Ix>) -> Edges<E, Ix>
    {
        self.edges_directed(a, EdgeDirection::Outgoing)
    }

    /// Return an iterator of all neighbors that have an edge between them and `a`,
    /// in the specified direction, paired with the respective edge weights.
    ///
    /// If the graph's edges are undirected, this is equivalent to *.edges(a)*.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `(NodeIndex<Ix>, &E)`.
    pub fn edges_directed(&self, a: NodeIndex<Ix>, dir: EdgeDirection) -> Edges<E, Ix>
    {
        let mut iter = self.edges_undirected(a);
        if self.is_directed() {
            let k = dir as usize;
            iter.next[1 - k] = EdgeIndex::end();
            iter.skip_start = NodeIndex::end();
        }
        iter
    }

    /// Return an iterator over the edges from `a` to its neighbors, then *to* `a` from its
    /// neighbors.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `(NodeIndex<Ix>, &E)`.
    fn edges_undirected(&self, a: NodeIndex<Ix>) -> Edges<E, Ix>
    {
        Edges {
            skip_start: a,
            edges: &self.edges,
            next: match self.nodes.get(a.index()) {
                None => [EdgeIndex::end(), EdgeIndex::end()],
                Some(n) => n.next,
            }
        }
    }

    /// Lookup an edge from `a` to `b`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of edges
    /// connected to `a` (and `b`, if the graph edges are undirected).
    pub fn find_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<EdgeIndex<Ix>>
    {
        if !self.is_directed() {
            self.find_edge_undirected(a, b).map(|(ix, _)| ix)
        } else {
            match self.nodes.get(a.index()) {
                None => None,
                Some(node) => {
                    let mut edix = node.next[0];
                    while let Some(edge) = self.edges.get(edix.index()) {
                        if edge.node[1] == b {
                            return Some(edix)
                        }
                        edix = edge.next[0];
                    }
                    None
                }
            }
        }
    }

    /// Lookup an edge between `a` and `b`, in either direction.
    ///
    /// If the graph is undirected, then this is equivalent to `.find_edge()`.
    ///
    /// Return the edge index and its directionality, with `Outgoing` meaning
    /// from `a` to `b` and `Incoming` the reverse,
    /// or `None` if the edge does not exist.
    pub fn find_edge_undirected(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<(EdgeIndex<Ix>, EdgeDirection)>
    {
        match self.nodes.get(a.index()) {
            None => None,
            Some(node) => {
                for &d in &DIRECTIONS {
                    let k = d as usize;
                    let mut edix = node.next[k];
                    while let Some(edge) = self.edges.get(edix.index()) {
                        if edge.node[1 - k] == b {
                            return Some((edix, d))
                        }
                        edix = edge.next[k];
                    }
                }
                None
            }
        }
    }

    /// Return an iterator over either the nodes without edges to them
    /// (`Incoming`) or from them (`Outgoing`).
    ///
    /// An *internal* node has both incoming and outgoing edges.
    /// The nodes in `.externals(Incoming)` are the source nodes and
    /// `.externals(Outgoing)` are the sinks of the graph.
    ///
    /// For a graph with undirected edges, both the sinks and the sources are
    /// just the nodes without edges.
    ///
    /// The whole iteration computes in **O(|V|)** time.
    pub fn externals(&self, dir: EdgeDirection) -> Externals<N, Ty, Ix>
    {
        Externals{iter: self.nodes.iter().enumerate(), dir: dir, ty: PhantomData}
    }

    /// Return an iterator over the node indices of the graph
    pub fn node_indices(&self) -> NodeIndices<Ix> {
        NodeIndices { r: 0..self.node_count(), ty: PhantomData }
    }

    /// Return an iterator yielding mutable access to all node weights.
    ///
    /// The order in which weights are yielded matches the order of their
    /// node indices.
    pub fn node_weights_mut(&mut self) -> NodeWeightsMut<N, Ix>
    {
        NodeWeightsMut { nodes: self.nodes.iter_mut() }
    }

    /// Return an iterator over the edge indices of the graph
    pub fn edge_indices(&self) -> EdgeIndices<Ix> {
        EdgeIndices { r: 0..self.edge_count(), ty: PhantomData }
    }

    /// Return an iterator yielding mutable access to all edge weights.
    ///
    /// The order in which weights are yielded matches the order of their
    /// edge indices.
    pub fn edge_weights_mut(&mut self) -> EdgeWeightsMut<E, Ix>
    {
        EdgeWeightsMut { edges: self.edges.iter_mut() }
    }

    // Remaining methods are of the more internal flavour, read-only access to
    // the data structure's internals.

    /// Access the internal node array.
    pub fn raw_nodes(&self) -> &[Node<N, Ix>]
    {
        &self.nodes
    }

    /// Access the internal edge array.
    pub fn raw_edges(&self) -> &[Edge<E, Ix>]
    {
        &self.edges
    }

    /// Convert the graph into a vector of Nodes and a vector of Edges
    pub fn into_nodes_edges(self) -> (Vec<Node<N, Ix>>, Vec<Edge<E, Ix>>) {
        (self.nodes, self.edges)
    }

    /// Accessor for data structure internals: the first edge in the given direction.
    pub fn first_edge(&self, a: NodeIndex<Ix>, dir: EdgeDirection) -> Option<EdgeIndex<Ix>>
    {
        match self.nodes.get(a.index()) {
            None => None,
            Some(node) => {
                let edix = node.next[dir as usize];
                if edix == EdgeIndex::end() {
                    None
                } else { Some(edix) }
            }
        }
    }

    /// Accessor for data structure internals: the next edge for the given direction.
    pub fn next_edge(&self, e: EdgeIndex<Ix>, dir: EdgeDirection) -> Option<EdgeIndex<Ix>>
    {
        match self.edges.get(e.index()) {
            None => None,
            Some(node) => {
                let edix = node.next[dir as usize];
                if edix == EdgeIndex::end() {
                    None
                } else { Some(edix) }
            }
        }
    }

    /// **Deprecated:** Use [`.neighbors_directed(a, dir).detach()`][1] instead.
    ///
    /// [1]: struct.Graph.html#method.neighbors_directed
    ///
    /// Return a “walker” object that can be used to step through the directed
    /// edges of the node `a` in direction `dir`.
    ///
    /// Note: The walker does not borrow from the graph, this is to allow mixing
    /// edge walking with mutating the graph's weights.
    ///
    /// - `Directed`, `Outgoing`: All edges from `a`.
    /// - `Directed`, `Incoming`: All edges to `a`.
    pub fn walk_edges_directed(&self, a: NodeIndex<Ix>, dir: EdgeDirection) -> WalkEdges<Ix>
    {
        let first_edge = self.first_edge(a, dir).unwrap_or(EdgeIndex::end());
        WalkEdges { next: first_edge, direction: dir }
    }

    /// Index the `Graph` by two indices, any combination of
    /// node or edge indices is fine.
    ///
    /// **Panics** if the indices are equal or if they are out of bounds.
    ///
    /// ```
    /// use petgraph::{Graph, Dfs, Incoming};
    ///
    /// let mut gr = Graph::new();
    /// let a = gr.add_node(0.);
    /// let b = gr.add_node(0.);
    /// let c = gr.add_node(0.);
    /// gr.add_edge(a, b, 3.);
    /// gr.add_edge(b, c, 2.);
    /// gr.add_edge(c, b, 1.);
    ///
    /// // walk the graph and sum incoming edges into the node weight
    /// let mut dfs = Dfs::new(&gr, a);
    /// while let Some(node) = dfs.next(&gr) {
    ///     // use a walker -- a detached neighbors iterator
    ///     let mut edges = gr.neighbors_directed(node, Incoming).detach();
    ///     while let Some(edge) = edges.next_edge(&gr) {
    ///         let (nw, ew) = gr.index_twice_mut(node, edge);
    ///         *nw += *ew;
    ///     }
    /// }
    ///
    /// // check the result
    /// assert_eq!(gr[a], 0.);
    /// assert_eq!(gr[b], 4.);
    /// assert_eq!(gr[c], 2.);
    /// ```
    pub fn index_twice_mut<T, U>(&mut self, i: T, j: U)
        -> (&mut <Self as Index<T>>::Output,
            &mut <Self as Index<U>>::Output)
        where Self: IndexMut<T> + IndexMut<U>,
              T: GraphIndex,
              U: GraphIndex,
    {
        assert!(T::is_node_index() != U::is_node_index() ||
                i.index() != j.index());

        // Allow two mutable indexes here -- they are nonoverlapping
        unsafe {
            let self_mut = self as *mut _;
            (<Self as IndexMut<T>>::index_mut(&mut *self_mut, i),
             <Self as IndexMut<U>>::index_mut(&mut *self_mut, j))
        }
    }

    /// Reverse the direction of all edges
    pub fn reverse(&mut self) {
        // swap edge endpoints,
        // edge incoming / outgoing lists,
        // node incoming / outgoing lists
        for edge in &mut self.edges {
            edge.node.swap(0, 1);
            edge.next.swap(0, 1);
        }
        for node in &mut self.nodes {
            node.next.swap(0, 1);
        }
    }

    /// Remove all nodes and edges
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }

    /// Remove all edges
    pub fn clear_edges(&mut self) {
        self.edges.clear();
        for node in &mut self.nodes {
            node.next = [EdgeIndex::end(), EdgeIndex::end()];
        }
    }

    /// Return the current node and edge capacity of the graph.
    pub fn capacity(&self) -> (usize, usize) {
        (self.nodes.capacity(), self.edges.capacity())
    }

    /// Reserves capacity for at least `additional` more nodes to be inserted in
    /// the graph. Graph may reserve more space to avoid frequent reallocations.
    ///
    /// **Panics** if the new capacity overflows `usize`.
    pub fn reserve_nodes(&mut self, additional: usize) {
        self.nodes.reserve(additional);
    }

    /// Reserves capacity for at least `additional` more edges to be inserted in
    /// the graph. Graph may reserve more space to avoid frequent reallocations.
    ///
    /// **Panics** if the new capacity overflows `usize`.
    pub fn reserve_edges(&mut self, additional: usize) {
        self.edges.reserve(additional);
    }

    /// Reserves the minimum capacity for exactly `additional` more nodes to be
    /// inserted in the graph. Does nothing if the capacity is already
    /// sufficient.
    ///
    /// Prefer `reserve_nodes` if future insertions are expected.
    ///
    /// **Panics** if the new capacity overflows `usize`.
    pub fn reserve_exact_nodes(&mut self, additional: usize) {
        self.nodes.reserve_exact(additional);
    }

    /// Reserves the minimum capacity for exactly `additional` more edges to be
    /// inserted in the graph.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Prefer `reserve_edges` if future insertions are expected.
    ///
    /// **Panics** if the new capacity overflows `usize`.
    pub fn reserve_exact_edges(&mut self, additional: usize) {
        self.edges.reserve_exact(additional);
    }

    /// Shrinks the capacity of the underlying nodes collection as much as possible.
    pub fn shrink_to_fit_nodes(&mut self) {
        self.nodes.shrink_to_fit();
    }

    /// Shrinks the capacity of the underlying edges collection as much as possible.
    pub fn shrink_to_fit_edges(&mut self) {
        self.edges.shrink_to_fit();
    }

    /// Shrinks the capacity of the graph as much as possible.
    pub fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
        self.edges.shrink_to_fit();
    }

    /// Keep all nodes that return `true` from the `visit` closure,
    /// remove the others.
    ///
    /// `visit` is provided a mutable reference to the graph, so that
    /// the graph can be walked and associated data modified. You should
    /// not add or remove nodes.
    ///
    /// The order nodes are visited is not specified.
    pub fn retain_nodes<F>(&mut self, mut visit: F)
        where F: FnMut(&mut Self, NodeIndex<Ix>) -> bool
    {
        for index in self.node_indices().rev() {
            if !visit(self, index) {
                let ret = self.remove_node(index);
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }
    }

    /// Keep all edges that return `true` from the `visit` closure,
    /// remove the others.
    ///
    /// `visit` is provided a mutable reference to the graph, so that
    /// the graph can be walked and associated data modified. You should
    /// not add or remove nodes or edges.
    ///
    /// The order edges are visited is not specified.
    pub fn retain_edges<F>(&mut self, mut visit: F)
        where F: FnMut(&mut Self, EdgeIndex<Ix>) -> bool
    {
        for index in self.edge_indices().rev() {
            if !visit(self, index) {
                let ret = self.remove_edge(index);
                debug_assert!(ret.is_some());
                let _ = ret;
            }
        }
    }


    /// Create a new `Graph` from an iterable of edges.
    ///
    /// Node weights `N` are set to default values.
    /// Edge weights `E` may either be specified in the list,
    /// or they are filled with default values.
    ///
    /// Nodes are inserted automatically to match the edges.
    ///
    /// ```
    /// use petgraph::Graph;
    ///
    /// let gr = Graph::<(), i32>::from_edges(&[
    ///     (0, 1), (0, 2), (0, 3),
    ///     (1, 2), (1, 3),
    ///     (2, 3),
    /// ]);
    /// ```
    pub fn from_edges<I>(iterable: I) -> Self
        where I: IntoIterator,
              I::Item: IntoWeightedEdge<E>,
              <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex<Ix>>,
              N: Default,
    {
        let mut g = Self::with_capacity(0, 0);
        g.extend_with_edges(iterable);
        g
    }

    /// Extend the graph from an iterable of edges.
    ///
    /// Node weights `N` are set to default values.
    /// Edge weights `E` may either be specified in the list,
    /// or they are filled with default values.
    ///
    /// Nodes are inserted automatically to match the edges.
    pub fn extend_with_edges<I>(&mut self, iterable: I)
        where I: IntoIterator,
              I::Item: IntoWeightedEdge<E>,
              <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex<Ix>>,
              N: Default,
    {
        let iter = iterable.into_iter();
        let (low, _) = iter.size_hint();
        self.edges.reserve(low);

        for elt in iter {
            let (source, target, weight) = elt.into_weighted_edge();
            let (source, target) = (source.into(), target.into());
            let nx = cmp::max(source, target);
            while nx.index() >= self.node_count() {
                self.add_node(N::default());
            }
            self.add_edge(source, target, weight);
        }
    }


    /// Create a new `Graph` by mapping node and
    /// edge weights to new values.
    ///
    /// The resulting graph has the same structure and the same
    /// graph indices as `self`.
    pub fn map<'a, F, G, N2, E2>(&'a self, mut node_map: F, mut edge_map: G)
        -> Graph<N2, E2, Ty, Ix>
        where F: FnMut(NodeIndex<Ix>, &'a N) -> N2,
              G: FnMut(EdgeIndex<Ix>, &'a E) -> E2,
    {
        let mut g = Graph::with_capacity(self.node_count(), self.edge_count());
        for (i, node) in enumerate(&self.nodes) {
            g.nodes.push(Node {
                weight: node_map(NodeIndex::new(i), &node.weight),
                next: node.next,
            });
        }
        for (i, edge) in enumerate(&self.edges) {
            g.edges.push(Edge {
                weight: edge_map(EdgeIndex::new(i), &edge.weight),
                next: edge.next,
                node: edge.node,
            });
        }
        g
    }

    /// Create a new `Graph` by mapping nodes and edges.
    /// A node or edge may be mapped to `None` to exclude it from
    /// the resulting graph.
    ///
    /// Nodes are mapped first with the `node_map` closure, then
    /// `edge_map` is called for the edges that have not had any endpoint
    /// removed.
    ///
    /// The resulting graph has the structure of a subgraph of the original graph.
    /// If no nodes are removed, the resulting graph has compatible node
    /// indices; if neither nodes nor edges are removed, the result has
    /// the same graph indices as `self`.
    pub fn filter_map<'a, F, G, N2, E2>(&'a self, mut node_map: F, mut edge_map: G)
        -> Graph<N2, E2, Ty, Ix>
        where F: FnMut(NodeIndex<Ix>, &'a N) -> Option<N2>,
              G: FnMut(EdgeIndex<Ix>, &'a E) -> Option<E2>,
    {
        let mut g = Graph::with_capacity(0, 0);
        // mapping from old node index to new node index, end represents removed.
        let mut node_index_map = vec![NodeIndex::end(); self.node_count()];
        for (i, node) in enumerate(&self.nodes) {
            if let Some(nw) = node_map(NodeIndex::new(i), &node.weight) {
                node_index_map[i] = g.add_node(nw);
            }
        }
        for (i, edge) in enumerate(&self.edges) {
            // skip edge if any endpoint was removed
            let source = node_index_map[edge.source().index()];
            let target = node_index_map[edge.target().index()];
            if source != NodeIndex::end() && target != NodeIndex::end() {
                if let Some(ew) = edge_map(EdgeIndex::new(i), &edge.weight) {
                    g.add_edge(source, target, ew);
                }
            }
        }
        g
    }

    /// Convert the graph into either undirected or directed. No edge adjustments
    /// are done, so you may want to go over the result to remove or add edges.
    ///
    /// Computes in **O(1)** time.
    pub fn into_edge_type<NewTy>(self) -> Graph<N, E, NewTy, Ix> where
        NewTy: EdgeType
    {
        Graph{nodes: self.nodes, edges: self.edges,
              ty: PhantomData}
    }
}

/// An iterator over either the nodes without edges to them or from them.
pub struct Externals<'a, N: 'a, Ty, Ix: IndexType = DefIndex> {
    iter: iter::Enumerate<slice::Iter<'a, Node<N, Ix>>>,
    dir: EdgeDirection,
    ty: PhantomData<Ty>,
}

impl<'a, N: 'a, Ty, Ix> Iterator for Externals<'a, N, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;
    fn next(&mut self) -> Option<NodeIndex<Ix>>
    {
        let k = self.dir as usize;
        loop {
            match self.iter.next() {
                None => return None,
                Some((index, node)) => {
                    if node.next[k] == EdgeIndex::end() &&
                        (Ty::is_directed() ||
                         node.next[1-k] == EdgeIndex::end()) {
                        return Some(NodeIndex::new(index))
                    } else {
                        continue
                    }
                },
            }
        }
    }
}

/// Iterator over the neighbors of a node.
///
/// Iterator element type is `NodeIndex<Ix>`.
///
/// Created with [`.neighbors()`][1], [`.neighbors_directed()`][2] or
/// [`.neighbors_undirected()`][3].
///
/// [1]: struct.Graph.html#method.neighbors
/// [2]: struct.Graph.html#method.neighbors_directed
/// [3]: struct.Graph.html#method.neighbors_undirected
pub struct Neighbors<'a, E: 'a, Ix: 'a = DefIndex> where
    Ix: IndexType,
{
    iter: Edges<'a, E, Ix>,
}

impl<'a, E, Ix> Iterator for Neighbors<'a, E, Ix> where
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<NodeIndex<Ix>> {
        self.iter.next().map(|(index, _)| index)
    }
}

impl<'a, E, Ix> Clone for Neighbors<'a, E, Ix>
    where Ix: IndexType,
{
    fn clone(&self) -> Self {
        Neighbors { iter: self.iter.clone() }
    }
}

impl<'a, E, Ix> Neighbors<'a, E, Ix>
    where Ix: IndexType,
{
    /// Return a “walker” object that can be used to step through the
    /// neighbors and edges from the origin node.
    ///
    /// Note: The walker does not borrow from the graph, this is to allow mixing
    /// edge walking with mutating the graph's weights.
    pub fn detach(&self) -> WalkNeighbors<Ix> {
        WalkNeighbors {
            skip_start: self.iter.skip_start,
            next: self.iter.next
        }
    }
}

struct EdgesMut<'a, E: 'a, Ix: IndexType = DefIndex> {
    edges: &'a mut [Edge<E, Ix>],
    next: EdgeIndex<Ix>,
    dir: EdgeDirection,
}

impl<'a, E, Ix> EdgesMut<'a, E, Ix> where
    Ix: IndexType,
{
    fn new(edges: &'a mut [Edge<E, Ix>], next: EdgeIndex<Ix>, dir: EdgeDirection) -> Self
    {
        EdgesMut{
            edges: edges,
            next: next,
            dir: dir
        }
    }
}

impl<'a, E, Ix> Iterator for EdgesMut<'a, E, Ix> where
    Ix: IndexType,
{
    type Item = (EdgeIndex<Ix>, &'a mut Edge<E, Ix>);
    fn next(&mut self) -> Option<(EdgeIndex<Ix>, &'a mut Edge<E, Ix>)>
    {
        let this_index = self.next;
        let k = self.dir as usize;
        match self.edges.get_mut(self.next.index()) {
            None => None,
            Some(edge) => {
                self.next = edge.next[k];
                // We cannot in safe rust, derive a &'a mut from &mut self,
                // when the life of &mut self is shorter than 'a.
                //
                // We guarantee that this will not allow two pointers to the same
                // edge, and use unsafe to extend the life.
                //
                // See http://stackoverflow.com/a/25748645/3616050
                let long_life_edge = unsafe {
                    &mut *(edge as *mut _)
                };
                Some((this_index, long_life_edge))
            }
        }
    }
}

/// Iterator over the edges of a node.
pub struct Edges<'a, E: 'a, Ix: IndexType = DefIndex> {
    /// starting node to skip over
    skip_start: NodeIndex<Ix>,
    edges: &'a [Edge<E, Ix>],
    next: [EdgeIndex<Ix>; 2],
}

impl<'a, E, Ix> Iterator for Edges<'a, E, Ix> where
    Ix: IndexType,
{
    type Item = (NodeIndex<Ix>, &'a E);
    fn next(&mut self) -> Option<(NodeIndex<Ix>, &'a E)>
    {
        // First any outgoing edges
        match self.edges.get(self.next[0].index()) {
            None => {}
            Some(edge) => {
                self.next[0] = edge.next[0];
                return Some((edge.node[1], &edge.weight))
            }
        }
        // Then incoming edges
        // For an "undirected" iterator (traverse both incoming
        // and outgoing edge lists), make sure we don't double
        // count selfloops by skipping them in the incoming list.
        while let Some(edge) = self.edges.get(self.next[1].index()) {
            self.next[1] = edge.next[1];
            if edge.node[0] != self.skip_start {
                return Some((edge.node[0], &edge.weight));
            }
        }
        None
    }
}

impl<'a, E, Ix> Clone for Edges<'a, E, Ix>
    where Ix: IndexType
{
    fn clone(&self) -> Self {
        Edges {
            skip_start: self.skip_start,
            edges: self.edges,
            next: self.next,
        }
    }
}

/// Iterator yielding mutable access to all node weights.
pub struct NodeWeightsMut<'a, N: 'a, Ix: IndexType = DefIndex> {
    nodes: ::std::slice::IterMut<'a, Node<N, Ix>>,
}

impl<'a, N, Ix> Iterator for NodeWeightsMut<'a, N, Ix> where
    Ix: IndexType,
{
    type Item = &'a mut N;

    fn next(&mut self) -> Option<&'a mut N> {
        self.nodes.next().map(|node| &mut node.weight)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.nodes.size_hint()
    }
}

/// Iterator yielding mutable access to all edge weights.
pub struct EdgeWeightsMut<'a, E: 'a, Ix: IndexType = DefIndex> {
    edges: ::std::slice::IterMut<'a, Edge<E, Ix>>,
}

impl<'a, E, Ix> Iterator for EdgeWeightsMut<'a, E, Ix> where
    Ix: IndexType,
{
    type Item = &'a mut E;

    fn next(&mut self) -> Option<&'a mut E> {
        self.edges.next().map(|edge| &mut edge.weight)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.edges.size_hint()
    }
}

/// Index the `Graph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> Index<NodeIndex<Ix>> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        &self.nodes[index.index()].weight
    }
}

/// Index the `Graph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> IndexMut<NodeIndex<Ix>> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        &mut self.nodes[index.index()].weight
    }

}

/// Index the `Graph` by `EdgeIndex` to access edge weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> Index<EdgeIndex<Ix>> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Output = E;
    fn index(&self, index: EdgeIndex<Ix>) -> &E {
        &self.edges[index.index()].weight
    }
}

/// Index the `Graph` by `EdgeIndex` to access edge weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> IndexMut<EdgeIndex<Ix>> for Graph<N, E, Ty, Ix> where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn index_mut(&mut self, index: EdgeIndex<Ix>) -> &mut E {
        &mut self.edges[index.index()].weight
    }
}

/// Create a new empty `Graph`.
impl<N, E, Ty, Ix> Default for Graph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn default() -> Self { Self::with_capacity(0, 0) }
}

/// A  `GraphIndex` is a node or edge index.
pub trait GraphIndex : Copy {
    #[doc(hidden)]
    fn index(&self) -> usize;
    #[doc(hidden)]
    fn is_node_index() -> bool;
}

impl<Ix: IndexType> GraphIndex for NodeIndex<Ix> {
    #[inline]
    fn index(&self) -> usize { NodeIndex::index(*self) }
    #[inline]
    fn is_node_index() -> bool { true }
}

impl<Ix: IndexType> GraphIndex for EdgeIndex<Ix> {
    #[inline]
    fn index(&self) -> usize { EdgeIndex::index(*self) }
    #[inline]
    fn is_node_index() -> bool { false }
}

/// A “walker” object that can be used to step through the edge list of a node.
///
/// Created with [`.detach()`](struct.Neighbors.html#method.detach).
///
/// The walker does not borrow from the graph, so it lets you step through
/// neighbors or incident edges while also mutating graph weights, as
/// in the following example:
///
/// ```
/// use petgraph::{Graph, Dfs, Incoming};
///
/// let mut gr = Graph::new();
/// let a = gr.add_node(0.);
/// let b = gr.add_node(0.);
/// let c = gr.add_node(0.);
/// gr.add_edge(a, b, 3.);
/// gr.add_edge(b, c, 2.);
/// gr.add_edge(c, b, 1.);
///
/// // step through the graph and sum incoming edges into the node weight
/// let mut dfs = Dfs::new(&gr, a);
/// while let Some(node) = dfs.next(&gr) {
///     // use a detached neighbors walker
///     let mut edges = gr.neighbors_directed(node, Incoming).detach();
///     while let Some(edge) = edges.next_edge(&gr) {
///         gr[node] += gr[edge];
///     }
/// }
///
/// // check the result
/// assert_eq!(gr[a], 0.);
/// assert_eq!(gr[b], 4.);
/// assert_eq!(gr[c], 2.);
/// ```
pub struct WalkNeighbors<Ix> {
    skip_start: NodeIndex<Ix>,
    next: [EdgeIndex<Ix>; 2],
}

impl<Ix> Clone for WalkNeighbors<Ix>
    where Ix: IndexType,
{
    fn clone(&self) -> Self {
        WalkNeighbors {
            skip_start: self.skip_start,
            next: self.next,
        }
    }
}

impl<Ix: IndexType> WalkNeighbors<Ix> {
    /// Step to the next edge and its endpoint node in the walk for graph `g`.
    ///
    /// The next node indices are always the others than the starting point
    /// where the `WalkNeighbors` value was created.
    /// For an `Outgoing` walk, the target nodes,
    /// for an `Incoming` walk, the source nodes of the edge.
    pub fn next<N, E, Ty: EdgeType>(&mut self, g: &Graph<N, E, Ty, Ix>)
        -> Option<(EdgeIndex<Ix>, NodeIndex<Ix>)> {
        // First any outgoing edges
        match g.edges.get(self.next[0].index()) {
            None => {}
            Some(edge) => {
                let ed = self.next[0];
                self.next[0] = edge.next[0];
                return Some((ed, edge.node[1]));
            }
        }
        // Then incoming edges
        // For an "undirected" iterator (traverse both incoming
        // and outgoing edge lists), make sure we don't double
        // count selfloops by skipping them in the incoming list.
        while let Some(edge) = g.edges.get(self.next[1].index()) {
            let ed = self.next[1];
            self.next[1] = edge.next[1];
            if edge.node[0] != self.skip_start {
                return Some((ed, edge.node[0]));
            }
        }
        None
    }

    pub fn next_node<N, E, Ty: EdgeType>(&mut self, g: &Graph<N, E, Ty, Ix>)
        -> Option<NodeIndex<Ix>>
    {
        self.next(g).map(|t| t.1)
    }

    pub fn next_edge<N, E, Ty: EdgeType>(&mut self, g: &Graph<N, E, Ty, Ix>)
        -> Option<EdgeIndex<Ix>>
    {
        self.next(g).map(|t| t.0)
    }
}

/// **Deprecated.**
///
/// A “walker” object that can be used to step through the edge list of a node.
///
/// See [*.walk_edges_directed()*](struct.Graph.html#method.walk_edges_directed)
/// for more information.
#[derive(Clone, Debug)]
pub struct WalkEdges<Ix: IndexType = DefIndex> {
    next: EdgeIndex<Ix>, // a valid index or EdgeIndex::max()
    direction: EdgeDirection,
}

impl<Ix: IndexType> WalkEdges<Ix> {
    /// Fetch the next edge index in the walk for graph `g`.
    pub fn next<N, E, Ty: EdgeType>(&mut self, g: &Graph<N, E, Ty, Ix>) -> Option<EdgeIndex<Ix>> {
        self.next_neighbor(g).map(|(e, _)| e)
    }

    /// Fetch the next edge index and the next node index in the walk for graph `g`.
    ///
    /// The next node indices are always the others than the starting point
    /// where the `WalkEdges` value was created.
    /// For an `Outgoing` walk, the target nodes,
    /// for an `Incoming` walk, the source nodes of the edge.
    pub fn next_neighbor<N, E, Ty: EdgeType>(&mut self, g: &Graph<N, E, Ty, Ix>)
        -> Option<(EdgeIndex<Ix>, NodeIndex<Ix>)> {
        match g.edges.get(self.next.index()) {
            None => None,
            Some(edge) => {
                let edge_index = self.next;
                self.next = edge.next[self.direction as usize];
                Some((edge_index, edge.node[1 - self.direction as usize]))
            }
        }
    }
}


fn enumerate<I>(iterable: I) -> ::std::iter::Enumerate<I::IntoIter>
    where I: IntoIterator,
{
    iterable.into_iter().enumerate()
}

/// Iterator over the node indices of a graph.
#[derive(Clone, Debug)]
pub struct NodeIndices<Ix = DefIndex> {
    r: Range<usize>,
    ty: PhantomData<Ix>,
}

impl<Ix: IndexType> Iterator for NodeIndices<Ix> {
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.r.next().map(node_index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.r.size_hint()
    }
}

impl<Ix: IndexType> DoubleEndedIterator for NodeIndices<Ix> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.r.next_back().map(node_index)
    }
}

/// Iterator over the edge indices of a graph.
#[derive(Clone, Debug)]
pub struct EdgeIndices<Ix = DefIndex> {
    r: Range<usize>,
    ty: PhantomData<Ix>,
}

impl<Ix: IndexType> Iterator for EdgeIndices<Ix> {
    type Item = EdgeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.r.next().map(edge_index)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.r.size_hint()
    }
}

impl<Ix: IndexType> DoubleEndedIterator for EdgeIndices<Ix> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.r.next_back().map(edge_index)
    }
}

#[cfg(feature = "stable_graph")]
#[path = "stable.rs"]
pub mod stable;
