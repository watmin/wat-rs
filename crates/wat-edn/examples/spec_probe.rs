fn main() {
    use wat_edn::parse;
    for input in [":foo:bar", ":foo#bar", "foo:bar"] {
        match parse(input) {
            Ok(v) => println!("{:?} → OK: {:?}", input, v),
            Err(e) => println!("{:?} → ERR: {}", input, e),
        }
    }
}
