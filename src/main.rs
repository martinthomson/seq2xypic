#![deny(clippy::pedantic)]

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fs::File;
use std::io;
use std::ops::RangeInclusive;
use std::path::PathBuf;

#[derive(Default)]
struct Nodes {
    all: Vec<String>,
}

impl Nodes {
    pub fn add(&mut self, n: &str) {
        let n = n.trim();
        if self.all.iter().all(|x| x != n) {
            self.all.push(n.to_owned());
        }
    }
    pub fn len(&self) -> usize {
        self.all.len()
    }
    pub fn index_of(&self, n: &str) -> usize {
        self.all
            .iter()
            .enumerate()
            .find_map(|(i, v)| if v == n { Some(i) } else { None })
            .unwrap()
    }
    pub fn print(&self) {
        fn p(n: &str, delim: &str) {
            println!(r#"  *+[F]{{\txt{{{}}}}} {}"#, n, delim,);
        }
        for n in &self.all[..self.len() - 1] {
            p(n, "&");
        }
        p(&self.all[self.all.len() - 1], r#"\\"#);
    }
}

enum Item {
    Arrow {
        from: String,
        to: String,
        text: String,
    },
    Note {
        from: Option<String>,
        to: Option<String>,
        text: String,
        line: usize,
    },
    Group {
        text: String,
        lines: usize,
    },
}

impl Item {
    pub fn txt(t: &str) -> String {
        t.trim()
            .replace('\\', "\\backslash")
            .replace('_', "\\_")
            .replace('\n', " \\\\\n")
    }

    fn print_arrow(nodes: &Nodes, from: &str, to: &str, text: &str) {
        let start = nodes.index_of(from);
        let end = nodes.index_of(to);
        let (l, c, d) = if start > end {
            (start - end, "l", '_')
        } else {
            (end - start, "r", '^')
        };
        println!(
            r#"    {} \ar[{}]{}{{\txt{{{}}}}} {} \\"#,
            "&".repeat(start),
            c.repeat(l),
            d,
            Self::txt(text),
            "&".repeat(nodes.len() - start - 1)
        );
    }

    #[allow(clippy::needless_range_loop)] // More readable like this.
    fn print_note(
        nodes: &Nodes,
        from: &Option<String>,
        to: &Option<String>,
        text: &str,
        line: usize,
        verticals: &[usize],
    ) -> RangeInclusive<usize> {
        const LINE_HEIGHT: f64 = 1.5;
        let start = from.as_ref().map_or(0, |x| nodes.index_of(&x));
        let end = to
            .as_ref()
            .map_or_else(|| nodes.len() - 1, |x| nodes.index_of(&x));
        match start.cmp(&end) {
            Ordering::Equal => {
                println!(
                    r#"    {} *+[F.:<3pt>]{{\txt{{{}}}}} \ar@{{-}}[{}] {} \\"#,
                    "&".repeat(start),
                    Self::txt(text),
                    "u".repeat(verticals[start]),
                    "&".repeat(nodes.len() - end),
                );
            }
            Ordering::Less => {
                let middle = (end - start) / 2;
                let lines =
                    f64::from(u32::try_from(text.trim().matches('\n').count() + 1).unwrap());
                print!(
                    r#"    {} *+<{}em>{{}} \save [].[{}] *[F.:<3pt>]\frm{{}} \restore"#,
                    "&".repeat(start),
                    LINE_HEIGHT * lines,
                    "r".repeat(end - start),
                );
                for i in start..middle {
                    if i > start {
                        print!(r#"  *+<{}em>{{}}"#, LINE_HEIGHT * lines);
                    }
                    print!(r#" \ar@{{-}}[{}] &"#, "u".repeat(verticals[i]));
                }
                print!(r#" *+\txt{{{}}}"#, Self::txt(text));
                for i in middle..=end {
                    if i > middle {
                        print!(r#"  *+<{}em>{{}}"#, LINE_HEIGHT * lines);
                    }
                    print!(r#" \ar@{{-}}[{}]"#, "u".repeat(verticals[i]));
                    if i < end {
                        print!(" &");
                    } else {
                        println!(r#" {} \\"#, "&".repeat(nodes.len() - end - 1));
                    }
                }
            }
            Ordering::Greater => {
                panic!("unsupported note ordering on line {}", line);
            }
        }
        start..=end
    }

    fn print_group(text: &str, lines: usize, nnodes: usize, verticals0: usize) {
        if lines == 0 {
            println!("% empty group: {}", text);
        } else {
            println!(
                r#"    \save [].[{}] {{\txt{{{}}}}} \restore \ar@{{-}}[{}]"#,
                "r".repeat(nnodes - 1),
                Self::txt(text),
                "u".repeat(verticals0),
            );
            println!(
                r#"      \save [].[{}{}] *+[F-,]\frm{{}} \restore {} \\"#,
                "d".repeat(lines),
                "r".repeat(nnodes - 1),
                "&".repeat(nnodes - 1),
            );
        }
    }

    #[allow(clippy::needless_range_loop)] // The loops are clearer.
    pub fn print(&self, nodes: &Nodes, verticals: &mut Vec<usize>) {
        for i in &mut verticals[..] {
            *i += 1;
        }
        match &self {
            Self::Arrow { from, to, text } => {
                Self::print_arrow(nodes, from, to, text);
            }
            Self::Note {
                from,
                to,
                text,
                line,
            } => {
                let range = Self::print_note(nodes, from, to, text, *line, verticals);
                for i in range {
                    verticals[i] = 0;
                }
            }
            Self::Group { text, lines } => {
                Self::print_group(text, *lines, nodes.len(), verticals[0]);
                verticals[0] = 0;
            }
        }
    }
}

#[derive(Default)]
struct Items {
    title: String,
    options: String,
    label: Option<String>,
    nodes: Nodes,
    all: Vec<Item>,
}

impl Items {
    pub fn arrow(&mut self, a: &str, b: &str, text: String) {
        let a = a.trim();
        let b = b.trim();
        self.nodes.add(a);
        self.nodes.add(b);
        self.all.push(Item::Arrow {
            from: a.to_owned(),
            to: b.to_owned(),
            text,
        });
    }

    pub fn note(&mut self, a: Option<&str>, b: Option<&str>, text: String, line: usize) {
        let from = a.map(|x| x.trim().to_owned());
        let to = b.map(|x| x.trim().to_owned());
        self.all.push(Item::Note {
            from,
            to,
            text,
            line,
        });
    }

    pub fn group(&mut self, text: String) {
        self.all.push(Item::Group { text, lines: 0 })
    }

    pub fn end_group(&mut self) {
        let count = self.all.len();
        for i in (0..count).rev() {
            if let Item::Group { lines, .. } = &mut self.all[i] {
                *lines = count - i - 1;
                break;
            }
        }
    }

    pub fn add_text(&mut self, t: &str) {
        if let Some(Item::Arrow { text, .. } | Item::Note { text, .. } | Item::Group { text, .. }) =
            self.all.last_mut()
        {
            text.push('\n');
            text.push_str(t);
        }
    }

    pub fn print(&self) {
        println!(r#"\begin{{figure}}"#);
        println!(r#"\small"#);
        println!(r#"\[ \xymatrix {} {{"#, self.options);

        let mut verticals = vec![0; self.nodes.len()];

        self.nodes.print();
        for item in &self.all {
            item.print(&self.nodes, &mut verticals);
        }

        for i in &verticals[..verticals.len() - 1] {
            print!(r#" \ar@{{-}}[{}] &"#, "u".repeat(*i + 1));
        }
        println!(
            r#" \ar@{{-}}[{}] \\"#,
            "u".repeat(*verticals.last().unwrap() + 1)
        );
        println!(r#"}} \]"#);
        println!(r#"\caption{{{}}}"#, Item::txt(&self.title));
        if let Some(label) = &self.label {
            println!(r#"\label{{fig:{}}}"#, Item::txt(label));
        }
        println!(r#"\end{{figure}}"#);
    }

    pub fn label(&mut self, label: &str) {
        self.label = Some(label.to_owned())
    }

    pub fn parse(&mut self, r: &mut impl io::BufRead) {
        let mut line = 1;

        loop {
            let mut t = String::new();
            if r.read_line(&mut t).unwrap() == 0 {
                break;
            }
            let s = t.trim();
            if s.starts_with('#') {
                continue;
            }
            if s.is_empty() {
                self.add_text(s);
                continue;
            }
            if let Some((label, text)) = s.split_once(':') {
                let label = label.trim();
                let text = text.trim().to_owned();
                if label == "xypic" {
                    self.options = text;
                } else if label == "title" {
                    self.title = text;
                } else if label == "note" {
                    self.note(None, None, text, line);
                } else if let Some(x) = label.strip_prefix("note ") {
                    if let Some((a, b)) = x.split_once(',') {
                        self.note(Some(a), Some(b), text, line);
                    } else {
                        self.note(Some(x), Some(x), text, line);
                    }
                } else if let Some((a, b)) = label.split_once("->") {
                    self.arrow(a, b, text);
                } else if let Some((a, b)) = label.split_once("<-") {
                    self.arrow(b, a, text);
                } else if label == "group" {
                    self.group(text);
                } else {
                    println!("% skipped {}: {}", label, text);
                }
            } else if s == "end" {
                self.end_group();
            } else {
                self.add_text(s);
            }

            line += 1;
        }
    }
}

fn main() {
    let mut items = Items::default();
    let mut file = false;
    for arg in std::env::args().skip(1) {
        let path = PathBuf::from(&arg);
        if let Ok(f) = File::open(&path) {
            file = true;
            if let Some(label) = path
                .file_stem()
                .or_else(|| path.file_name())
                .and_then(std::ffi::OsStr::to_str)
            {
                items.label(label);
            };
            items.parse(&mut io::BufReader::new(f));
            items.print();
        } else {
            panic!("cannot open file: {}", arg);
        }
    }
    if !file {
        items.parse(&mut io::BufReader::new(io::stdin()));
        items.print();
    }
}
