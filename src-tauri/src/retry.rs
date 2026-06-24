use log::warn;
use std::future::Future;
use std::time::Duration;

pub async fn with_retry<F, Fut, G, T, E>(
    mut f: F,
    is_retryable: G,
    max_attempts: u32,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    G: Fn(&E) -> bool,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    loop {
        match f().await {
            Ok(val) => return Ok(val),
            Err(e) if attempt + 1 >= max_attempts => return Err(e),
            Err(e) => {
                if is_retryable(&e) {
                    let delay = Duration::from_secs(2u64.pow(attempt));
                    warn!("Request failed (attempt {attempt}): {e}, retrying in {delay:?}");
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                } else {
                    warn!("Request failed (attempt {attempt}): {e}, not retryable");
                    return Err(e);
                }
            }
        }
    }
}
