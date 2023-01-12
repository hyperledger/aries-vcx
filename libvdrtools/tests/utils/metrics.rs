use vdrtoolsrs::{future::Future, metrics, IndyError};

pub fn collect_metrics() -> Result<String, IndyError> {
    metrics::collect_metrics().wait()
}
