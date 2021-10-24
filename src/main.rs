use chrono::{DateTime, Utc};
use clap::{AppSettings, Clap};
use dotenv::dotenv;
use std::collections::VecDeque;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
mod models;
mod repository;

#[derive(Clap)]
#[clap(version = "1.0", author = "Kevin K. <kbknapp@gmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "default.conf")]
    config: String,
    /// The path where the comparison will start.
    #[clap(short, long, default_value = "./")]
    start_path: String,
    /// The location where the db will be stored.
    #[clap(short, long, env = "DATABASE_URL")]
    db_target: String,
    /// All files below the threshold are ignored
    #[clap(short, long, default_value = "1000")]
    kb_threshold: u64,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let opts: Opts = Opts::parse();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    println!("Value for config: {}", opts.start_path);
    println!("Using input file: {}", opts.db_target);
    /*
    let creation_result = repository::PathRepository::new(&opts.db_target).await;
    let repo = creation_result.unwrap();
    let store_result = repo
        .store(models::PathInfo {
            id: 0,
            path: "test".to_string(),
            hash: "test".to_string(),
            last_modified: Utc::now(),
        })
        .await;

    let inserted_row = store_result.unwrap();

    let find_it_result = repo.find_by_path("test").await;

    let maybe_path = find_it_result.unwrap();

    match maybe_path {
        Some(path) => println!("Hash found: {}", path.hash),
        None => println!("could not find path"),
    }

    println!("Inserted row is: {}", inserted_row);

    //let paths = fs::read_dir("./").unwrap();
    */
    travel_dirs(opts).await.unwrap()
}

async fn travel_dirs(opts: Opts) -> anyhow::Result<()> {
    let blacklist = vec!["dll", "temp", "exe"];
    let mut queue: VecDeque<PathBuf> = VecDeque::new();

    let creation_result = repository::PathRepository::new(&opts.db_target).await;
    let repo = creation_result.unwrap();

    let start_path = Path::new(&opts.start_path);
    let start_absolute_path = fs::canonicalize(&start_path)?;
    queue.push_back(start_absolute_path);
    println!("Start iterating");
    while !queue.is_empty() {
        let visit_path = queue.pop_front().unwrap();
        let meta_data = visit_path.metadata().unwrap();
        let absolute_path = fs::canonicalize(&visit_path).unwrap();
        let absolute_path_str = absolute_path.as_path().display().to_string();

        if meta_data.is_dir() {
            for dir_entry in fs::read_dir(visit_path).unwrap().filter_map(|d| d.ok()) {
                let absolute_path_add = fs::canonicalize(dir_entry.path());
                match absolute_path_add {
                    Ok(e) => queue.push_back(e),
                    Err(e) => println!("Error occured when writing tio file {:?}", e),
                }
            }
        } else if meta_data.is_file() {
            let modification_date = DateTime::from(meta_data.modified().unwrap());
            let entry = repo.find_by_path(&absolute_path_str).await?;

            if meta_data.len() < opts.kb_threshold * 1024 {
                continue;
            }
            let fileExt = absolute_path
                .extension()
                .and_then(|s| s.to_str())
                .filter(|ext| blacklist.contains(ext));
            if fileExt.is_some() {
                println!(
                    "Skipped path because it is in blacjklist {:?}",
                    absolute_path_str
                );
                continue;
            }

            match entry {
                None => {
                    println!("Added new path {:?}", absolute_path_str);
                    let reader = BufReader::new(File::open(visit_path).unwrap());
                    let mut hasher = blake3::Hasher::new();
                    for line in reader.lines().filter_map(|l| l.ok()) {
                        hasher.update(line.as_bytes());
                    }
                    let hash = hasher.finalize();
                    let store_result = repo
                        .store(models::PathInfo {
                            path: absolute_path_str.to_string(),
                            hash: hash.to_string(),
                            last_modified: modification_date,
                        })
                        .await;
                    let inserted_row = store_result.unwrap();
                }
                Some(e) => {
                    if modification_date <= e.last_modified {
                        println!("Skipped path {:?}", absolute_path_str);
                        continue;
                    }
                    println!("Updated path {:?}", absolute_path_str);
                    let reader = BufReader::new(File::open(visit_path).unwrap());
                    let mut hasher = blake3::Hasher::new();
                    for line in reader.lines().filter_map(|l| l.ok()) {
                        hasher.update(line.as_bytes());
                    }
                    let hash = hasher.finalize();
                    let store_result = repo
                        .update(models::PathInfo {
                            path: absolute_path_str.to_string(),
                            hash: hash.to_string(),
                            last_modified: modification_date,
                        })
                        .await;
                    let inserted_row = store_result.unwrap();
                }
            }
        }
    }
    Ok(())
}
