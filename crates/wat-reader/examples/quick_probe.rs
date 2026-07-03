fn main() {
    let cases = ["😀", ":a😀", "é", ":aé", "(:a😀)", "foo∅bar", "expected ∅, got String"];
    for src in cases {
        let r = std::panic::catch_unwind(|| wat_reader::parse_one_with_file(src, "<t>"));
        match r {
            Err(_) => println!("{src:?} => PANIC"),
            Ok(Ok(v)) => println!("{src:?} => OK: {v:?}"),
            Ok(Err(e)) => println!("{src:?} => ERR: {e:?}"),
        }
    }
}
