use crate::model::{Cluster, Endpoint};
use std::collections::HashMap;

pub async fn get_clusters() -> Result<Vec<Cluster>, Box<dyn std::error::Error>> {
    let mut clusters = HashMap::new();
    let body = reqwest::get("http://127.0.0.1:9901/clusters")
        .await?
        .text()
        .await?;
    body.split("\n")
        .filter(|line| line.contains("cx_active"))
        .for_each(|line| {
            let mut parts = line.split("::");
            let name = parts.next().unwrap().to_string();
            let addr_port = parts.next().unwrap();
            let mut parts = addr_port.split(":");
            let addr = parts.next().unwrap().to_string();
            let port = parts.next().unwrap().parse::<u32>().unwrap();
            let endpoint = Endpoint { addr, port };
            clusters
                .entry(name.clone())
                .and_modify(|cluster: &mut Cluster| cluster.endpoints.push(endpoint.clone()))
                .or_insert_with(|| Cluster {
                    name,
                    endpoints: vec![endpoint],
                });
        });
    Ok(Vec::from_iter(clusters.values().cloned()))
}

pub async fn poll_until_eq(expected: Vec<Cluster>) -> Result<(), Box<dyn std::error::Error>> {
    let mut failures = 0;
    loop {
        let clusters = get_clusters().await?;
        if clusters == expected {
            return Ok(());
        }
        failures += 1;
        if failures >= 5 {
            return Err("too many attempts".into());
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
    }
}
