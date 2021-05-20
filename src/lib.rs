use petgraph::dot::Dot;
use petgraph::visit::EdgeRef;
use petgraph::Direction::{Incoming, Outgoing};
use petgraph::{Directed, Graph};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};

#[allow(dead_code)]
pub struct PerfectPhylogeny {
    matrix: Vec<Vec<usize>>,
    order: Vec<(usize, usize)>,
    perfect: bool,
    tree: Graph<String, String>,
}

impl PerfectPhylogeny {
    #[allow(dead_code)]
    pub fn new(mat: &Vec<Vec<usize>>, internal: bool) -> PerfectPhylogeny {
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
                matrix: mat.clone(),
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
            let mut tmp = graph.node_weight_mut(curr).unwrap().clone();
            if tmp == "Root" {
                let node = graph.add_node(format!(" S_{}", i + 1));
                graph.update_edge(source, node, "".to_string());
            } else {
                tmp.push_str(&*format!(" S_{}", i + 1));
                *graph.node_weight_mut(curr).unwrap() = tmp;
            }
        }

        // create leaves for interal node with label
        for node in graph.node_indices() {
            if graph.node_weight(node).unwrap() != "Root"
                && graph.node_weight(node).unwrap() != ""
                && graph.neighbors_directed(node, Outgoing).count() != 0
            {
                let label_tot: Vec<String> = graph.node_weight(node).unwrap().clone().split_whitespace().map(|s| s.to_string()).collect();
                for label in label_tot {
                    if label != "" {
                        let new_node = graph.add_node(label);
                        if !internal {
                            *graph.node_weight_mut(node).unwrap() = "".to_string();
                        }
                        graph.add_edge(node, new_node, "".to_string());
                    }
                }
            }
        }

        // separets leaves
        for node in graph.node_indices() {
            if graph.node_weight(node).unwrap() != "Root"
                && graph.node_weight(node).unwrap() != ""
                && graph.neighbors_directed(node, Outgoing).count() == 0
            {
                let label_tot: Vec<String> = graph.node_weight(node).unwrap().clone().split_whitespace().map(|s| s.to_string()).collect();
                if label_tot.len() > 1 {
                    for label in label_tot {
                        if label != "" {
                            let new_node = graph.add_node(label);
                            if !internal {
                                *graph.node_weight_mut(node).unwrap() = "".to_string();
                            }
                            graph.add_edge(node, new_node, "".to_string());
                        }
                    }
                }
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
            matrix: mat.clone(),
            order: dim,
            perfect: true,
            tree: graph,
        }
    }

    #[allow(dead_code)]
    pub fn from_file(file: &str, internal: bool) -> PerfectPhylogeny {
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
        PerfectPhylogeny::new(&mat, internal)
    }


