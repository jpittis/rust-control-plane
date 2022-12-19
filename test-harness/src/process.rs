use crate::model::{parse_clusters, sort_clusters, Cluster, Endpoint};
use pretty_assertions::Comparison;
use std::fmt;
use std::future::Future;
use std::io;
use std::process::{Child, Command, Stdio};
use tokio::time::{Duration, Instant};

pub enum PollError {
    Reqwest(reqwest::Error),
    ClustersNotEqual(Vec<Cluster>, Vec<Cluster>),
}

impl fmt::Debug for PollError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PollError::Reqwest(err) => write!(f, "{}", err),
            PollError::ClustersNotEqual(a, b) => write!(f, "{}", Comparison::new(&a, &b)),
        }
    }
}

const CLUSTERS_URL: &str = "http://127.0.0.1:9901/clusters";

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
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null());
        cmd
    }

    pub async fn poll_until_started(&self) -> Result<(), PollError> {
        self.poll_until(|| async {
            get_clusters().await.map_err(PollError::Reqwest)?;
            Ok(())
        })
        .await
    }

    pub async fn poll_until_eq(&self, mut expected: Vec<Cluster>) -> Result<(), PollError> {
        // Append the default static xds cluster that's always present.
        expected.push(Cluster {
            name: "xds".to_string(),
            hidden: false,
            endpoints: vec![Endpoint {
                addr: "127.0.0.1".to_string(),
                port: 5678,
            }],
        });
        // Filter out hidden clusters.
        expected.retain(|cluster| !cluster.hidden);
        // Sort to make sure the clusters are still in the right order.
        sort_clusters(&mut expected);
        self.poll_until(|| async {
            let clusters = get_clusters().await.map_err(PollError::Reqwest)?;
            if clusters == expected {
                Ok(())
            } else {
                Err(PollError::ClustersNotEqual(clusters, expected.clone()))
            }
        })
        .await
    }

    pub async fn poll_until<T, F, Fut, E>(&self, mut f: F) -> Result<T, E>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        let start = Instant::now();
        let mut failed_attempts = 0;
        loop {
            match f().await {
                Ok(val) => return Ok(val),
                Err(err) => {
                    failed_attempts += 1;
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

async fn get_clusters() -> Result<Vec<Cluster>, reqwest::Error> {
    let body = reqwest::get(CLUSTERS_URL).await?.text().await?;
    Ok(parse_clusters(&body))
}
