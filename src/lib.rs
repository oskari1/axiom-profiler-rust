mod css;
mod dot_output;
pub mod file_io;
pub mod items;
mod render;
mod sort_filter;
pub mod continue_agent;
pub mod z3parser_rc;

use crate::file_io::*;
use crate::items::*;
use regex::Regex;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::dot_output::output_dot_to_file;
use crate::render::RenderSVG;

const QVAR_REGEX_STR_1: &str = r"\(;(?P<sort>\S+)\)";
const QVAR_REGEX_STR_2: &str = r"\(\|(?P<name>\S+)\|\s;\s\|(?P<sort>\S+)\|\)";

const OUT_FILE_INST: &str = "out/instantiations.txt";
const OUT_FILE_QUANT: &str = "out/quantifiers.txt";
const OUT_FILE_DEP: &str = "out/dependencies.txt";
const OUT_FILE_TERMS: &str = "out/terms.txt";
const OUT_FILE_EQ: &str = "out/eq_expls.txt";
const OUT_FILE_PRETTY_TERM: &str = "out/pretty-printed-text.txt";
const OUT_FILE_PRETTY_QUANT: &str = "out/pretty-printed-quant.txt";
const OUT_FILE_DOT: &str = "out/output.dot";
const OUT_FILE_CSS: &str = "out/styles.css";
const OUT_FILE_SVG_2: &str = "out/output2.svg";
const OUT_FILE_SVG: &str = "out/output.svg";

// TODO: struct for outputs; connect to settings (see file_io.rs; add settings)
pub trait LogParser {
    fn read_and_parse_file(
        &mut self,
        filename: &str,
        settings: &Settings,
    ) -> Result<(String,), String>;
    fn should_continue(&self) -> bool;
    fn get_continue_mutex(&self) -> Arc<Mutex<bool>>;
}

pub trait Z3LogParser : LogParser {
    fn version_info(&mut self, l: &[&str]);
    fn mk_quant(&mut self, l: &[&str]);
    fn mk_var(&mut self, l: &[&str]);
    fn mk_proof_app(&mut self, l: &[&str]);
    fn attach_meaning(&mut self, l: &[&str]);
    fn attach_vars(&mut self, l: &[&str], qvar_re_1: &Regex, l0: &str, qvar_re_2: &Regex);
    fn attach_enode(&mut self, l: &[&str]);
    fn eq_expl(&mut self, l: &[&str]);
    fn new_match(&mut self, l: &[&str], line_no: usize);
    fn inst_discovered(&mut self, l: &[&str], line_no: usize, l0: &str);
    fn instance(&mut self, l: &[&str], line_no: usize);
    fn end_of_instance(&mut self);

    fn decide_and_or(&mut self, _l: &[&str]) {}
    fn decide(&mut self, _l: &[&str]) {}
    fn assign(&mut self, _l: &[&str]) {}
    fn push(&mut self, _l: &[&str]) {}
    fn pop(&mut self, _l: &[&str]) {}
    fn begin_check(&mut self, _l: &[&str]) {}
    fn query_done(&mut self, _l: &[&str]) {}
    fn resolve_process(&mut self, _l: &[&str]) {}
    fn resolve_lit(&mut self, _l: &[&str]) {}
    fn conflict(&mut self, _l: &[&str]) {}

