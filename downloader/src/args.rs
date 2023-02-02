use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long, short)]
    pub base_url: String,

    #[clap(long, short)]
    pub manifest_path: String,

    #[clap(long, short)]
    pub output_dir: String,
}
