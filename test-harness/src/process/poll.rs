use std::future::Future;
use tokio::time::{Duration, Instant};

pub struct Config {
    pub timeout: Duration,
    pub backoff: Duration,
}

pub async fn until<T, F, Fut, E>(config: Config, mut f: F) -> Result<T, E>
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
                if Instant::now().duration_since(start) > config.timeout {
                    return Err(err);
                }
                let multiplier = 2_u32.pow(failed_attempts - 1) as f64;
                tokio::time::sleep(config.backoff.mul_f64(multiplier)).await;
            }
        }
    }
}
