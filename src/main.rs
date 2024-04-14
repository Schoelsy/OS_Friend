use std::path::PathBuf;

use eyre::Result;
use clap::Parser;
use tracing::info;


#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    pub movie_path: PathBuf,
    #[arg(short, long, default_value = "eng")]
    pub language: String
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let Cli {
        movie_path,
        language,
    } = Cli::parse();

   // println!("path to movie: {:?}, language: {:?} ", movie_path, language);
    info!(?movie_path, language, "got");
    Ok(())
}
