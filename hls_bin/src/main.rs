use hls::Line;

fn main() {
    env_logger::init();

    let path = std::env::args().nth(1).expect("Expected manifest to parse");
    let input = std::fs::read_to_string(path).unwrap();
    let manifest: Vec<Line> = hls::from_str(input.as_str()).unwrap();
    println!("{:#?}", manifest);
}
