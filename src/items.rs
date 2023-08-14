use std::cell::RefCell;
use std::fmt::Debug;
use std::collections::{HashMap, BTreeMap};
use std::rc::Rc;


/// Trait for pretty printing
pub trait Print: Debug {
    fn format(&self) -> String where Self: Debug {
        format!("{:?}\n", self)
    }

    fn print(&self) where Self: Debug {
        print!("{}", self.format());
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Ident {
    pub namespace: String,
    pub num: usize,
    pub reuse_num: usize
}

impl Print for Ident {
    fn format(&self) -> String where Self: Debug {
        format!("{}#{}#{}", self.namespace, self.num, self.reuse_num)
    }
}

#[derive(Debug, Hash, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fingerprint {
    fingerprint: u64,
    reuse_num: usize
}

impl Print for Fingerprint {
    fn format(&self) -> String where Self: Debug {
        format!("({}, {})", self.fingerprint, self.reuse_num)
    }
}

#[derive(Debug, Eq, PartialEq, Default)]
pub struct Term {
    pub kind: String,
    pub id: usize,
    pub name: String,
    pub theory: String,
    pub child_ids: Vec<String>,
    pub dep_term_ids: Vec<String>,
    pub resp_inst_line_no: Option<usize>,

    // pub children: RcVec<Term>,
    // pub dep_terms: RcVec<Term>,
    // pub resp_inst: RcOption<Instantiation>,
    pub text: String
}

impl Print for Term {}
impl Term {
    // TODO: see if this can be made more efficient? Memoize text, topological sort?
    // BUILD text as term is made?
    pub fn pretty_text(&self, map: &TwoDMap<Term>) -> String {
        let child_text: Vec<String> = self.child_ids.iter().map(|c| {
            let term: &Term = map.get(c).unwrap();
            term.pretty_text(map)
        }).collect();
        let text = child_text.join(", ");
        if !text.is_empty() {
            return String::from(&self.name) + "[" + &self.id.to_string() + "]" + "(" + &text + ")";
        }
        String::from(&self.name) + "[" + &self.id.to_string() + "]"
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct RcTerm {
    pub id: Ident,
    pub name: String,
    pub theory: String,
    pub children: Vec<Ident>,
    pub dep_terms: Vec<Ident>,
    pub resp_inst_line_no: Option<usize>,
    pub text: String
}
impl Print for RcTerm {}

#[derive(Debug, PartialEq, Default)]
pub struct Quantifier {
    pub num_vars: usize,
    pub name: String,
    pub term: String,
    pub cost: f32,
    pub instances: Vec<usize>,
    pub vars: Vec<(String, String)>,
    pub vars_set: bool
}

impl Print for Quantifier { 
    fn format(&self) -> String {
        format!("(name: {}[{}], vars: {:?}({}), cost: {}, instances: {} {:?})\n", self.name, self.term, self.vars, self.num_vars, self.cost, self.instances.len(), self.instances)
    }
}
impl Quantifier {
    pub fn pretty_text(&self, map: &TwoDMap<Term>) -> String {
        if &self.term != "N/A" {
            let term = map.get(&self.term).unwrap();
            let mut result = String::from("FORALL ");
            let var_text: Vec<String> = self.vars.iter().map(|(v, s)| format!("{}: {}", v, s)).collect();
            result += &var_text.join(", ");
            result += &format!("({})", &term.pretty_text(map));
            if self.vars_set {
                for i in 0..self.num_vars {
                    result = result.replace(&(String::from("qvar_") + &i.to_string()), &self.vars[i].0);
                }
            }
            return result
        }
        self.name.clone()
    }
}

#[derive(Debug, PartialEq, Default)]
pub struct RcQuantifier {
    pub num_vars: usize,
    pub name: String,
    pub term: Rc<RefCell<Term>>,
    pub cost: f32,
    pub instances: Vec<usize>,
    pub vars: Vec<(String, String)>,
    pub vars_set: bool
}



#[derive(Debug, Clone)]
pub struct Instantiation {
    pub line_no: usize,
    pub match_line_no: usize,
    pub fingerprint: u64,
    pub resulting_term: String,
    pub z3_gen: u32,
    pub cost: f32,
    pub quant_id: String,
    pub pattern_id: String,
    pub yields_terms: Vec<String>,
    pub bound_terms: Vec<String>,
    pub blamed_terms: Vec<BlamedTermItem>,
    pub equality_expls: Vec<String>,
    pub dep_instantiations: Vec<usize>
}

impl Print for Instantiation {
    fn format(&self) -> String {
        format!("(@{}, @{}, {:x}, resulting: {}, gen: {}, cost: {}, Q: {}, pattern: {}, yields: {:?}({}), bound: {:?}, blamed: {:?}, eq: {:?}, dep: {:?})\n",
        self.line_no, self.match_line_no, self.fingerprint, self.resulting_term, self.z3_gen, self.cost, self.quant_id, self.pattern_id, self.yields_terms, 
        self.yields_terms.len(), self.bound_terms, self.blamed_terms, self.equality_expls, self.dep_instantiations)
    }
}

#[derive(Debug, Clone)]
pub enum BlamedTermItem {
    Single(String),
    Pair(String, String)
}

/// Splits an ID string into namespace and ID number
pub fn parse_id(s: &str) -> (String, usize) {
    let split: Vec<&str> = s.split('#').collect(); 
    let n =
    if let Some(s2) = split.get(1) {
        if let Ok(n) = s2.parse::<usize>() {
            n
        } else {
            0
        }
    } else {
        0
    };
    (split[0].to_string(), n)
}

#[derive(Debug)]
pub struct TwoDMap<V>(pub HashMap<String, BTreeMap<usize, V>>);
// NOTE: maybe replace generic with enum for Term, Quant, etc.

impl<T> Default for TwoDMap<T> {
    fn default() -> Self {
        TwoDMap(HashMap::new())
    }
}

impl<V> TwoDMap<V> {
    /// Inserts given term into given HashMap (uses given ID)
    pub fn insert(&mut self, id: &str, item: V) {
        let (ns, num) = parse_id(id);
        match self.0.get_mut(&ns) {
            Some(ns_map) => {
                ns_map.insert(num, item);
            },
            None => {
                let mut ns_map = BTreeMap::new();
                ns_map.insert(num, item);
                self.0.insert(ns, ns_map);
            }
        }
    }

    /// Gets item with given ID as an immutable reference
    pub fn get<'a>(&'a self, id: &str) -> Option<&'a V> {
        let (ns, num) = parse_id(id);
        match self.0.get(&ns) {
            Some(ns_map) => ns_map.get(&num),
            None => None
        }
    }

