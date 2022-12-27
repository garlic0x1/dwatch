use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use std::process::Command;
use std::{collections::HashMap, thread, time};
use walkdir;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Arguments {
    #[clap(default_value = ".dwatch", short, long)]
    file: String,

    #[clap(default_value_t = 2, short, long)]
    delay: u64,
}

#[derive(Deserialize)]
struct JobConfig {
    dir: String,
    script: String,
}

struct Job {
    dir: String,
    script: String,
    history: HashMap<String, time::SystemTime>,
}

fn updated(job: &mut Job) -> bool {
    0 < walkdir::WalkDir::new(job.dir.clone())
        .into_iter()
        .filter_map(|file| file.ok())
        .map(|file| -> Result<bool> {
            let modified = file.metadata()?.modified()?;
            let filename = file.path().to_str().unwrap_or(".").to_string();
            if let Some(last) = job.history.insert(filename.clone(), modified) {
                Ok(last != modified)
            } else {
                Ok(true)
            }
        })
        .filter_map(|res| res.ok())
        .filter(|res| *res)
        .count()
}

fn do_job(job: &mut Job) {
    if updated(job) {
        println!("running: {}", &job.script);
        let out = Command::new("sh")
            .arg("-c")
            .arg(&job.script)
            .output()
            .expect("failed to execute process");
        let stdout = String::from_utf8(out.stdout).unwrap();
        let stderr = String::from_utf8(out.stderr).unwrap();
        println!("STDOUT:\n{}", stdout);
        if stderr != "" {
            println!("STDERR:\n{}", stderr);
        }
    }
}

fn main() -> Result<()> {
    let args = Arguments::parse();
    let jobs_config: Vec<JobConfig> = serde_yaml::from_str(&std::fs::read_to_string(args.file)?)?;
    let mut jobs: Vec<Job> = jobs_config
        .iter()
        .map(|config| Job {
            dir: config.dir.clone(),
            script: config.script.clone(),
            history: HashMap::new(),
        })
        .collect();

    loop {
        thread::sleep(time::Duration::from_secs(args.delay));
        for i in 0..jobs.len() {
            do_job(&mut jobs[i]);
        }

        if false {
            break;
        }
    }

    Ok(())
}
