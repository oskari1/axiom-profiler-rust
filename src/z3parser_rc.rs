use std::cell::RefCell;
use std::rc::Rc;
use crate::*;
#[derive(Default)]
pub struct Z3ParserRc {
    terms: RcBTreeMap<String, Term>,
    quantifiers: RcBTreeMap<String, Quantifier>,
    matches: RcHashMap<Z3Fingerprint, Instantiation>,
    instantiations: RcBTreeMap<usize, Instantiation>,
    inst_stack: Vec<(usize, Rc<RefCell<Instantiation>>)>,
    temp_dependencies: BTreeMap<usize, Vec<Dependency>>,
    eq_expls: BTreeMap<String, EqualityExpl>,
    fingerprints: BTreeMap<usize, Z3Fingerprint>,
    dependencies: Vec<Dependency>,
    reuses: HashMap<String, usize>,
    version_info: VersionInfo,
}

impl LogParser for Z3ParserRc {
    fn read_and_parse_file(
        &mut self,
        filename: &str,
        settings: &Settings,
    ) -> Result<(String,), String> {
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
        todo!()
    }

    fn get_continue_mutex(&self) -> Arc<Mutex<bool>> {
        todo!()
    }
}

impl Z3LogParser for Z3ParserRc {
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

    fn mk_quant(&mut self, _l: &[&str]) {
        todo!()
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
        self.terms.insert(l[1].to_string(), get_rc_refcell(term));
    }

    fn mk_proof_app(&mut self, _l: &[&str]) {
        todo!()
    }

    fn attach_meaning(&mut self, _l: &[&str]) {
        todo!()
    }

    fn attach_vars(&mut self, _l: &[&str], _qvar_re_1: &Regex, _l0: &str, _qvar_re_2: &Regex) {
        todo!()
    }

    fn attach_enode(&mut self, _l: &[&str]) {
        todo!()
    }

    fn eq_expl(&mut self, _l: &[&str]) {
        todo!()
    }

    fn new_match(&mut self, _l: &[&str], _line_no: usize) {
        todo!()
    }

    fn inst_discovered(&mut self, _l: &[&str], _line_no: usize, _l0: &str) {
        todo!()
    }

    fn instance(&mut self, _l: &[&str], _line_no: usize) {
        todo!()
    }

    fn end_of_instance(&mut self) {
        todo!()
    }

    fn save_output_to_files(&mut self, _settings: &Settings, _now: &Instant) {
        todo!()
    }

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

    fn main_parse_loop(&mut self, filename: &str, qvar_re_1: Regex, qvar_re_2: Regex) {
        if let Ok(lines) = read_lines(filename) {
            for (line_no, line) in lines.enumerate() {
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
}

impl Z3ParserRc {
    fn get_ident(&self, id: &str) -> Ident {
        let (ns, num) = parse_id(id);
        let mut reuse_num = 1;
        if let Some(n) = self.reuses.get(id) {
            reuse_num = *n;
        }
        Ident {
            namespace: ns,
            num,
            reuse_num,
        }
    }
}
fn get_rc_refcell<T>(item: T) -> Rc<RefCell<T>> {
    Rc::new(RefCell::from(item))
}

/// Get an Ident for a new ID (never seen before, or reused).
/// Updates reuse map
fn get_ident_new(reuse_map: &mut HashMap<String, usize>, id: &str, do_reuses: bool) -> Ident {
    let (namespace, num) = parse_id(id);
    let reuse_num = if !do_reuses {
        0
    } else {
        *reuse_map.entry(id.to_string())
        .and_modify(|n| {*n += 1})
        .or_insert(1_usize) // increment reuse count if exists, otherwise insert 1 (and then get the current reuse count)
    };
    Ident {
        namespace,
        num,
        reuse_num,
    }
}


