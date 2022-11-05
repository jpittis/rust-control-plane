use crate::model::{parse_clusters, Cluster};
use log::info;
use std::error::Error;
use std::future::Future;
use std::io;
use std::process::{Child, Command};
use tokio::time::{Duration, Instant};

const CLUSTERS_URL: &'static str = "http://127.0.0.1:9901/clusters";

pub struct EnvoyProcess {
    config_path: String,
    service_node: String,
    service_cluster: String,
    child: Option<Child>,
    poll_timeout: Duration,
    poll_backoff: Duration,
}

impl EnvoyProcess {
    pub fn spawn(&mut self) -> io::Result<()> {
        let child = self.command().spawn()?;
        self.child = Some(child);
        Ok(())
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new("envoy");
        cmd.args([
            "-c",
            &self.config_path,
            "--service-node",
            &self.service_node,
            "--service-cluster",
            &self.service_cluster,
        ]);
        cmd
    }

    pub async fn poll_until_started(&self) -> Result<(), Box<dyn Error>> {
        self.poll_until(|| async {
            get_clusters().await?;
            Ok(())
        })
        .await
    }

    pub async fn poll_until_eq(&self, expected: Vec<Cluster>) -> Result<(), Box<dyn Error>> {
        self.poll_until(|| async {
            let clusters = get_clusters().await?;
            if clusters == expected {
                Ok(())
            } else {
                Err("Not equal".into())
            }
        })
        .await
    }

    pub async fn poll_until<T, F, Fut>(&self, mut f: F) -> Result<T, Box<dyn std::error::Error>>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, Box<dyn Error>>>,
    {
        let start = Instant::now();
        let mut failed_attempts = 0;
        loop {
            match f().await {
                Ok(val) => return Ok(val),
                Err(err) => {
                    failed_attempts += 1;
                    info!("failed_attempts {}", failed_attempts);
                    let backoff = self.poll_backoff;
                    if Instant::now().duration_since(start) > self.poll_timeout {
                        return Err(err);
                    }
                    let multiplier = 2_u32.pow(failed_attempts - 1) as f64;
                    tokio::time::sleep(backoff.mul_f64(multiplier)).await;
                }
            }
        }
    }

    pub fn kill(&mut self) -> io::Result<()> {
        self.child.as_mut().unwrap().kill()
    }
}

impl Drop for EnvoyProcess {
    fn drop(&mut self) {
        self.kill().unwrap()
    }
}

impl Default for EnvoyProcess {
    fn default() -> Self {
        Self {
            config_path: "envoy/config.yaml".to_string(),
            service_node: "lol".to_string(),
            service_cluster: "wat".to_string(),
            child: None,
            poll_timeout: Duration::from_millis(3200),
            poll_backoff: Duration::from_millis(200),
        }
    }
}

async fn get_clusters() -> Result<Vec<Cluster>, Box<dyn Error>> {
    let body = reqwest::get(CLUSTERS_URL).await?.text().await?;
    Ok(parse_clusters(&body))
}
