mod colors {
    use random_color::{self, RandomColor};
    /// Generate `n` distinct colors; each color represents instantiations from the same quantifier.
    /// NOTE: may change to same quantifier + pattern/trigger, since quantifiers can have multiple patterns.
    #[allow(dead_code)] 
    pub fn make_rgb_strings(n: usize) -> Vec<String> {
        let mut result = vec![];
        for _ in 0..n {
            result.push(RandomColor::new().to_rgb_string());
        }
        result
    }

    pub fn make_hsl_strings(n: usize) -> Vec<String> {
        const DEFAULT_SAT: f64 = 100.0;
        const DEFAULT_LUM: f64 = 70.0;
        generate_colors(n).iter().map(|hue| 
            format!("hsl({}, {}%, {}%)", hue, DEFAULT_SAT, DEFAULT_LUM)).collect::<Vec<String>>()
    }

    /// Generate colors from golden ratio; essentially pseudorandomly with fixed seed
    /// Described here: https://martin.ankerl.com/2009/12/09/how-to-create-random-colors-programmatically/
    fn generate_colors(n: usize) -> Vec<f64> {
        const GOLDEN_RATIO_RECIP: f64 = 0.618033988749895;
        let mut hue = 0.0;
        let mut result = vec![hue];
        for _ in 1..n {
            hue += GOLDEN_RATIO_RECIP;
            hue = hue.fract() * 360.0;
            result.push(hue);
        }
        result
    }
}

pub mod make_css {
    use std::collections::HashSet;

    use super::colors::*;

    const NODE_HOVER_RULE: &str = ".node:hover {\n\topacity: 0.6\n}\n\n";
    const EDGE_HOVER_RULE: &str = ".edge:hover * {\n\topacity: 0.4\n}\n\n";
    pub fn css_string(quants: &HashSet<String>) -> String {
        let mut result = String::from(NODE_HOVER_RULE) + EDGE_HOVER_RULE;
        let rgb_strings = make_hsl_strings(quants.len());
        for (i, quant) in quants.iter().enumerate() {
            result += &format!(".{} > ellipse {{\n\tfill: {} !important\n}}\n\n", quant, rgb_strings[i]);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use regex::Regex;
    use super::colors::*;

    const RGB_REGEX: &str = r"#[0-9A-F]{6}";

    #[test]
    fn test_make_one_color() {
        let rgb_regex = Regex::new(RGB_REGEX).unwrap();
        let colors = make_rgb_strings(1);
        assert_eq!(colors.len(), 1);
        assert!(rgb_regex.is_match(&colors[0]));
    }

    #[test]
    fn test_make_some_colors() {
        let rgb_regex = Regex::new(RGB_REGEX).unwrap();
        let colors = make_rgb_strings(5);
        assert_eq!(colors.len(), 5);
        let distinct_colors: HashSet<String> = HashSet::from_iter(colors);
        assert_eq!(distinct_colors.len(), 5);
        for color in distinct_colors {
            assert!(rgb_regex.is_match(&color));
        }
    }
}