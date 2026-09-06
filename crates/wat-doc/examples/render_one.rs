//! THROWAWAY probe — render the #wat.doc/Row for one already-decorated Rust item,
//! so the builder can see what the migration would actually insert.
use std::fs;

fn main() {
    let path = std::env::args().nth(1).expect("usage: render_one <file.rs> <start> <end>");
    let start: usize = std::env::args().nth(2).unwrap().parse().unwrap();
    let end: usize = std::env::args().nth(3).unwrap().parse().unwrap();
    let src = fs::read_to_string(&path).unwrap();
    let raw: String = src
        .lines()
        .skip(start - 1)
        .take(end - start + 1)
        .map(|l| l.trim_start().trim_start_matches("///").trim_start_matches(' '))
        .collect::<Vec<_>>()
        .join("\n");
    match wat_doc::parse(&raw) {
        Ok(doc) => println!("{}", wat_doc::print(&doc)),
        Err(e) => println!("PARSE REFUSED: {e:?}"),
    }
}