    fn main_parse_loop(&mut self, filename: &str, qvar_re_1: Regex, qvar_re_2: Regex) {
        if let Ok(lines) = read_lines(filename) {
            for (line_no, line) in lines.enumerate() {
                if !self.should_continue() {
                    println!("Interrupted");
                    break;
                }
                let l0 = line.unwrap_or_else(|_| panic!("Error reading line {}", line_no));
                let l: Vec<&str> = l0.split(' ').collect();
                match l[0] {
                    // match the line case
                    "[tool-version]" => {
                        self.version_info(&l);
                    }
                    "[mk-quant]" | "[mk-lambda]" => {
                        self.mk_quant(&l);
                    }
                    "[mk-var]" => {
                        self.mk_var(&l);
                    }
                    "[mk-proof]" | "[mk-app]" => {
                        self.mk_proof_app(&l);
                    }
                    "[attach-meaning]" => {
                        self.attach_meaning(&l);
                    }
                    "[attach-var-names]" => {
                        self.attach_vars(&l, &qvar_re_1, &l0, &qvar_re_2);
                    }
                    "[attach-enode]" => {
                        self.attach_enode(&l);
                    }
                    "[eq-expl]" => {
                        self.eq_expl(&l);
                    }
                    "[new-match]" => {
                        self.new_match(&l, line_no);
                    }
                    "[inst-discovered]" => {
                        self.inst_discovered(&l, line_no, &l0);
                    }
                    "[instance]" => {
                        self.instance(&l, line_no);
                    }
                    "[end-of-instance]" => {
                        self.end_of_instance();
                    }
                    "[decide-and-or]" => {
                        self.decide_and_or(&l);
                    }
                    "[decide]" => {
                        self.decide(&l);
                    }
                    "[assign]" => {
                        self.assign(&l);
                    }
                    "[push]" => {
                        self.push(&l);
                    }
                    "[pop]" => {
                        self.pop(&l);
                    }
                    "[begin-check]" => {
                        self.begin_check(&l);
                    }
                    "[query-done]" => {
                        self.query_done(&l);
                    }
                    "[eof]" => {
                        break;
                    }
                    "[resolve-process]" => {
                        self.resolve_process(&l);
                    }
                    "[resolve-lit]" => {
                        self.resolve_lit(&l);
                    }
                    "[conflict]" => {
                        self.conflict(&l);
                    }
                    _ => println!("Unknown line case: {}", l0),
                }
            }
        } else {
            panic!("Failed reading lines")
        }
    }
    
    fn read_and_parse_file(&mut self, filename: &str, settings: &Settings) -> Result<(), String> {
        let qvar_re_1 = Regex::new(QVAR_REGEX_STR_1).unwrap();
        let qvar_re_2 = Regex::new(QVAR_REGEX_STR_2).unwrap();

        let now = Instant::now();

        self.main_parse_loop(filename, qvar_re_1, qvar_re_2);

        let elapsed_time = now.elapsed();
        println!(
            "Finished parsing after {} seconds",
            elapsed_time.as_secs_f32()
        );
        self.save_output_to_files(settings, &now);
        let render_engine = crate::render::GraphVizRender;
        render_engine.make_svg(OUT_FILE_DOT, OUT_FILE_SVG);
        crate::render::add_link_to_svg(OUT_FILE_SVG, OUT_FILE_SVG_2);
        println!(
            "Finished render sequence after {} seconds",
            now.elapsed().as_secs_f32()
        );

        let elapsed_time = now.elapsed();
        println!("Done, run took {} seconds.", elapsed_time.as_secs_f32());

        Ok(())
    }
    fn save_output_to_files(&mut self, settings: &Settings, now: &Instant);
    // fn get_term(&self, id: &str) -> Term;
    // fn get_quantifiers(&self, id: &str) -> Quantifier;
    // fn get_instantiations(&self, line_no: &usize) -> Instantiation;
}

type Z3Fingerprint = u64;

#[derive(Default)]
struct VersionInfo {
    solver: String,
    version: String,
}

pub struct Z3Parser1 {
    terms: TwoDMap<Term>,             // [namespace => [ID number => Term]]
    quantifiers: TwoDMap<Quantifier>, // [namespace => [ID number => Quantifier]]
    matches: HashMap<Z3Fingerprint, Instantiation>, // [match line number => Instantiation]
    instantiations: BTreeMap<usize, Instantiation>, // [line number => Instantiation]
    inst_stack: Vec<(usize, Z3Fingerprint)>, // [(line_no, fingerprint)]
    temp_dependencies: BTreeMap<usize, Vec<Dependency>>, // [match line number => Vec<Dependency>]
    eq_expls: BTreeMap<String, EqualityExpl>, // [ID => EqualityExpl from ID]
    fingerprints: BTreeMap<usize, Z3Fingerprint>, // [match_line_number => fingerprint]
    dependencies: Vec<Dependency>,
    version_info: VersionInfo,
    pub continue_parsing: Arc<Mutex<bool>>, // continue parsing or not?
}

