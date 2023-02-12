use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Debug)]
struct PathNode {
    id: NodeId,
    forward_if_id: IfaceIndex,
    reverse_if_id: IfaceIndex,
}

struct Path {
    nodes: VecDeque<PathNode>,
}

impl Path {
    fn new() -> Self {
        Path {
            nodes: VecDeque::new()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct NodeId(u32);

#[derive(Debug, PartialEq)]
enum InterfaceType {
    LocalApp,
    LocalNet,
    Internet,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct IfaceIndex(u8);

#[derive(Debug)]
struct Interface {
    id: IfaceIndex,
    if_type: InterfaceType,
    neighbors: Vec<NodeId>,
}

impl Interface {
    fn new(id: IfaceIndex,
           if_type: InterfaceType,
           neighbors: Vec<NodeId>
    ) -> Self {
        Self {
            id,
            if_type,
            neighbors,
        }
    }
}

#[derive(Debug)]
struct TopologyNode {
    id: NodeId,
    ifaces: HashMap<IfaceIndex, Interface>,
}

impl TopologyNode {
    fn new(id: NodeId) -> Self {
        Self {
            id,
            ifaces: HashMap::new(),
        }
    }

    fn add_iface(&mut self, iface: Interface) {
        self.ifaces.insert(iface.id, iface);
    }
}

struct Topology {
    nodes: HashMap<NodeId, TopologyNode>,
}

impl Topology {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    fn add_node(&mut self, node: TopologyNode) {
        self.nodes.insert(node.id, node);
    }

    fn get_node_mut(&mut self, node_id: NodeId) -> &mut TopologyNode {
        self.nodes.get_mut(&node_id).unwrap()
    }

    fn find_internet_gateway(&self) -> Vec<NodeId> {
        let mut res: Vec<NodeId> = Vec::new();

        for (&n_id, node) in self.nodes.iter() {
            for iface in node.ifaces.values() {
                if iface.if_type == InterfaceType::Internet {
                    res.push(n_id);
                    break;
                }
            }
        }
        res
    }

    fn find_path(self, start_id: NodeId, finish_id: NodeId) -> Path {
        Path::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // A(1) -- (1)B(2) -- (1)C
    fn create_line_topology() -> Topology {
        let n_a = NodeId(0xA);
        let n_b = NodeId(0xB);
        let n_c = NodeId(0xC);

        let if_a_1 = Interface::new(IfaceIndex(1), InterfaceType::LocalNet, vec![n_b]);
        let if_b_1 = Interface::new(IfaceIndex(1), InterfaceType::LocalNet, vec![n_a]);
        let if_b_2 = Interface::new(IfaceIndex(2), InterfaceType::LocalNet, vec![n_c]);
        let if_c_1 = Interface::new(IfaceIndex(1), InterfaceType::LocalNet, vec![n_b]);

        let mut node_a = TopologyNode::new(n_a);
        let mut node_b = TopologyNode::new(n_b);
        let mut node_c = TopologyNode::new(n_c);

        node_a.add_iface(if_a_1);
        node_b.add_iface(if_b_1);
        node_b.add_iface(if_b_2);
        node_c.add_iface(if_c_1);

        let mut topo = Topology::new();
        topo.add_node(node_a);
        topo.add_node(node_b);
        topo.add_node(node_c);
        topo
    }

    // A(1) -- (1)B(2) -- (1)C(2) -- Internet
    fn create_line_topology_with_internet() -> Topology {
        let mut topo = create_line_topology();
        let if_c_2 = Interface::new(IfaceIndex(2), InterfaceType::Internet, vec![]);
        let node_c = topo.get_node_mut(NodeId(0xC));

        node_c.add_iface(if_c_2);

        topo
    }

    // Internet -- (2)A(1) -- (1)B(2) -- (1)C(2) -- Internet
    fn create_line_topology_with_internet_2() -> Topology {
        let mut topo = create_line_topology_with_internet();
        let if_a_2 = Interface::new(IfaceIndex(2), InterfaceType::Internet, vec![]);
        let node_a = topo.get_node_mut(NodeId(0xA));

        node_a.add_iface(if_a_2);

        topo
    }

    #[test]
    fn find_gateway_no_internet() {
        let topo = create_line_topology();
        let gts = topo.find_internet_gateway();
        assert!(gts.is_empty());
    }

    #[test]
    fn find_gateway_internet() {
        let topo = create_line_topology_with_internet();
        let gts = topo.find_internet_gateway();
        assert!(!gts.is_empty());
        assert!(gts.len() == 1);
    }

    #[test]
    fn find_gateway_inetnet_2() {
        let topo = create_line_topology_with_internet_2();
        let gts = topo.find_internet_gateway();
        assert!(!gts.is_empty());
        assert!(gts.len() == 2);
    }
}
