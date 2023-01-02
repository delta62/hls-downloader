use lalrpop_util::lalrpop_mod;

mod ast;
lalrpop_mod!(pub hls);
mod manifest;

fn main() {
    env_logger::init();
    log::info!("Hello world");

    let manifest =
        std::fs::read_to_string("/home/sam/src/dvr_clj/resources/1800_complete.m3u8").unwrap();
    let start_time = std::time::Instant::now();
    let ast = hls::ManifestParser::new().parse(&manifest).unwrap();
    let manifest = manifest::MediaManifest::from_ast(&ast);
    let duration = std::time::Instant::now().duration_since(start_time);

    println!("{:#?}", manifest.unwrap());
    println!("{:#?}", duration);
}