    /// Gets item with given ID as a mutable reference
    pub fn get_mut<'a>(&'a mut self, id: &str) -> Option<&'a mut V> {
        let (ns, num) = parse_id(id);
        match self.0.get_mut(&ns) {
            Some(ns_map) => ns_map.get_mut(&num),
            None => None
        }
    }

    //fn lazy_print(term: Term) {}
}

// pub fn print_all<T: std::fmt::Debug + Default> (map: &TwoDMap<T>, ns: &str) {
//     if let Some(ns_map) = map.0.get(ns) {
//         let mut list: Vec<&String> = ns_map.keys().collect();
//         list.sort_unstable();
//         // println!("{:?}", list);
//         for i in list {
//             println!("{:?}", ns_map.get(i).expect("None obtained for valid map key?!"));
//         }
//     } else {
//         println!("Namespace {} not in map", ns);
//     }
// }

// #[derive(Debug, Default)]
// pub struct RcHashMap<K, V>(HashMap<K, Rc<RefCell<V>>>);

// #[derive(Debug, Default)]
// pub struct RcBTreeMap<K, V>(BTreeMap<K, Rc<RefCell<V>>>);

// #[derive(Debug, Default, PartialEq, Eq)]
// pub struct RcVec<T>(Vec<Rc<RefCell<T>>>);

// #[derive(Debug, Default, PartialEq, Eq)]
// pub struct RcOption<T>(Option<Rc<RefCell<T>>>);

pub type RcHashMap<K, V> = HashMap<K, Rc<RefCell<V>>>;
pub type RcBTreeMap<K, V> = BTreeMap<K, Rc<RefCell<V>>>;
pub type RcVec<T> = Vec<Rc<RefCell<T>>>;
pub type RcOption<T> = Option<Rc<RefCell<T>>>;


#[derive(Debug, Clone)]
pub enum DepType {
    None,
    Term,
    Equality
}


#[derive(Debug, Clone)]
pub struct Dependency {
    pub from: usize,
    pub to: usize,
    pub blamed: String,
    pub dep_type: DepType,
    pub quant: String,
    // pub cost: f64  // may want to just have single score field
}

impl Print for Dependency {
    fn format(&self) -> String {
        format!("@{} -> @{} ({}, {:?}, {})\n", self.from, self.to, self.blamed, self.dep_type, self.quant)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EqualityExpl {
    Root {
        id: String
    },
    Literal {
        from: String,
        eq: String,
        to: String
    },
    Congruence {
        id: String,
        arg_eqs: Vec<(String, String)>,
        to: String
        // dependent instantiations
    },
    Theory {
        id: String,
        theory: String,
        term: String
    },
    Axiom {
        id: String,
        term: String
    },
    Unknown {
        id: String,
        term: String
    }
}

impl Print for EqualityExpl {
    fn format(&self) -> String {
        // println!("{:?}", self);
        match self {
            EqualityExpl::Root { id } => format!("Root {}\n", id),
            EqualityExpl::Literal { from: id, eq: from, to } => format!("Lit. {}: {}, {}\n", id, from, to),
            EqualityExpl::Congruence { id, arg_eqs: terms, to } => format!("Cong. {}: {:?}, {}\n", id, terms, to),
            EqualityExpl::Theory { id, theory, term } => format!("Theory {}: {} {}\n", id, theory, term),
            EqualityExpl::Axiom { id, term } => format!("Axiom {}: {}\n", id, term),
            EqualityExpl::Unknown { id, term } => format!("Unknown {}: {}\n", id, term),
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::Term;

//     #[test]
//     fn test_print_basic_term() {
        
//     }
// }