mod job;

use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use job::Job;
use serde::Deserialize;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Arguments {
    /// Specify config file
    #[clap(default_value = ".dwatch", short, long)]
    file: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct JobConfig {
    dir: String,
    filetypes: Option<Vec<String>>,
    scripts: Option<Vec<String>>,
    servers: Option<Vec<String>>,
    delay: Option<u64>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();
    let jobs_config: Vec<JobConfig> = serde_yaml::from_str(&std::fs::read_to_string(args.file)?)?;
    println!("{:?}", jobs_config);
    let mut jobs = jobs_config
        .iter()
        .map(|config| Job::from_config(config.clone()))
        .collect::<Vec<Job>>();

    let futs = jobs.iter_mut().map(|job| job.watch()).collect::<Vec<_>>();

    println!("n futs {}", futs.len());

    join_all(futs).await;

    Ok(())
}
