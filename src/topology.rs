use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Display;

#[derive(Debug, Clone)]
struct PathNode {
    id: NodeId,
    forward_if_id: IfaceIndex,
    reverse_if_id: IfaceIndex,
}

impl PathNode {
    fn new(id: NodeId) -> Self {
        PathNode {
            id,
            forward_if_id: Default::default(),
            reverse_if_id: Default::default(),
        }
    }
}

impl Display for PathNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("({}){}({})", self.reverse_if_id, self.id, self.forward_if_id))
    }
}

#[derive(Clone)]
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

impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("Path: "))?;
        f.write_fmt(format_args!("{}",
                                 self.nodes
                                 .iter()
                                 .map(|x| format!("{}", x))
                                 .collect::<Vec<String>>()
                                 .join(" => ")
        ))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub(crate) struct NodeId(u32);

impl Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{:X}", self.0))
    }
}

#[derive(Debug, PartialEq)]
enum InterfaceType {
    LocalApp,
    LocalNet,
    Internet,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub(crate) struct IfaceIndex(u8);

impl Display for IfaceIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug)]
struct Interface {
    id: IfaceIndex,
    if_type: InterfaceType,
    neighbors: Vec<(NodeId, IfaceIndex)>,
}

impl Interface {
    fn new(id: IfaceIndex,
           if_type: InterfaceType,
           neighbors: Vec<(NodeId, IfaceIndex)>
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

    fn get_adjacent_interface(&self, from_node: NodeId,
                              via_if: IfaceIndex,
                              to_node: NodeId) -> Option<IfaceIndex> {
        let node = self.nodes.get(&from_node).unwrap();
        let iface = node.ifaces.get(&via_if).unwrap();

        for (neigh_id, adj_iface) in &iface.neighbors {
            if *neigh_id == to_node {
                return Some(*adj_iface);
            }
        }
        None
    }

    fn get_local_iface_id_type(&self, id: NodeId, if_type: InterfaceType) -> Option<IfaceIndex> {
        let node = self.nodes.get(&id).unwrap();
        for (if_id, iface) in &node.ifaces {
            if iface.if_type == if_type {
                return Some(*if_id);
            }
        }
        return None;
    }

    fn get_local_app_iface_id(&self, id: NodeId) -> Option<IfaceIndex> {
        self.get_local_iface_id_type(id, InterfaceType::LocalApp)
    }

    fn get_internet_iface_id(&self, id: NodeId) -> Option<IfaceIndex> {
        self.get_local_iface_id_type(id, InterfaceType::Internet)
    }

    fn check_if_visitted(&self, id: NodeId, path: &Path) -> bool {
        for node in &path.nodes {
            if node.id == id {
                return true;
            }
        }
        false
    }

    fn find_path(&self,
                 start_id: NodeId,
                 start_if_id: IfaceIndex,
                 finish_id: NodeId,
                 finish_if_id: IfaceIndex,
                 curr_path: &mut Path,
                 path_vec: &mut Vec<Path>,
    ) -> bool {
        // println!("searching path from {start_id} to {finish_id}");

        let start_node = self.nodes.get(&start_id).unwrap();
        let mut path_node = PathNode::new(start_id);
        path_node.reverse_if_id = start_if_id;
        curr_path.nodes.push_back(path_node);

        if start_id == finish_id {
            let mut last_node = curr_path.nodes.back_mut().unwrap();
            last_node.forward_if_id = finish_if_id;

            // println!("found finish node {finish_id}");
            // println!("{}", curr_path);

            path_vec.push(curr_path.clone());

            return true;
        }

        let mut ifaces_to_visit: Vec<IfaceIndex> = start_node.ifaces.values()
                                                                    .filter(|&x| x.if_type == InterfaceType::LocalNet)
                                                                    .map(|x| x.id)
                                                                    .collect();

        let mut flag = false;

        for (if_id, iface) in start_node.ifaces.iter() {
            ifaces_to_visit.retain(|&x| x != *if_id);

            for (neigh_id, neigh_if_id) in &iface.neighbors {
                if !self.check_if_visitted(*neigh_id, curr_path) {
                    let mut last_node = curr_path.nodes.back_mut().unwrap();
                    last_node.forward_if_id = *if_id;

                    // println!("visiting {start_id}({if_id}) => {neigh_id}({neigh_if_id})");
                    if self.find_path(*neigh_id, *neigh_if_id, finish_id, finish_if_id, curr_path, path_vec) {
                        // println!("found path from {start_id} to {finish_id}");
                        if !ifaces_to_visit.is_empty() {
                            let _tail = curr_path.nodes.pop_back().unwrap();
                            // println!("{start_id}: not all interfaces were visitted. extracting {}", tail.id);
                            flag = true;
                            continue;
                        }
                        else {
                            let _tail = curr_path.nodes.pop_back().unwrap();
                            // println!("{start_id}: all interfaces were visitted. extracting {}", tail.id);
                            return true;
                        }
                    }
                    else {
                        let _tail = curr_path.nodes.pop_back().unwrap();
                        // println!("{start_id}: this is the dead end. extracting {}", tail.id);
                    }
                }
                else {
                    // println!("{start_id}: neighbor {neigh_id} was already visitted");
                }
            }
            if !iface.neighbors.is_empty() {
                // println!("{start_id}: we have seen all available neighbors over interface {if_id}");
            }
        }
        // println!("{start_id}: we have seen all available interfaces");
        flag
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
        let if_1 = IfaceIndex(1);
        let if_2 = IfaceIndex(2);

        let if_a_1 = Interface::new(if_1, InterfaceType::LocalNet, vec![(n_b, if_1)]);
        let if_b_1 = Interface::new(if_1, InterfaceType::LocalNet, vec![(n_a, if_1)]);
        let if_b_2 = Interface::new(if_2, InterfaceType::LocalNet, vec![(n_c, if_1)]);
        let if_c_1 = Interface::new(if_1, InterfaceType::LocalNet, vec![(n_b, if_2)]);

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
        assert_eq!(gts.len(), 2);
    }

    #[test]
    fn find_adjacent_interface() {
        let n_a = NodeId(0xA);
        let n_b = NodeId(0xB);
        let n_c = NodeId(0xC);

        let if_1 = IfaceIndex(1);
        let if_2 = IfaceIndex(2);

        // Internet -- (2)A(1) -- (1)B(2) -- (1)C(2) -- Internet
        let topo = create_line_topology_with_internet_2();

        assert_eq!(topo.get_adjacent_interface(n_a, if_1, n_a), None);
        assert_eq!(topo.get_adjacent_interface(n_a, if_2, n_a), None);
        assert_eq!(topo.get_adjacent_interface(n_a, if_1, n_b), Some(if_1));
        assert_eq!(topo.get_adjacent_interface(n_a, if_2, n_b), None);
        assert_eq!(topo.get_adjacent_interface(n_a, if_2, n_c), None);
        assert_eq!(topo.get_adjacent_interface(n_a, if_1, n_c), None);

        assert_eq!(topo.get_adjacent_interface(n_b, if_1, n_a), Some(if_1));
        assert_eq!(topo.get_adjacent_interface(n_b, if_2, n_a), None);
        assert_eq!(topo.get_adjacent_interface(n_b, if_1, n_b), None);
        assert_eq!(topo.get_adjacent_interface(n_b, if_2, n_b), None);
        assert_eq!(topo.get_adjacent_interface(n_b, if_2, n_c), Some(if_1));
        assert_eq!(topo.get_adjacent_interface(n_b, if_1, n_c), None);

        assert_eq!(topo.get_adjacent_interface(n_c, if_1, n_a), None);
        assert_eq!(topo.get_adjacent_interface(n_c, if_2, n_a), None);
        assert_eq!(topo.get_adjacent_interface(n_c, if_1, n_b), Some(if_2));
        assert_eq!(topo.get_adjacent_interface(n_c, if_2, n_b), None);
        assert_eq!(topo.get_adjacent_interface(n_c, if_1, n_c), None);
        assert_eq!(topo.get_adjacent_interface(n_c, if_2, n_c), None);
    }

    #[test]
    fn find_path() {
        let n_a = NodeId(0xA);
        let n_c = NodeId(0xC);

        let topo = create_line_topology();
        let mut path = Path::new();
        let mut paths: Vec<Path> = Vec::new();

        topo.find_path(n_a,
                       topo.get_local_app_iface_id(n_a).unwrap(),
                       n_c,
                       topo.get_local_app_iface_id(n_c).unwrap(),
                       &mut path, &mut paths);
    }

    // Internet -- (1)A(2) -- (1)B(4) -- (1)C(2) -- Internet
    //                        (2)(3)    (3)(4)
    //                         |  |    /    |
    //                        (2)(3)(4)     (2)
    //                D(1) -- (1)E(5) -- (1)F
    fn create_big_topology() -> Topology {
        let n_a = NodeId(0xA);
        let n_b = NodeId(0xB);
        let n_c = NodeId(0xC);
        let n_d = NodeId(0xD);
        let n_e = NodeId(0xE);
        let n_f = NodeId(0xF);

        let if_0 = IfaceIndex(0);
        let if_1 = IfaceIndex(1);
        let if_2 = IfaceIndex(2);
        let if_3 = IfaceIndex(3);
        let if_4 = IfaceIndex(4);
        let if_5 = IfaceIndex(5);

        let if_a_a = Interface::new(if_0, InterfaceType::LocalApp, vec![]);
        let if_a_1 = Interface::new(if_1, InterfaceType::Internet, vec![]);
        let if_a_2 = Interface::new(if_2, InterfaceType::LocalNet, vec![(n_b, if_1)]);

        let if_b_a = Interface::new(if_0, InterfaceType::LocalApp, vec![]);
        let if_b_1 = Interface::new(if_1, InterfaceType::LocalNet, vec![(n_a, if_2)]);
        let if_b_2 = Interface::new(if_2, InterfaceType::LocalNet, vec![(n_e, if_2)]);
        let if_b_3 = Interface::new(if_3, InterfaceType::LocalNet, vec![(n_e, if_3)]);
        let if_b_4 = Interface::new(if_4, InterfaceType::LocalNet, vec![(n_c, if_1)]);

        let if_c_a = Interface::new(if_0, InterfaceType::LocalApp, vec![]);
        let if_c_1 = Interface::new(if_1, InterfaceType::LocalNet, vec![(n_b, if_4)]);
        let if_c_2 = Interface::new(if_2, InterfaceType::Internet, vec![]);
        let if_c_3 = Interface::new(if_3, InterfaceType::LocalNet, vec![(n_e, if_4)]);
        let if_c_4 = Interface::new(if_4, InterfaceType::LocalNet, vec![(n_f, if_2)]);

        let if_d_a = Interface::new(if_0, InterfaceType::LocalApp, vec![]);
        let if_d_1 = Interface::new(if_1, InterfaceType::LocalNet, vec![(n_e, if_1)]);

        let if_e_a = Interface::new(if_0, InterfaceType::LocalApp, vec![]);
        let if_e_1 = Interface::new(if_1, InterfaceType::LocalNet, vec![(n_d, if_1)]);
        let if_e_2 = Interface::new(if_2, InterfaceType::LocalNet, vec![(n_b, if_2)]);
        let if_e_3 = Interface::new(if_3, InterfaceType::LocalNet, vec![(n_b, if_3)]);
        let if_e_4 = Interface::new(if_4, InterfaceType::LocalNet, vec![(n_c, if_3)]);
        let if_e_5 = Interface::new(if_5, InterfaceType::LocalNet, vec![(n_f, if_1)]);

        let if_f_a = Interface::new(if_0, InterfaceType::LocalApp, vec![]);
        let if_f_1 = Interface::new(if_1, InterfaceType::LocalNet, vec![(n_e, if_5)]);
        let if_f_2 = Interface::new(if_2, InterfaceType::LocalNet, vec![(n_c, if_4)]);

        let mut node_a = TopologyNode::new(n_a);
        let mut node_b = TopologyNode::new(n_b);
        let mut node_c = TopologyNode::new(n_c);
        let mut node_d = TopologyNode::new(n_d);
        let mut node_e = TopologyNode::new(n_e);
        let mut node_f = TopologyNode::new(n_f);

        node_a.add_iface(if_a_a);
        node_a.add_iface(if_a_1);
        node_a.add_iface(if_a_2);

        node_b.add_iface(if_b_a);
        node_b.add_iface(if_b_1);
        node_b.add_iface(if_b_2);
        node_b.add_iface(if_b_3);
        node_b.add_iface(if_b_4);

        node_c.add_iface(if_c_a);
        node_c.add_iface(if_c_1);
        node_c.add_iface(if_c_2);
        node_c.add_iface(if_c_3);
        node_c.add_iface(if_c_4);

        node_d.add_iface(if_d_a);
        node_d.add_iface(if_d_1);

        node_e.add_iface(if_e_a);
        node_e.add_iface(if_e_1);
        node_e.add_iface(if_e_2);
        node_e.add_iface(if_e_3);
        node_e.add_iface(if_e_4);
        node_e.add_iface(if_e_5);

        node_f.add_iface(if_f_a);
        node_f.add_iface(if_f_1);
        node_f.add_iface(if_f_2);

        let mut topo = Topology::new();
        topo.add_node(node_a);
        topo.add_node(node_b);
        topo.add_node(node_c);
        topo.add_node(node_d);
        topo.add_node(node_e);
        topo.add_node(node_f);

        topo
    }

    #[test]
    fn find_path_in_big_topo_d_c() {
        let n_c = NodeId(0xC);
        let n_d = NodeId(0xD);

        let topo = create_big_topology();
        let mut path = Path::new();
        let mut paths: Vec<Path> = Vec::new();

        topo.find_path(n_d, topo.get_local_app_iface_id(n_d).unwrap(),
                       n_c, topo.get_internet_iface_id(n_c).unwrap(),
                       &mut path, &mut paths);

        for found_path in paths {
            println!("{found_path}");
        }
    }
}
