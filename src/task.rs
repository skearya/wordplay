use tokio::task::JoinHandle;

pub fn spawn<F, T>(future: F) -> JoinHandle<anyhow::Result<T>>
where
    F: Future<Output = anyhow::Result<T>> + Send + 'static,
    T: Send + 'static,
{
    tokio::spawn(async {
        match future.await {
            Ok(res) => Ok(res),
            Err(err) => {
                tracing::error!(?err);
                Err(err)
            }
        }
    })
}
