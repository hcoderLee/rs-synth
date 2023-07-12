use petgraph::data::DataMapMut;
use petgraph::stable_graph::StableGraph;
use petgraph::visit::{
    Data, DfsPostOrder, GraphBase, IntoNeighborsDirected, Reversed, VisitMap, Visitable,
};
use petgraph::{Incoming, Outgoing};

pub fn test_dfs() {
    let mut graph = StableGraph::<String, ()>::with_capacity(20, 400);
    let a = graph.add_node("a".to_string());
    let b = graph.add_node("b".to_string());
    let c = graph.add_node("c".to_string());
    let d = graph.add_node("d".to_string());
    let dest = graph.add_node("dest".to_string());
    graph.add_edge(a, b, ());
    graph.add_edge(b, c, ());
    graph.add_edge(c, a, ());
    graph.add_edge(c, d, ());
    graph.add_edge(d, dest, ());
    dfs(&mut graph, d);
}

fn dfs<G>(graph: &G, node: G::NodeId)
where
    G: Visitable + DataMapMut + Data<NodeWeight = String>,
    for<'a> &'a G: GraphBase<NodeId = G::NodeId> + IntoNeighborsDirected,
    G::Map: VisitMap<G::NodeId>,
{
    let r_graph = Reversed(graph);
    let mut dfs_post_order: DfsPostOrder<G::NodeId, G::Map> = DfsPostOrder::new(r_graph, node);
    println!("Traversal graph by dfs post order");
    while let Some(node) = dfs_post_order.next(r_graph) {
        let node_data: &str = graph.node_weight(node).unwrap();
        println!("Current node {}", node_data);
        print!("Incoming edges: ");
        for in_n in graph.neighbors_directed(node, Incoming) {
            let in_node_data: &str = graph.node_weight(in_n).unwrap();
            print!("{:?} ", (in_node_data, node_data))
        }
        print!("\nOutgoing edges: ");
        for o_n in graph.neighbors_directed(node, Outgoing) {
            let o_node_data: &str = graph.node_weight(o_n).unwrap();
            print!("{:?} ", (node_data, o_node_data))
        }
        println!("\n");
    }
}
