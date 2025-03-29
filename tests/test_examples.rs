use aigerox::aig::Aig;

fn parse_all(pattern: &str) {
    for entry in glob::glob(pattern).unwrap() {
        let path = entry.unwrap();
        println!("Loading '{}'", path.display());
        let aig = Aig::from_file(&path).unwrap();
        println!("aig = {}", aig);
    }
}

#[test]
fn test_load_examples() {
    parse_all("data/examples/*.aag");
}

#[test]
fn test_load_arithmetic() {
    parse_all("data/arithmetic/*.aag");
}
