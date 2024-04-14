use std::path::PathBuf;

use eyre::Result;
use clap::Parser;


#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    movie_path: PathBuf,
    #[arg(short, long, default_value = "eng")]
    language: String
}

#[tokio::main]
async fn main() -> Result<()> {

    let Cli {
        movie_path,
        language,
    } = Cli::parse();

    println!("path to movie: {:?}, language: {:?} ", movie_path, language);
    Ok(())
}
