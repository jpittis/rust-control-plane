pub mod get;
pub mod poll;

use std::io;
use std::process::{Child, Command, Stdio};

pub struct Config {
    pub ads: bool,
    pub delta: bool,
    pub service_node: String,
    pub service_cluster: String,
}

pub struct Process {
    config: Config,
    child: Option<Child>,
}

impl Process {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            child: None,
        }
    }

    pub fn spawn(&mut self) -> io::Result<()> {
        let child = self.command().spawn()?;
        self.child = Some(child);
        Ok(())
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new("envoy");
        cmd.args([
            "-c",
            &config_path(self.config.ads, self.config.delta),
            "--service-node",
            &self.config.service_node,
            "--service-cluster",
            &self.config.service_cluster,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
        cmd
    }

    fn kill(&mut self) -> io::Result<()> {
        self.child.as_mut().expect("No child found").kill()
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.kill().expect("Failed to kill process")
    }
}

fn config_path(ads: bool, delta: bool) -> String {
    if !delta && ads {
        return "envoy/ads.yaml".to_string();
    } else if delta && !ads {
        return "envoy/delta_config.yaml".to_string();
    } else if delta && ads {
        return "envoy/delta_ads.yaml".to_string();
    } else {
        return "envoy/config.yaml".to_string();
    }
}
