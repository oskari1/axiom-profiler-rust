use std::io::Write;

pub trait RenderSVG {
    fn make_svg(&self, input_file: &str, output_file: &str) -> String;
}

pub struct GraphVizRender;
impl RenderSVG for GraphVizRender {
    fn make_svg(&self, input_file: &str, output_file: &str) -> String {
        use std::process::Command;
        let output = Command::new("dot")
                    .arg("-Tsvg")
                    .arg(input_file)
                    .arg("|")
                    .arg("tee")
                    .arg(output_file)
                    .output()
                    .expect("failed to execute process")
                    .stdout;
        String::from_utf8_lossy(&output).to_string()
    }
}

pub fn add_link_to_svg(input_file: &str, output_file: &str) {
    use crate::file_io;
    const INSERT_LINE_NO: usize = 9;
    const INSERT_LINE: &str = "<link xmlns=\"http://www.w3.org/1999/xhtml\" rel=\"stylesheet\" href=\"styles.css\" type=\"text/css\" />\n";
    let file_lines = file_io::read_lines(input_file).unwrap();
    // open new file
    let mut out_file = file_io::open_file_truncate(output_file);
    for (line_no, line0) in file_lines.enumerate() {
        // if correct line number
        if line_no == INSERT_LINE_NO {
            out_file.write_all(INSERT_LINE.as_bytes()).expect("failed to write line to new SVG");
        }
        // write constant line
        // write line
        let line = line0.unwrap() + "\n";
        out_file.write_all(line.as_bytes()).expect("failed to write line to new SVG");

    }
}