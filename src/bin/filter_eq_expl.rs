use std::{collections::BTreeMap, fs};

use prototype::{file_io, items::{self, EqualityExpl, BlamedTermItem, Instantiation}};

/// Just for testing "cg" equality explanations and finding the explanations they depend on.
fn main() {
    let mut eq_expls = BTreeMap::new();
    let mut matches = vec![];
    // let mut input = String::new();
    // println!("Enter filename");
    //     stdin().read_line(&mut input).unwrap();
    //     input = input.strip_suffix('\n').unwrap().to_string();
    //     println!("{}", input);
    let paths = fs::read_dir("logs/vpr_logs").unwrap();
    for path in paths {
        if path.is_err() {
            continue;
        }
        let path = path.unwrap();
        println!("Name: {}", path.path().display());
        for (line_no, l0) in file_io::read_lines(path.path()).unwrap().enumerate() {
            let line = l0.unwrap();
            let l = line.split(' ').collect::<Vec<&str>>();
            match l[0] {
                "[eq-expl]" => {
                    use crate::items::EqualityExpl::*;
            let id = l[1].to_string();
            let id_ = id.clone();
            let kind = l[2];
            let eq_expl = match kind {
                "root" => Root { id },
                "lit" => Literal {
                    from: id,
                    eq: l[3].to_string(),
                    to: l[5].to_string(),
                },
                "cg" => {
                    let semicolon_index = l
                        .iter()
                        .position(|&t| t == ";")
                        .expect("Semicolon should be found");
                    let mut terms = vec![];
                    for i in (3..semicolon_index).step_by(2) {
                        let first = &l[i][1..];
                        let second = &l[i + 1][..l[i + 1].len() - 1];
                        terms.push((first.to_string(), second.to_string()));
                    }
                    let to = l[semicolon_index + 1].to_string();
                    Congruence {
                        from: id,
                        arg_eqs: terms,
                        to,
                    }
                    // For each
                }
                "th" => Theory {
                    from: id,
                    theory: l[3].to_string(),
                    to: l[5].to_string(),
                },
                "ax" => Axiom {
                    from: id,
                    to: l[4].to_string(),
                }, // format #A ax ; #B
                _ => Unknown {
                    from: id,
                    to: l[4].to_string(),
                },
            };
            eq_expls.insert(id_, eq_expl);
                },
                "[new-match]" => {
                    let semicolon_index = l
                .iter()
                .position(|&t| t == ";")
                .expect("Semicolon should be found");
            let bound_terms: Vec<String> = l[4..semicolon_index]
                .iter()
                .map(|&t| String::from(t))
                .collect();
            let mut blamed_terms: Vec<BlamedTermItem> = vec![];
            let fingerprint = u64::from_str_radix(l[1], 16).expect("Should be valid hex string");
            // fingerprints.insert(line_no + 1, fingerprint);
            let quant_id = l[2];
            let pattern_id = l[3];
            let mut equality_expls = vec![];
            let dep_instantiations = vec![];
            // self.temp_dependencies.insert(line_no + 1, vec![]);
            for (i, word) in l[semicolon_index + 1..].iter().enumerate() {
                if let Some(first_term) = word.strip_prefix('(') {
                    // assumes that if we see "(#A", the next word in the split is "#B)"
                    let next_word = l[semicolon_index + i + 2];
                    let second_term = next_word.strip_suffix(')').unwrap();
                    blamed_terms.push(BlamedTermItem::Pair(
                        first_term.to_string(),
                        next_word[..next_word.len() - 1].to_string(),
                    ));
                    equality_expls.append(&mut get_all_equality_expls(first_term, second_term, &eq_expls, line_no + 1)
                    .iter()
                    .map(|expl| format!("{:?}", expl))
                .collect::<Vec<String>>());
                }
            }
            let instant = Instantiation {
                line_no: line_no + 1,
                match_line_no: line_no + 1,
                fingerprint,
                resulting_term: String::from("N/A"),
                z3_gen: 0,
                cost: 1.0,
                quant_id: quant_id.to_string(),
                pattern_id: pattern_id.to_string(),
                yields_terms: vec![],
                bound_terms,
                blamed_terms,
                equality_expls,
                dep_instantiations,
            };
            matches.push(instant);
                },
                _ => {}
            }
        }
        println!("SUCCESS!");
    }
}

/// Returns vec of all the equality explanations needed for (#from #to) as blamed terms.
fn get_all_equality_expls<'a>(from_term: &str, to_term: &str, eq_expls: &'a BTreeMap<String, EqualityExpl>, line_no: usize) -> Vec<&'a EqualityExpl> {
    use EqualityExpl::*;
    let mut result = vec![];
    if from_term != to_term {
        let mut key = from_term;
        while let Some(expl) = eq_expls.get(key) {
            result.push(expl);
            match expl {
                Root { .. } => {break; },
                Literal { to , .. } => { key = to;},
                Congruence { to , .. } => { key = to;},
                Theory { to: term , .. } => { key = term;},
                Axiom { to: term , .. } => { key = term;},
                Unknown { to: term , .. } => { key = term;},
            }
            if key == to_term {
                break;
            }
        }
        if key != to_term {
            key = to_term;
            let mut result2 = vec![];
            while let Some(expl) = eq_expls.get(key) {
                result2.push(expl);
                match expl {
                    Root { .. } => {break; },
                    Literal { to , .. } => { key = to;},
                    Congruence { to , .. } => { key = to;},
                    Theory { to: term , .. } => { key = term;},
                    Axiom { to: term , .. } => { key = term;},
                    Unknown { to: term , .. } => { key = term;},
                }
                if key == from_term {
                    break;
                }
            }
            assert!(key == from_term || result.last().unwrap() == result.last().unwrap(), "Equality retrieval failed at line {} for ({} {}):\n
            from chain: {:?}, to chain: {:?}", line_no, from_term, to_term, result, result2);
            result2.pop();
            result2.reverse();
            result.append(&mut result2);
        }
    }
    result

}