use std::path::PathBuf;

use clap::Parser;
use eyre::{bail, Context, Result};
use std::fs;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::mem;
use tracing::info;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    pub movie_path: PathBuf,
    #[arg(short, long, default_value = "eng")]
    pub language: String,
}

const HASH_BLK_SIZE: u64 = 65536;

fn create_hash(file: File, fsize: u64) -> Result<String, std::io::Error> {

    let mut buf = [0u8; 8];
    let mut word: u64;

    let mut hash_val: u64 = fsize;  // seed hash with file size

    let iterations = HASH_BLK_SIZE /  8;

    let mut reader = BufReader::with_capacity(HASH_BLK_SIZE as usize, file);

    for _ in 0..iterations {
        reader.read(&mut buf)?;
        unsafe { word = mem::transmute(buf); };
        hash_val = hash_val.wrapping_add(word);
    }

    reader.seek(SeekFrom::Start(fsize - HASH_BLK_SIZE))?;

    for _ in 0..iterations {
        reader.read(&mut buf)?;
        unsafe { word = mem::transmute( buf); };
        hash_val = hash_val.wrapping_add(word);
    }

    let hash_string = format!("{:01$x}", hash_val, 16);

    Ok(hash_string)
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

    let fsize = fs::metadata(&movie_path).wrap_err("getting file size")?.len();
    if fsize < HASH_BLK_SIZE {
        bail!("File too small");
    }
    let file = File::open(&movie_path).wrap_err("Opening file")?;
    let fhash = create_hash(file, fsize)?;
    info!(fhash, "hash of: {:?}", movie_path);

    Ok(())
}