impl LogParser for Z3Parser1 {
    fn read_and_parse_file(
        &mut self,
        filename: &str,
        settings: &Settings,
    ) -> Result<(String,), String> {
        // let mut parser_data = Z3Parser1::default();

        // let mut terms= TwoDMap(HashMap::new()); // HashMap of HashMap
        // // let mut main_terms = HashMap::new();
        // let mut quantifiers = TwoDMap(HashMap::new());
        // //let mut main_quantifiers = HashMap::new();
        // let mut matches = BTreeMap::new();  // fingerprint: instantiation
        // let mut instantiations = BTreeMap::new();   // line no: instantiation
        // let mut inst_stack = vec![];

        // let mut temp_dependencies: BTreeMap<usize, Vec<Dependency>> = BTreeMap::new();
        // let mut eq_expls = BTreeMap::new(); // ID (first term) : full EqualityExpl
        // let mut fingerprints = BTreeMap::new();
        // let mut dependencies = vec![];
        //let mut term_reuse_counter: HashMap<String, usize> = HashMap::new(); // ID (first term) : current reuse

        let qvar_re_1 = Regex::new(QVAR_REGEX_STR_1).unwrap();
        let qvar_re_2 = Regex::new(QVAR_REGEX_STR_2).unwrap();

        let now = Instant::now();

        self.main_parse_loop(filename, qvar_re_1, qvar_re_2);

        let elapsed_time = now.elapsed();
        println!(
            "Finished parsing after {} seconds",
            elapsed_time.as_secs_f32()
        );
        self.save_output_to_files(settings, &now);
        let render_engine = crate::render::GraphVizRender;
        let result = render_engine.make_svg(OUT_FILE_DOT, OUT_FILE_SVG);
        crate::render::add_link_to_svg(OUT_FILE_SVG, OUT_FILE_SVG_2);
        println!(
            "Finished render sequence after {} seconds",
            now.elapsed().as_secs_f32()
        );

        let elapsed_time = now.elapsed();
        println!("Done, run took {} seconds.", elapsed_time.as_secs_f32());

        Ok((result,))
    }

    fn should_continue(&self) -> bool {
        !self.inst_stack.is_empty() || match self.continue_parsing.lock() {
            Ok(guard) => *guard,
            Err(_poisoned) => false,    // if poisoned, assume trying to stop
        }
    }

    fn get_continue_mutex(&self) -> Arc<Mutex<bool>> {
        Arc::clone(&self.continue_parsing)
    }
}

impl Z3LogParser for Z3Parser1 {
    fn version_info(&mut self, l: &[&str]) {
        self.version_info = VersionInfo {
            solver: l[1].to_string(),
            version: l[2].to_string(),
        };
        println!(
            "{} {}",
            &self.version_info.solver, &self.version_info.version
        );
    }

    fn end_of_instance(&mut self) {
        let (inst_line_no, _fingerprint) = self.inst_stack.pop().unwrap();
        // println!("{} {}", inst_line_no, fingerprint);
        let inst = self.instantiations.get_mut(&inst_line_no).unwrap();
        let deps = self.temp_dependencies.get_mut(&inst.match_line_no).unwrap();
        deps.iter_mut().for_each(|dep| {
            dep.to = inst_line_no;
            dep.quant = inst.quant_id.clone();
        });
        self.dependencies.append(deps);
    }

    fn instance(&mut self, l: &[&str], line_no: usize) {
        let fingerprint = u64::from_str_radix(l[1], 16).expect("Should be valid hex string");
        self.fingerprints.insert(line_no + 1, fingerprint);
        // BAD ASSUMPTION (or ensure all instantiations' fingerprints are accounted for)
        let instant = self
            .matches
            .get_mut(&fingerprint)
            .expect("Fingerprint should be in instantiations");

        instant.resulting_term = String::from(l[2]);
        if let Some(z3_gen) = l.get(4) {
            instant.z3_gen = z3_gen.parse().expect("Should be number");
        }
        instant.line_no = line_no + 1;
        self.inst_stack.push((line_no + 1, fingerprint));
        if let Some(quantifier) = self.quantifiers.get_mut(&instant.quant_id) {
            quantifier.instances.push(instant.line_no);
            quantifier.cost += 1.0;
        }
        self.instantiations.insert(line_no + 1, instant.clone());
    }

