use std::fmt;

pub struct Line {
    pub diff: char,
    pub depth: usize,
    pub contents: String,
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let diff = match self.diff {
            '.' => "".to_owned(),
            _ => format!("{} ", self.diff),
        };
        write!(f, "{}{}{}", " ".repeat(self.depth * 2), diff, self.contents)
    }
}

impl Line {
    pub fn new(diff: char, contents: String) -> Line {
        Line {
            diff,
            depth: 0,
            contents,
        }
    }

    pub fn wrap(lines: Vec<Line>, name: &str) -> Vec<Line> {
        let mut result: Vec<Line> = vec![];
        if !lines.is_empty() {
            result.push(Line::new('.', format!("{}{{", name)));
            for mut line in lines {
                line.depth += 1;
                result.push(line);
            }
            result.push(Line::new('.', "}".to_owned()));
        }
        result
    }
}
