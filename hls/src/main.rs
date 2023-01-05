use hls_derive::from_str;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Foo {
    bar: String,
}

fn main() {
    let path = std::env::args().nth(1).expect("Expected manifest to parse");
    let input = std::fs::read_to_string(path).unwrap();
    let manifest: Foo = hls_derive::from_str(input.as_str()).unwrap();
    println!("{:?}", manifest);
}