    fn inst_discovered(&mut self, l: &[&str], line_no: usize, l0: &str) {
        let method = l[1];
        let fingerprint = u64::from_str_radix(l[2], 16).expect("Should be valid hex string");
        let mut blamed_terms = vec![];
        let mut bound_terms = vec![];
        let mut dep_instantiations = vec![];
        let mut name = String::new();
        self.temp_dependencies.insert(line_no + 1, vec![]);
        if method == "theory-solving" {
            name = l[3].to_string();

            blamed_terms = if l.len() >= 5 {
                l[5..]
                    .iter()
                    .map(|t| BlamedTermItem::Single(t.to_string()))
                    .collect()
            } else {
                vec![]
            };
            for term in &blamed_terms {
                if let BlamedTermItem::Single(term) = term {
                    Z3Parser1::add_dependency(
                        &mut self.terms,
                        term,
                        &mut self.instantiations,
                        &mut self.temp_dependencies,
                        &mut dep_instantiations,
                        DepType::Term,
                        line_no + 1,
                    );
                }
            }
        } else if method == "MBQI" {
            name = String::from("MBQI");
            bound_terms = l[3..].iter().map(|t| t.to_string()).collect();
        } else {
            println!("Unknown line case: {}", l0);
        }
        if !self.quantifiers.0.contains_key(&name) {
            let q = Quantifier {
                num_vars: 0,
                name: name.clone(),
                term: String::from(""),
                instances: vec![],
                cost: 0.0,
                vars: vec![],
                vars_set: false,
            };
            self.quantifiers.insert(&name, q);
        }
        Z3Parser1::add_blank_dependency_if_needed(
            &mut dep_instantiations,
            &name,
            &mut self.temp_dependencies,
            line_no + 1,
        );
        let instant = Instantiation {
            line_no: line_no + 1,
            match_line_no: line_no + 1,
            fingerprint,
            resulting_term: String::from("N/A"),
            z3_gen: 0,
            cost: 1.0,
            quant_id: name,
            pattern_id: String::from("N/A"),
            yields_terms: vec![],
            bound_terms,
            blamed_terms,
            equality_expls: vec![],
            dep_instantiations,
        };

        self.matches.insert(fingerprint, instant);
    }

    fn new_match(&mut self, l: &[&str], line_no: usize) {
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
        let mut dep_instantiations = vec![];
        self.temp_dependencies.insert(line_no + 1, vec![]);
        for (i, word) in l[semicolon_index + 1..].iter().enumerate() {
            if let Some(first_term) = word.strip_prefix('(') {
                // assumes that if we see "(#A", the next word in the split is "#B)"
                let next_word = l[semicolon_index + i + 2];
                let second_term = next_word.strip_suffix(')').unwrap();
                if first_term != second_term {
                    let eq = self.eq_expls.get(first_term).unwrap();
                    equality_expls.push(first_term.to_string());
                    use crate::items::EqualityExpl::*;
                    match eq {
                        Root { .. } => {}
                        Literal { eq: from, .. } => {
                            Z3Parser1::add_dependency(
                                &mut self.terms,
                                from,
                                &mut self.instantiations,
                                &mut self.temp_dependencies,
                                &mut dep_instantiations,
                                DepType::Equality,
                                line_no + 1,
                            );
                        }
                        Congruence { .. } => {}
                        Theory { .. } => {}
                        Axiom { .. } => {}
                        Unknown { .. } => {}
                    }
                }
                blamed_terms.push(BlamedTermItem::Pair(
                    first_term.to_string(),
                    next_word[..next_word.len() - 1].to_string(),
                ));
            } else if !word.ends_with(')') {
                Z3Parser1::add_dependency(
                    &mut self.terms,
                    word,
                    &mut self.instantiations,
                    &mut self.temp_dependencies,
                    &mut dep_instantiations,
                    DepType::Term,
                    line_no + 1,
                );

                blamed_terms.push(BlamedTermItem::Single(word.to_string()));
            }
        }
        Z3Parser1::add_blank_dependency_if_needed(
            &mut dep_instantiations,
            quant_id,
            &mut self.temp_dependencies,
            line_no + 1,
        );
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
        self.matches.insert(fingerprint, instant);
    }

