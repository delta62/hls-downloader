use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(long, short)]
    pub manifest_url: String,
}
