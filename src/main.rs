use clap::{AppSettings, Clap};
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
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    println!("Value for config: {}", opts.start_path);
    println!("Using input file: {}", opts.db_target);

    let creation_result = repository::PathRepository::new(&opts.db_target).await;
    let repo = creation_result.unwrap();
    let store_result = repo
        .store(models::PathInfo {
            id: 0,
            path: "test".to_string(),
            hash: "test".to_string(),
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
}


async fn upsert()
