pub trait IpcDispatcher: Send + Sync {
    fn dispatch(&self, request: &str) -> String;
}