    fn eq_expl(&mut self, l: &[&str]) {
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
                    id,
                    arg_eqs: terms,
                    to,
                }
                // For each
            }
            "th" => Theory {
                id,
                theory: l[3].to_string(),
                term: l[5].to_string(),
            },
            "ax" => Axiom {
                id,
                term: l[4].to_string(),
            }, // format #A ax ; #B
            _ => Unknown {
                id,
                term: l[4].to_string(),
            },
        };
        self.eq_expls.insert(id_, eq_expl);
    }

    fn attach_enode(&mut self, l: &[&str]) {
        let term = l[1];
        if !self.inst_stack.is_empty() {
            let (inst_line_no, _) = self.inst_stack.last().unwrap();
            let inst: &mut Instantiation = self.instantiations.get_mut(inst_line_no).unwrap();
            inst.yields_terms.push(term.to_string());
            self.terms.get_mut(term).unwrap().resp_inst_line_no = Some(*inst_line_no);
        }
    }

    fn attach_vars(&mut self, l: &[&str], qvar_re_1: &Regex, l0: &str, qvar_re_2: &Regex) {
        let q = self
            .quantifiers
            .get_mut(l[1])
            .expect(&(String::from("No quantifier with ID") + l[1]));
        let mut vars = vec![];
        for (i, caps) in qvar_re_1.captures_iter(l0).enumerate() {
            let sort = match caps.name("sort") {
                Some(m) => m.as_str().to_string(),
                None => String::new(),
            };
            let name = String::from("qvar_") + &i.to_string();
            vars.push((name, sort));
        }
        for (i, caps) in qvar_re_2.captures_iter(l0).enumerate() {
            let name = match caps.name("name") {
                Some(m) => m.as_str().to_string(),
                None => String::from("qvar_") + &i.to_string(),
            };
            let sort = match caps.name("sort") {
                Some(m) => m.as_str().to_string(),
                None => String::new(),
            };
            vars.push((name, sort));
        }
        q.vars = vars;
        q.vars_set = true;
    }

    fn attach_meaning(&mut self, l: &[&str]) {
        let _full_id = parse_id(l[1]);
        let t = self
            .terms
            .get_mut(l[1])
            .expect(&(String::from("No term with ID ") + l[1]));
        let theory = l[2];
        t.theory = theory.to_string();
        let mut name = l[3].to_string();
        if l[3] == "(-" {
            name += l[4];
        }
        t.name = name;
    }

    fn mk_proof_app(&mut self, l: &[&str]) {
        // TODO: add rewrite, monotonicity cases
        let children = if let Some(vector) = l.get(3..) {
            vector.iter().map(|x| x.to_string()).collect()
        } else {
            vec![]
        };
        let mut child_texts = vec![];
        for child in &children[..] {
            let c_term: &mut Term = self.terms.get_mut(child).unwrap();
            c_term.dep_term_ids.push(l[1].to_string());
            let text = c_term.name.clone()
                + "["
                + &c_term.id.to_string()
                + "]"
                + if !c_term.child_ids.is_empty() {
                    "(...)"
                } else {
                    ""
                };
            child_texts.push(text);
        }
        let full_id = parse_id(l[1]);
        let name = l[2].to_string();
        let mut text = name.clone() + "[" + l[1] + "]";
        if !child_texts.is_empty() {
            text += "(";
            text += &child_texts.join(", ");
            text += ")";
        }
        let term = Term {
            kind: name.clone(),
            id: full_id.1,
            name: name.clone(),
            theory: String::new(),
            child_ids: children,
            dep_term_ids: vec![],
            resp_inst_line_no: None,
            text,
        };
        self.terms.insert(l[1], term);
    }

    fn mk_var(&mut self, l: &[&str]) {
        let full_id = parse_id(l[1]);
        let name = "qvar_".to_string() + l[2];
        let term = Term {
            kind: name.clone(),
            id: full_id.1,
            name: name.clone(),
            theory: String::new(),
            child_ids: vec![],
            dep_term_ids: vec![],
            resp_inst_line_no: None,
            text: name,
        };
        self.terms.insert(l[1], term);
    }

    fn mk_quant(&mut self, l: &[&str]) {
        let full_id = parse_id(l[1]);
        let name = l[2].to_string();
        let num_vars = l[3].parse().expect("l[3] was not an integer");
        let children: Vec<String> = l[4..].iter().map(|t| String::from(*t)).collect();
        assert!(!children.is_empty());
        let mut text = String::from("FORALL[") + l[1] + "]";
        text += "(";
        let mut child_texts = vec![];
        for child in &children[..] {
            let c_term: &mut Term = self.terms.get_mut(child).unwrap();
            c_term.dep_term_ids.push(l[1].to_string());
            child_texts.push(c_term.name.clone() + "[" + &c_term.id.to_string() + "]");
        }
        text += &child_texts.join(", ");
        text += ")";
        let term = make_term(full_id, name, children, text);
        // if full_id.0.is_empty() {
        //     main_terms.insert(full_id.1.parse::<usize>(), term.clone());
        // }
        self.terms.insert(l[1], term);
        let qvars = (0..num_vars)
            .map(|n| (String::from("qvar_") + &n.to_string(), String::new()))
            .collect();
        //println!("{:?}", term);
        let q = Quantifier {
            num_vars,
            name: l[2].to_string(),
            term: l[1].to_string(),
            instances: vec![],
            cost: 0.0,
            vars: qvars,
            vars_set: false,
        };
        //println!("{:?}", q);
        self.quantifiers.insert(l[1], q);
    }

    fn save_output_to_files(&mut self, settings: &Settings, now: &Instant) {
        let terms_main = self.terms.0.get("").unwrap();
        if settings.enable_io {
            let mut file = open_file_truncate(OUT_FILE_TERMS);
            let mut file2 = open_file_truncate(OUT_FILE_PRETTY_TERM);
            // let mut term_print = String::new();
            // let mut term_pretty_print = String::new();
            for i in 0..terms_main.len() {
                let t = terms_main.get(&(i + 1)).unwrap();
                if settings.verbose {
                    t.print();
                }
                if settings.enable_io {
                    write(&mut file, t);
                    pretty_write(&mut file2, &format!("{}\n", &t.text));
                }
            }
            file.flush().unwrap();
            file2.flush().unwrap();
        }

        println!(
            "Finished printing terms ({}) after {} seconds",
            terms_main.len(),
            now.elapsed().as_secs_f32()
        );

        self.update_costs();

        println!(
            "Finished cost after {} seconds",
            now.elapsed().as_secs_f32()
        );

        Z3Parser1::output_to_file(OUT_FILE_INST, &self.instantiations, |_| (), settings);
        println!(
            "Finished printing instantiations ({}) after {} seconds",
            &self.instantiations.len(),
            now.elapsed().as_secs_f32()
        );

        let insts_sorted = Z3Parser1::filter_instantiations_by_cost(&self.instantiations, 250);
        // sorted by cost
        Z3Parser1::output_to_file_vec("out/inst_by_cost.txt", &insts_sorted, |_| (), settings);
        println!(
            "Finished printing cost-sorted instantiations ({}) after {} seconds",
            &self.instantiations.len(),
            now.elapsed().as_secs_f32()
        );

        let quantifiers_main = self.quantifiers.0.get("").unwrap();
        if settings.enable_io {
            let mut file = open_file_truncate(OUT_FILE_QUANT);
            let mut file2 = open_file_truncate(OUT_FILE_PRETTY_QUANT);
            for q in quantifiers_main.values() {
                if settings.verbose {
                    q.print();
                }
                write(&mut file, q);
                pretty_write(&mut file2, &(q.pretty_text(&self.terms) + "\n"));
            }
            file.flush().unwrap();
            file2.flush().unwrap();
        }
        println!(
            "Finished printing quants ({}) after {} seconds",
            quantifiers_main.len(),
            now.elapsed().as_secs_f32()
        );

        Z3Parser1::output_to_file_vec(OUT_FILE_DEP, &self.dependencies, |_| (), settings);
        println!(
            "Finished printing deps ({}) after {} seconds",
            self.dependencies.len(),
            now.elapsed().as_secs_f32()
        );

        let sorted_deps = Self::filter_dependencies_by_cost(&insts_sorted, &self.dependencies);
        Z3Parser1::output_to_file_vec("out/dep_costs_sorted.txt", &sorted_deps, |_| (), settings);
        println!(
            "Finished printing cost-sorted deps ({}) after {} seconds",
            self.dependencies.len(),
            now.elapsed().as_secs_f32()
        );

        Z3Parser1::output_to_file(OUT_FILE_EQ, &self.eq_expls, |_| (), settings);
        println!(
            "Finished printing eq-expls ({}) after {} seconds",
            self.eq_expls.len(),
            now.elapsed().as_secs_f32()
        );

        output_dot_to_file(OUT_FILE_DOT, OUT_FILE_CSS, &sorted_deps); // sorted option
        println!(
            "Finished dot sequence after {} seconds",
            now.elapsed().as_secs_f32()
        );
    }
}

