mod parser;

fn main() {
    env_logger::init();
    log::info!("Hello world");

    let manifest_path = std::env::args()
        .nth(1)
        .expect("please include a manifest path");

    let manifest = std::fs::read_to_string(manifest_path).unwrap();
    let start_time = std::time::Instant::now();
    let manifest = parser::Manifest::parse(manifest.as_str()).unwrap();
    let duration = std::time::Instant::now().duration_since(start_time);

    println!("{:#?}", manifest);
    println!("Parsed manifest in {:?}", duration);
}
