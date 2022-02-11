pub trait ResultExt<V, E: ToString> {
    fn log_err(self, label: &str) -> Result<V, E>;
}

impl<V, E: ToString> ResultExt<V, E> for Result<V, E> {
    fn log_err(self, label: &str) -> Result<V, E> {
        match self {
            Ok(t) => Ok(t),
            Err(e) => {
                tracing::error!("{}: {}", label, e.to_string());
                Err(e)
            }
        }
    }
}