fn make_term(full_id: (String, usize), name: String, children: Vec<String>, text: String) -> Term {
    Term {
        kind: String::from("FORALL"),
        id: full_id.1,
        name: name.clone(),
        theory: String::new(),
        child_ids: children,
        dep_term_ids: vec![],
        resp_inst_line_no: None,
        text,
    }
}

impl Default for Z3Parser1 {
    fn default() -> Z3Parser1 {
        Z3Parser1 {
            terms: TwoDMap(HashMap::new()),
            quantifiers: TwoDMap(HashMap::new()),
            matches: HashMap::new(),
            instantiations: BTreeMap::new(),
            inst_stack: vec![],
            temp_dependencies: BTreeMap::new(),
            eq_expls: BTreeMap::new(),
            fingerprints: BTreeMap::new(),
            dependencies: vec![],
            version_info: VersionInfo::default(),
            continue_parsing: Arc::new(Mutex::new(true)),
        }
    }
}

impl Z3Parser1 {
    fn update_costs(&mut self) {
        // propagate cost data
        // not quite right; doesn't count theory instantiations, some refactoring of Instantiations map may be needed (e.g. changing key system)
        let mut cost_data: Vec<(Vec<usize>, usize)> = self
            .instantiations
            .values()
            .map(|inst| (inst.dep_instantiations.clone(), inst.line_no))
            .collect();
        // grab dep inst line #s and own line #s
        cost_data.sort_unstable_by(|a, b| a.1.cmp(&b.1));
        // sort by their line numbers
        cost_data.reverse();
        // start from latest instantiations and go backwards
        for (dep_insts, line_no) in cost_data {
            let num_deps = dep_insts.len() as f32;
            let cost = self.instantiations.get(&line_no).unwrap().cost / num_deps;
            for dep_inst_line in &dep_insts {
                // let dep_inst_fingerprint = fingerprints.get(&dep_inst_line).unwrap();
                let dep_inst = self
                    .instantiations
                    .get_mut(dep_inst_line)
                    .unwrap_or_else(|| panic!("{} {:?}", dep_inst_line, &dep_insts));
                let dep_inst_quant = self
                    .quantifiers
                    .get_mut(&dep_inst.quant_id)
                    .unwrap_or_else(|| panic!("{}", &dep_inst.quant_id));
                dep_inst.cost += cost;
                dep_inst_quant.cost += cost;
            }
        }
    }