    #[allow(dead_code)]
    pub fn from_file_err(file: &str, internal: bool) -> PerfectPhylogeny {
        let f = BufReader::new(File::open(&file).unwrap());
        let mut mat = Vec::new();
        let mut err = false;
        for mut line in f.lines() {
            if line.as_ref().unwrap().contains("*") {
                err = true;
                line = Ok(line.as_ref().unwrap().replace("*", "2"));
            }
            let tmp: Vec<usize> = line
                .unwrap()
                .split_whitespace()
                .map(|c| c.parse::<usize>().unwrap())
                .collect();
            mat.push(tmp);
        }
        let mut perf_errs = Vec::new();
        if err == true {
            let correction = vec![(0, 0), (0, 1), (1, 0), (1, 1)];
            for corr in correction {
                println!("testing ({},{})", corr.0, corr.1);
                let mut mattmp = mat.clone();
                println!("{:?}", mattmp);
                let mut first = true;
                for i in 0..mattmp.len() {
                    for j in 0..mattmp[0].len() {
                        if mattmp[i][j] == 2 {
                            if first {
                                mattmp[i][j] = corr.0;
                                first = false;
                            } else {
                                mattmp[i][j] = corr.1;
                            }
                        }
                    }
                }
                println!("{:?}", &mattmp);
                let perf = PerfectPhylogeny::new(&mattmp, internal);
                if perf.perfect() {
                    println!("ok");
                    perf_errs.push(perf);
                }
            }
            println!("total: {}", perf_errs.len());
            for (i, per) in perf_errs.iter().enumerate() {
                let out = file.to_string().clone();
                let names_slash: Vec<&str> = out.split("/").collect();
                let names_dot: Vec<&str> = names_slash[1].split(".").collect();
                let mut outstr = names_dot[0].to_string();
                outstr.push_str("_");
                outstr.push_str(&i.to_string());
                per.get_dot(&format!("output/{}", outstr));
            }
        }
        return PerfectPhylogeny::new(&mat, internal);
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
        /*for i in 0..mat.len() {
            for j in 0..mat[0].len(){
                print!("{} ", mat[i][j]);
            }
            println!();
        }
        println!("*-*-*-*-*-*");
        for i in 0..mat.len() {
            for j in order.iter(){
                print!("{} ", mat[i][j.1]);
            }
            println!();
        }
        println!("*-*-*-*-*-*");*/
        let rowlen = mat.len();
        let collen = mat[0].len();
        let mut lmat = vec![vec![0; collen]; rowlen];
        for i in 0..rowlen {
            let mut k = -1;
            for j in order.iter() {
                if mat[i][j.1] == 1 {
                    lmat[i][j.1] = k;
                    k = (j.1 + 1) as i32;
                }
            }
        }
        for i in 0..lmat.len() {
            for j in 0..lmat[0].len(){
                print!("{} ", lmat[i][j]);
            }
            println!();
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
        let mut dot = format!("{:?}", Dot::new(&self.tree));
        dot = dot.trim().to_string();
        dot = dot.replace("\\\"", "");
        dot = dot.replace("_", "");
        dot = dot.replace("\" ", "\"");
        dot = dot.replace(" \"", "\"");
        let mut fileout = File::create(output).expect("error");
        fileout.write_all(dot.as_bytes()).expect("error");
    }
}

#[cfg(test)]
mod tests {
    use crate::PerfectPhylogeny;

    #[test]
    fn test_phylo() {
        let per_phy = PerfectPhylogeny::from_file("input/matrix.txt", false);
        per_phy.get_dot("output/good.dot");
        assert!(per_phy.perfect());
    }

    #[test]
    fn test_phylo_int() {
        let per_phy = PerfectPhylogeny::from_file("input/matrix.txt", true);
        per_phy.get_dot("output/goodi.dot");
        assert!(per_phy.perfect());
    }

    #[test]
    fn test_phylo_nop() {
        let per_phy = PerfectPhylogeny::from_file("input/matrix_nop.txt", false);
        assert!(!per_phy.perfect());
    }

    #[test]
    fn test_phylo_wiki() {
        let per_phy = PerfectPhylogeny::from_file("input/matrix2.txt", false);
        per_phy.get_dot("output/wiki.dot");
        assert!(per_phy.perfect());
    }

    #[test]
    fn test_phylo_bho() {
        let per_phy = PerfectPhylogeny::from_file("input/matrix_bho.txt", false);
        per_phy.get_dot("output/bho.dot");
        assert!(per_phy.perfect());
    }

    #[test]
    fn test_phylo_err() {
        let per_phy = PerfectPhylogeny::from_file("input/matrix_error.txt", false);
        per_phy.get_dot("output/bho.dot");
        //assert!(per_phy.perfect());
    }

    #[test]
    fn test_phylo_a() {
        //let per_phy = PerfectPhylogeny::from_file("input/matrix_error.txt", false);
        //per_phy.get_dot("output/s.dot");
        //assert!(per_phy.perfect());
        let per_phy = PerfectPhylogeny::from_file("input/matrix_error2.txt", false);
        assert!(per_phy.perfect());
    }

    #[test]
    fn test_phylo_m() {
        //let per_phy = PerfectPhylogeny::from_file("input/matrix_error.txt", false);
        //per_phy.get_dot("output/s.dot");
        //assert!(per_phy.perfect());
        let per_phy = PerfectPhylogeny::from_file("input/m.txt", false);
        assert!(!   per_phy.perfect());
    }
}
