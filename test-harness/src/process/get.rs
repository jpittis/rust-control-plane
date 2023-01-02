use pretty_assertions::Comparison;
use serde::Deserialize;
use std::fmt;

const CONFIG_DUMP_PARAMS: &str =
    "resource=dynamic_active_clusters&mask=cluster.name,cluster.lb_policy";

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ConfigDump {
    pub configs: Option<Vec<Resource>>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Resource {
    pub cluster: Cluster,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Cluster {
    pub name: String,
    pub lb_config: Option<String>,
}

#[derive(Debug)]
pub enum RequestError {
    RequestError(reqwest::Error),
    ParseError(serde_json::Error),
}

pub enum EqualityError {
    Request(RequestError),
    ClustersNotEqual(ConfigDump, ConfigDump),
}

impl fmt::Debug for EqualityError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EqualityError::Request(err) => write!(f, "{:?}", err),
            EqualityError::ClustersNotEqual(a, b) => write!(f, "{}", Comparison::new(&a, &b)),
        }
    }
}

pub async fn config_dump(port: u32) -> Result<ConfigDump, RequestError> {
    let body = reqwest::get(format!(
        "http://127.0.0.1:{}/config_dump?{}",
        port, CONFIG_DUMP_PARAMS,
    ))
    .await
    .map_err(RequestError::RequestError)?
    .text()
    .await
    .map_err(RequestError::RequestError)?;
    let config_dump: ConfigDump = serde_json::from_str(&body).map_err(RequestError::ParseError)?;
    Ok(config_dump)
}
