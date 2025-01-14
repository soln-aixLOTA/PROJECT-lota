use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ResourceManager<T> {
    resource: Arc<Mutex<T>>,
}

impl<T> ResourceManager<T> {
    pub fn new(resource: T) -> Self {
        Self {
            resource: Arc::new(Mutex::new(resource)),
        }
    }

    pub async fn with_resource<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.resource.lock().await;
        f(&mut *guard)
    }
}

impl<T> Drop for ResourceManager<T> {
    fn drop(&mut self) {
        // Perform any necessary cleanup here
        log::info!("ResourceManager is being dropped, performing cleanup");
    }
} 