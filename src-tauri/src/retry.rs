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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_with_retry_success_first_try() {
        tauri::async_runtime::block_on(async {
            let attempts = Arc::new(AtomicU32::new(0));
            let attempts_clone = attempts.clone();

            let result: Result<i32, &'static str> = with_retry(
                || {
                    attempts_clone.fetch_add(1, Ordering::SeqCst);
                    async { Ok(42) }
                },
                |_| true,
                3,
            ).await;

            assert_eq!(result, Ok(42));
            assert_eq!(attempts.load(Ordering::SeqCst), 1);
        });
    }

    #[test]
    fn test_with_retry_failure_not_retryable() {
        tauri::async_runtime::block_on(async {
            let attempts = Arc::new(AtomicU32::new(0));
            let attempts_clone = attempts.clone();

            let result: Result<i32, &'static str> = with_retry(
                || {
                    attempts_clone.fetch_add(1, Ordering::SeqCst);
                    async { Err("fatal error") }
                },
                |e| *e == "retryable error",
                3,
            ).await;

            assert_eq!(result, Err("fatal error"));
            assert_eq!(attempts.load(Ordering::SeqCst), 1);
        });
    }

    #[test]
    fn test_with_retry_success_after_one_retry() {
        tauri::async_runtime::block_on(async {
            let attempts = Arc::new(AtomicU32::new(0));
            let attempts_clone = attempts.clone();

            let result: Result<i32, &'static str> = with_retry(
                || {
                    let current = attempts_clone.fetch_add(1, Ordering::SeqCst);
                    async move {
                        if current < 1 {
                            Err("retryable error")
                        } else {
                            Ok(100)
                        }
                    }
                },
                |e| *e == "retryable error",
                3,
            ).await;

            assert_eq!(result, Ok(100));
            assert_eq!(attempts.load(Ordering::SeqCst), 2);
        });
    }

    #[test]
    fn test_with_retry_failure_max_attempts() {
        tauri::async_runtime::block_on(async {
            let attempts = Arc::new(AtomicU32::new(0));
            let attempts_clone = attempts.clone();

            let result: Result<i32, &'static str> = with_retry(
                || {
                    attempts_clone.fetch_add(1, Ordering::SeqCst);
                    async { Err("retryable error") }
                },
                |e| *e == "retryable error",
                2,
            ).await;

            assert_eq!(result, Err("retryable error"));
            assert_eq!(attempts.load(Ordering::SeqCst), 2);
        });
    }
}
