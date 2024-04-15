use std::path::PathBuf;

use clap::Parser;
use eyre::{bail, eyre, Context, OptionExt, Result};
use reqwest::Url;
use scraper::selectable::Selectable;
use scraper::{Html, Selector};
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

    let mut hash_val: u64 = fsize; // seed hash with file size

    let iterations = HASH_BLK_SIZE / 8;

    let mut reader = BufReader::with_capacity(HASH_BLK_SIZE as usize, file);

    for _ in 0..iterations {
        reader.read(&mut buf)?;
        unsafe {
            word = mem::transmute(buf);
        };
        hash_val = hash_val.wrapping_add(word);
    }

    reader.seek(SeekFrom::Start(fsize - HASH_BLK_SIZE))?;

    for _ in 0..iterations {
        reader.read(&mut buf)?;
        unsafe {
            word = mem::transmute(buf);
        };
        hash_val = hash_val.wrapping_add(word);
    }

    let hash_string = format!("{:01$x}", hash_val, 16);

    Ok(hash_string)
}

static BASE_URL: &str = "https://www.opensubtitles.org";

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let Cli {
        movie_path,
        language,
    } = Cli::parse();

    // println!("path to movie: {:?}, language: {:?} ", movie_path, language);
    info!(?movie_path, language, "got");

    let fsize = fs::metadata(&movie_path)
        .wrap_err("getting file size")?
        .len();
    if fsize < HASH_BLK_SIZE {
        bail!("File too small");
    }
    let file = File::open(&movie_path).wrap_err("Couldn't open file")?;
    let fhash = create_hash(file, fsize)?;
    info!(fhash, "hash of: {:?}", movie_path);

    // e.g. https://www.opensubtitles.org/pl/search2/sublanguageid-pol/moviehash-6f834ea3a2407f46 - where 6f... is calculated hash from our movie file
    let url: Url = format!("{BASE_URL}/pl/search2/sublanguageid-{language}/moviehash-{fhash}")
        .parse()
        .wrap_err("Getting url")?;

    let page = reqwest::get(url)
        .await
        .wrap_err("Fetching page")?
        .text()
        .await
        .wrap_err("parsing to string");

    //info!(?page, "Page content");

    let html = Html::parse_document(&page?);

    let selector = Selector::parse("table#search_results").map_err(|e| eyre!("{e:?}"))?;
    let search_results_table = html.select(&selector).next().map(|elem| elem.html());

    let mut sub_url = String::new();

    if let Some(table) = search_results_table {
        let tr_selector = Selector::parse("tr").map_err(|e| eyre!("{e:?}"))?;
        let td_selector = Selector::parse("td").map_err(|e| eyre!("{e:?}"))?;
        let a_selector = Selector::parse("a").map_err(|e| eyre!("{e:?}"))?;
        let html = Html::parse_fragment(&table[..]);
        //let tr_content = html.select(&tr_selector).skip(1).next().map(|e| e.value());
        //println!("DEBUG_TR: {tr_content:?}");
        let td_iterator = html
                .select(&tr_selector).skip(1).next().map(|elem| {
                    elem.select(&td_selector)
                });
        if let Some(mut td) = td_iterator {
            td.next().ok_or_else(|| eyre!("No Movie title column"))?;
            td.next().ok_or_else(|| eyre!("No Language column"))?;
            td.next().ok_or_else(|| eyre!("No #CD column"))?;
            td.next().ok_or_else(|| eyre!("No upload column"))?;
            let _ = td.next().ok_or_else(|| eyre!("No Subtitle Download URL")).and_then(|elem| {
                elem.select(&a_selector).next().ok_or_else(|| eyre!("No 'a' element")).and_then(|v| {
                    v.value().attr("href").ok_or_else(|| eyre!("No href element")).and_then(|download_url| {
                        sub_url = format!("{BASE_URL}{download_url}");
                        Ok(())
                    })
                })
            });
        }
    } else {
        info!("Table has not been found");
    }

    info!("Url for sub download: {}", sub_url);

    //sub_url = html.slect(selector)

    Ok(())
}
