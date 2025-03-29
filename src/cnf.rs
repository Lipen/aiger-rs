use std::collections::HashMap;

use crate::aig::Aig;
use crate::node::Node;
use crate::reference::Ref;

impl Aig {
    pub fn to_cnf(&self) -> (Vec<Vec<i32>>, HashMap<u32, u32>) {
        if !self.latches().is_empty() {
            panic!("Can't convert AIG with latches to CNF");
        }

        let mut mapping = HashMap::new(); // {id: var}
        let mut clauses = Vec::new();

        for (i, &id) in self.inputs().iter().enumerate() {
            mapping.insert(id, i as u32 + 1);
        }

        fn ref2lit(r: Ref, mapping: &HashMap<u32, u32>) -> i32 {
            let var = mapping[&r.id()];
            let lit = var as i32;
            if r.is_negated() {
                -lit
            } else {
                lit
            }
        }

        for (i, layer) in self.layers_input().enumerate().skip(1) {
            for id in layer {
                match self.node(id) {
                    Node::Zero => {
                        panic!("Unexpected zero on level {}", i);
                    }
                    Node::Input(_) => {
                        panic!("Unexpected input on level {}", i);
                    }
                    Node::Latch(latch) => {
                        panic!("Unexpected latch on level {}: {:?}", i, latch);
                    }
                    Node::AndGate(gate) => {
                        let x = mapping.len() as u32 + 1;
                        mapping.insert(id, x);
                        let x = x as i32;
                        let [left, right] = gate.args;
                        match (left.get_const(), right.get_const()) {
                            (Some(l), Some(r)) => {
                                // Both LHS and RHS are constants
                                if l && r {
                                    clauses.push(vec![x]);
                                } else {
                                    clauses.push(vec![-x]);
                                }
                            }
                            (Some(l), None) => {
                                // LHS is constant
                                let rhs = ref2lit(right, &mapping);
                                if l {
                                    clauses.push(vec![x, -rhs]);
                                    clauses.push(vec![-x, rhs]);
                                } else {
                                    clauses.push(vec![-x]);
                                }
                            }
                            (None, Some(r)) => {
                                // RHS is constant
                                let lhs = ref2lit(left, &mapping);
                                if r {
                                    clauses.push(vec![x, -lhs]);
                                    clauses.push(vec![-x, lhs]);
                                } else {
                                    clauses.push(vec![-x]);
                                }
                            }
                            (None, None) => {
                                // Normal AND
                                let lhs = ref2lit(left, &mapping);
                                let rhs = ref2lit(right, &mapping);
                                clauses.push(vec![x, -lhs, -rhs]);
                                clauses.push(vec![-x, lhs]);
                                clauses.push(vec![-x, rhs]);
                            }
                        }
                    }
                }
            }
        }

        (clauses, mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::reference::Ref;

    #[test]
    fn test_to_cnf() {
        let mut aig = Aig::default();

        aig.add_input(1);
        aig.add_input(2);
        aig.add_input(3);
        aig.add_and_gate(4, [Ref::positive(1), Ref::positive(2)]); // 4 = 1 and 2
        aig.add_and_gate(5, [Ref::positive(4), Ref::positive(3)]); // 5 = 4 and 3
        aig.add_and_gate(6, [Ref::positive(5), Ref::FALSE]); // 6 = 5 and 0
        aig.add_output(Ref::positive(6));

        println!("Backward (input) layers: {}", aig.layers_input().count());
        for layer in aig.layers_input() {
            println!("  {:?}", layer);
        }

        let (clauses, _) = aig.to_cnf();
        println!("CNF of {} clauses:", clauses.len());
        for clause in clauses.iter() {
            println!(
                "{}0",
                clause.iter().map(|x| format!("{} ", x)).collect::<String>()
            );
        }
        assert_eq!(clauses.len(), 7);
    }
}