    /// Add a (partial) blank instantiation dependency. Used to keep track of instantiations dependent on no others.
    fn add_blank_dependency_if_needed(
        dep_insts: &mut Vec<usize>,
        quant_id: &str,
        temp_deps: &mut BTreeMap<usize, Vec<Dependency>>,
        match_line: usize,
    ) {
        if dep_insts.is_empty() {
            let dep = Dependency {
                from: 0,
                to: 0,
                blamed: String::new(),
                dep_type: DepType::None,
                quant: quant_id.to_string(),
            };
            temp_deps.get_mut(&match_line).unwrap().push(dep);
        }
    }

    /// Add a (partial) instantiation dependency from the quantifier that instantiated the term with ID `from`, if there is one.
    ///
    /// Since this is done during a `"[match]"` or `"[inst-discovered]"` line, the instantiation's actual line number is not available yet.
    /// Additionally, some matches are not instantiated at all.
    fn add_dependency(
        terms: &mut TwoDMap<Term>,
        from_term: &str,
        instantiations: &mut BTreeMap<usize, Instantiation>,
        temp_deps: &mut BTreeMap<usize, Vec<Dependency>>,
        dep_insts: &mut Vec<usize>,
        dep_type: DepType,
        match_line: usize,
    ) {
        let eq_term = terms.get_mut(from_term).unwrap();
        if let Some(inst_line_no) = eq_term.resp_inst_line_no {
            let inst = instantiations
                .get_mut(&inst_line_no)
                .unwrap_or_else(|| panic!("{}", inst_line_no));
            let dep = Dependency {
                from: inst.line_no,
                to: 0,
                blamed: from_term.to_string(),
                dep_type,
                quant: String::new(),
            };
            // temp_deps.insert(inst.line_no, dep);
            temp_deps.get_mut(&match_line).unwrap().push(dep);
            dep_insts.push(inst.line_no);
        }
    }

