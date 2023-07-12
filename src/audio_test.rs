use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use petgraph::data::DataMapMut;
use petgraph::graph::{EdgeIndex, NodeIndex};
use petgraph::stable_graph::StableGraph;
use petgraph::visit::{
    Data, DfsPostOrder, GraphBase, IntoNeighborsDirected, NodeIndexable, Reversed, Visitable,
};
use petgraph::Incoming;

pub struct NodeData<T> {
    pub buffer: i32,
    pub node: T,
}

impl<T> NodeData<T> {
    pub fn new(node: T) -> Self {
        NodeData { node, buffer: 0 }
    }
}

pub struct Input {
    pub node_id: usize,
    data: i32,
}

impl Input {
    fn new(node_id: usize, data: i32) -> Self {
        Input { node_id, data }
    }
}

pub trait Node {
    fn process(&mut self, inputs: &HashMap<usize, Input>, output: &mut i32);
}

pub struct BoxedNode(pub Box<dyn Node>);

impl BoxedNode {
    pub fn new(node: impl Node + 'static) -> Self {
        BoxedNode(Box::new(node))
    }
}

impl Deref for BoxedNode {
    type Target = Box<dyn Node>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BoxedNode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct Processor<G: Visitable> {
    dfs_post_order: DfsPostOrder<G::NodeId, G::Map>,
    inputs: HashMap<usize, Input>,
}

impl<G> Processor<G>
where
    G: Visitable + NodeIndexable,
{
    pub fn new() -> Self
    where
        G::Map: Default,
    {
        let mut dfs_post_order = DfsPostOrder::default();
        dfs_post_order.stack = Vec::new();
        let inputs = HashMap::new();
        Self {
            dfs_post_order,
            inputs,
        }
    }

    pub fn process(&mut self, graph: &mut G, node: G::NodeId)
    where
        G: Visitable + DataMapMut + Data<NodeWeight = AudioNodeData>,
        for<'a> &'a G: GraphBase<NodeId = G::NodeId> + IntoNeighborsDirected,
    {
        const NO_NODE: &str = "No node exists with the given index";
        self.dfs_post_order.reset(Reversed(&*graph));
        self.dfs_post_order.move_to(node);
        while let Some(n) = self.dfs_post_order.next(Reversed(&*graph)) {
            self.inputs.clear();
            for in_n in graph.neighbors_directed(n, Incoming) {
                if n == in_n {
                    continue;
                }

                let node_id: G::NodeId = in_n;
                let node_index = graph.to_index(node_id);
                let in_node_data: &AudioNodeData = graph.node_weight(node_id).expect(NO_NODE);
                let input = Input::new(node_index, in_node_data.buffer);
                self.inputs.insert(node_index, input);
            }

            let data: &mut AudioNodeData = graph.node_weight_mut(n).expect(NO_NODE);
            data.node.process(&self.inputs, &mut data.buffer);
        }
    }
}

pub type AudioNodeData = NodeData<BoxedNode>;
pub type AudioGraph = StableGraph<AudioNodeData, ()>;

pub struct AudioContext {
    pub graph: AudioGraph,
    pub destination: NodeIndex,
    pub input: NodeIndex,
    pub processor: Processor<AudioGraph>,
}

impl AudioContext {
    pub fn new() -> Self {
        let mut graph = AudioGraph::new();
        let destination = graph.add_node(AudioNodeData::new(BoxedNode::new(Sum2)));
        let input = graph.add_node(AudioNodeData::new(BoxedNode::new(Pass)));
        let processor = Processor::new();
        AudioContext {
            graph,
            input,
            destination,
            processor,
        }
    }

    pub fn add_node(&mut self, node: impl Node + 'static) -> NodeIndex {
        self.graph.add_node(NodeData::new(BoxedNode::new(node)))
    }

    pub fn connect(&mut self, from: NodeIndex, to: NodeIndex) -> EdgeIndex {
        self.graph.add_edge(from, to, ())
    }

    pub fn next_block(&mut self) -> i32 {
        self.processor.process(&mut self.graph, self.destination);
        self.graph[self.destination].buffer
    }
}

pub struct Sum2;

impl Node for Sum2 {
    fn process(&mut self, inputs: &HashMap<usize, Input>, output: &mut i32) {
        *output = inputs
            .values()
            .map(|input| input.data)
            .reduce(|acc, v| acc + v)
            .unwrap();
    }
}

pub struct Pass;

impl Node for Pass {
    fn process(&mut self, inputs: &HashMap<usize, Input>, output: &mut i32) {
        let input = match inputs.values().next() {
            Some(input) => input.data,
            None => return,
        };
        *output = input;
    }
}

pub struct ConstSig {
    value: i32,
}

impl ConstSig {
    pub fn new(value: i32) -> Self {
        ConstSig { value }
    }
}

impl Node for ConstSig {
    fn process(&mut self, _inputs: &HashMap<usize, Input>, output: &mut i32) {
        *output = self.value
    }
}

pub struct ConstSig {
    val: f32,
}

impl ConstSig {
    pub fn new(val: f32) -> Self {
        ConstSig { val }
    }
}

impl Into<BoxedNode> for ConstSig {
    fn into(self) -> BoxedNode {
        BoxedNode::new(self)
    }
}

pub fn test_audio_context() {
    let mut context = AudioContext::new();
    let const_sig_10 = context.add_node(ConstSig::new(10));
    let const_sig_43 = context.add_node(ConstSig::new(43));
    let pass_node = context.add_node(Pass);
    let sum_node = context.add_node(Sum2);
    context.connect(const_sig_10, pass_node);
    context.connect(pass_node, sum_node);
    context.connect(const_sig_43, sum_node);
    context.connect(sum_node, context.destination);
    println!("dest block {:?}", context.next_block());
}
