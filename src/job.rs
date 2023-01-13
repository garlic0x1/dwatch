use anyhow::Result;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{Child, Command};
use std::time;

use crate::JobConfig;

pub struct Job {
    dir: String,
    filetypes: Option<Vec<String>>,
    scripts: Vec<String>,
    servers: Vec<String>,
    delay: u64,
    processes: HashMap<String, Child>,
    history: HashMap<String, time::SystemTime>,
}

impl Job {
    pub fn from_config(cfg: JobConfig) -> Self {
        Self {
            dir: cfg.dir,
            filetypes: cfg.filetypes,
            scripts: cfg.scripts.unwrap_or_default(),
            servers: cfg.servers.unwrap_or_default(),
            delay: cfg.delay.unwrap_or(2),
            processes: HashMap::new(),
            history: HashMap::new(),
        }
    }

    pub async fn watch(&mut self) {
        loop {
            if self.updated() {
                self.run_scripts();
                self.run_servers();
            }
            tokio::time::sleep(time::Duration::from_secs(self.delay)).await;
        }
    }

    fn run_servers(&mut self) {
        self.servers.iter().for_each(|server| {
            if let Some(proc) = self.processes.get_mut(server) {
                proc.kill().expect("failed to kill child");
            }

            println!("restarting: '{}'", server);
            let child = Command::new("sh")
                .arg("-c")
                .arg(server)
                .spawn()
                .expect("failed to spawn process");

            self.processes.insert(server.into(), child);
        })
    }

    fn run_scripts(&self) {
        self.scripts.iter().for_each(|script| {
            println!("running: '{}'", script);
            let out = Command::new("sh")
                .arg("-c")
                .arg(script)
                .output()
                .expect("failed to execute process");
            let stdout = String::from_utf8(out.stdout).unwrap();
            let stderr = String::from_utf8(out.stderr).unwrap();

            println!("STDOUT:\n{}", stdout);
            if !stderr.is_empty() {
                println!("STDERR:\n{}", stderr);
            }
        });
    }

    fn updated(&mut self) -> bool {
        0 < walkdir::WalkDir::new(self.dir.clone())
            .into_iter()
            .filter_map(|file| file.ok())
            // check filetype
            .filter(|file| {
                let ext = Path::new(file.file_name())
                    .extension()
                    .and_then(OsStr::to_str)
                    .unwrap_or_default()
                    .to_string();
                self.filetypes.is_none() || self.filetypes.as_ref().unwrap().contains(&ext)
            })
            // check modified
            .map(|file| -> Result<bool> {
                let modified = file.metadata()?.modified()?;
                let filename = file.path().to_str().unwrap_or(".").to_string();
                if let Some(last) = self.history.insert(filename, modified) {
                    Ok(last != modified)
                } else {
                    Ok(true)
                }
            })
            .filter_map(|res| res.ok())
            .filter(|res| *res)
            .count()
    }
}