    /// Output a BTreeMap's values to file.
    fn output_to_file<K, V, F>(
        filename: &str,
        map: &BTreeMap<K, V>,
        do_stuff: F,
        settings: &Settings,
    ) where
        F: Fn(&V),
        V: Print,
    {
        if settings.enable_io {
            let mut file = open_file_truncate(filename);
            for item in map.values() {
                do_stuff(item);
                if settings.verbose {
                    item.print();
                }
                write(&mut file, item);
            }
            file.flush().unwrap();
        }
    }

    /// Output a Vec's values to file.
    fn output_to_file_vec<V, F>(filename: &str, vec: &Vec<V>, do_stuff: F, settings: &Settings)
    where
        F: Fn(&V),
        V: Print,
    {
        if settings.enable_io {
            let mut file = open_file_truncate(filename);
            for item in vec {
                do_stuff(item);
                if settings.verbose {
                    item.print();
                }
                if settings.enable_io {
                    write(&mut file, item);
                }
            }
            file.flush().unwrap();
        }
    }

    /// Returns a vector of instantiations, sorted by cost
    fn sort_instantiations_by_cost(
        instantiations: &BTreeMap<usize, Instantiation>,
    ) -> Vec<Instantiation> {
        let mut inst_vec = instantiations
            .values()
            .cloned()
            .collect::<Vec<Instantiation>>();
        inst_vec.sort_unstable_by(|inst1, inst2| inst2.cost.partial_cmp(&inst1.cost).unwrap());
        inst_vec
    }

    /// Show only the `max_elements` most costly instantiations
    fn filter_instantiations_by_cost(
        instantiations: &BTreeMap<usize, Instantiation>,
        max_elements: usize,
    ) -> Vec<Instantiation> {
        Z3Parser1::sort_instantiations_by_cost(instantiations)
            [..std::cmp::min(instantiations.len(), max_elements)]
            .to_vec()
    }

    fn filter_dependencies_by_cost(
        instantiations: &[Instantiation],
        dependencies: &[Dependency],
    ) -> Vec<Dependency> {
        let inst_set =
            HashSet::<_>::from_iter(instantiations.iter().map(|inst| inst.line_no));
        dependencies
            .iter()
            .filter(|dep| inst_set.contains(&dep.to))
            .cloned()
            .collect::<Vec<Dependency>>()
    }
}

// #[derive(Debug, Hash, Clone)]
// pub struct Ident {
//     namespace: String,
//     num: usize,
//     reuse_num: usize
// }

/*
if let Ok(lines) = read_lines(filename) {
            for (line_no, line) in lines.enumerate() {
                let l0 = line.unwrap_or_else(|_| panic!("Error reading line {}", line_no));
                let l: Vec<&str> = l0.split(' ').collect();
                match l[0] {    // match the line case
                    "[tool-version]" => {
                        version_info = VersionInfo { solver: l[1].to_string(), version: l[2].to_string()};
                        println!("{} {}", &version_info.solver, &version_info.version);
                    },
                    "[mk-quant]" | "[mk-lambda]" => {
                    },

                    "[mk-var]" => {
                    },
                    "[mk-proof]" | "[mk-app]" => {
                    },
                    "[attach-meaning]" => {
                    },
                    "[attach-var-names]" => {
                    },
                    "[attach-enode]" => {
                    },
                    "[eq-expl]" => {
                    },
                    "[new-match]" =>  {
                    },
                    "[inst-discovered]" =>  {
                    },
                    "[instance]" =>  {
                    },
                    "[end-of-instance]" =>  {
                    },
                    "[decide-and-or]" =>  {},
                    "[decide]" =>  {},
                    "[assign]" =>  {},
                    "[push]" =>  {},
                    "[pop]" =>  {},
                    "[begin-check]" =>  {},
                    "[query-done]" =>  {},
                    "[eof]" =>  { break; },
                    "[resolve-process]" =>  {},
                    "[resolve-lit]" =>  {},
                    "[conflict]" =>  {},
                    _ => println!("Unknown line case: {}", l0)
                }
            }
        } else {
            panic!("Failed reading lines")
        }
*/
