use petgraph::dot::Dot;
use petgraph::visit::EdgeRef;
use petgraph::Direction::{Incoming, Outgoing};
use petgraph::{Directed, Graph};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[allow(dead_code)]
struct PerfectPhylogeny {
    matrix: Vec<Vec<usize>>,
    order: Vec<(usize, usize)>,
    perfect: bool,
    tree: Graph<String, String>,
}

impl PerfectPhylogeny {
    #[allow(dead_code)]
    pub fn new(mat: Vec<Vec<usize>>) -> PerfectPhylogeny {
        let rowlen = mat.len();
        let collen = mat[0].len();
        let mut dim = vec![(0, 0); collen];
        // get order to read matrix
        for j in 0..collen {
            let mut sum = 0;
            for i in 0..rowlen {
                sum += mat[i][j];
            }
            dim[j] = (sum, j);
        }

        dim.sort_by_key(|&(a, b)| (!a, b));
        let check = PerfectPhylogeny::is_laminar(&mat, &dim);
        if !check {
            return PerfectPhylogeny {
                matrix: mat,
                order: dim,
                perfect: false,
                tree: Graph::new(),
            };
        }
        let mut graph: Graph<String, String> = Graph::new();
        let source = graph.add_node("Root".to_string());
        // follow algorithm
        for i in 0..rowlen {
            let mut curr = source;
            for j in dim.iter() {
                if mat[i][j.1] == 1 {
                    let mut found_node = false;
                    for adj in graph.edges(curr) {
                        if adj.weight() == &format!("C_{}", j.1 + 1) {
                            curr = adj.target();
                            found_node = true;
                            break;
                        }
                    }
                    if !found_node {
                        let node = graph.add_node("".to_string());
                        let label = format!("C_{}", j.1 + 1);
                        graph.update_edge(curr, node, label.clone());
                        curr = node;
                    }
                }
            }
            *graph.node_weight_mut(curr).unwrap() = format!("S_{}", i + 1);
        }

        // create leaves for interal node with label
        for node in graph.node_indices() {
            if graph.node_weight(node).unwrap() != "Root"
                && graph.node_weight(node).unwrap() != ""
                && graph.neighbors_directed(node, Outgoing).count() != 0
            {
                let label = graph.node_weight(node).unwrap().clone();
                let new_node = graph.add_node(label);
                *graph.node_weight_mut(node).unwrap() = "".to_string();
                graph.add_edge(node, new_node, "".to_string());
            }
        }

        // delete useless node and compact labels
        let mut node_to_remove = Vec::new();
        for node in graph.node_indices() {
            if graph.node_weight(node).unwrap() == ""
                && graph.neighbors_directed(node, Outgoing).count() == 1
                && graph.neighbors_directed(node, Incoming).count() == 1
            {
                let node_in = graph.neighbors_directed(node, Incoming).next().unwrap();
                let node_out = graph.neighbors_directed(node, Outgoing).next().unwrap();
                let edge_in = graph.edges_connecting(node_in, node);
                let edge_out = graph.edges_connecting(node, node_out);
                let label = format!(
                    "{},{}",
                    edge_in.into_iter().next().unwrap().weight(),
                    edge_out.into_iter().next().unwrap().weight()
                );
                let _ = graph.add_edge(node_in, node_out, label);
                node_to_remove.push(node);
            }
        }
        for node in node_to_remove {
            graph.remove_node(node);
        }
        PerfectPhylogeny {
            matrix: mat,
            order: dim,
            perfect: true,
            tree: graph,
        }
    }
    #[allow(dead_code)]
    pub fn from_file(file: &str) -> PerfectPhylogeny {
        let f = BufReader::new(File::open(file).unwrap());
        let mut mat = Vec::new();
        for line in f.lines() {
            let tmp: Vec<usize> = line
                .unwrap()
                .split_whitespace()
                .map(|c| c.parse::<usize>().unwrap())
                .collect();
            mat.push(tmp);
        }
        PerfectPhylogeny::new(mat)
    }

    #[allow(dead_code)]
    pub fn matrix(&self) -> &Vec<Vec<usize>> {
        &self.matrix
    }
    #[allow(dead_code)]
    pub fn order(&self) -> &Vec<(usize, usize)> {
        &self.order
    }
    #[allow(dead_code)]
    pub fn perfect(&self) -> bool {
        self.perfect
    }
    #[allow(dead_code)]
    pub fn tree(&self) -> &Graph<String, String, Directed, u32> {
        &self.tree
    }

    fn is_laminar(mat: &Vec<Vec<usize>>, order: &Vec<(usize, usize)>) -> bool {
        let rowlen = mat.len();
        let collen = mat[0].len();
        let mut lmat = vec![vec![0; collen]; rowlen];
        for i in 0..rowlen {
            let mut k = -1;
            for j in order.iter() {
                if mat[i][j.1] == 1 {
                    lmat[i][j.1] = k;
                    k = j.1 as i32;
                }
            }
        }
        for j in 0..collen {
            let mut first = true;
            let mut tmp = 0;
            for i in 0..rowlen {
                if first && lmat[i][j] != 0 {
                    tmp = lmat[i][j];
                    first = false;
                }
                if lmat[i][j] != 0 && lmat[i][j] != tmp {
                    return false;
                }
            }
        }
        true
    }

    #[allow(dead_code)]
    pub fn get_dot(&self, output: &str) {
        let dot = format!("{:?}", Dot::new(&self.tree));
        let mut fileout = File::create(output).expect("error");
        fileout.write_all(dot.as_bytes()).expect("error");
    }
}

#[cfg(test)]
mod tests {
    use crate::PerfectPhylogeny;

    #[test]
    fn test_phylo() {
        let per_phy = PerfectPhylogeny::from_file("input/matrix.txt");
        per_phy.get_dot("output/final.dot");
        assert!(per_phy.perfect());
    }

    #[test]
    fn test_phylo_nop() {
        let per_phy = PerfectPhylogeny::from_file("input/matrix_nop.txt");
        assert!(!per_phy.perfect());
    }

    #[test]
    fn test_phylo_wiki() {
        let per_phy = PerfectPhylogeny::from_file("input/matrix2.txt");
        per_phy.get_dot("output/wiki.dot");
        assert!(per_phy.perfect());
    }
}
