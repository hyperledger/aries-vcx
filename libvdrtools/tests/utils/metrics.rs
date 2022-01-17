use vdrtoolsrs::{IndyError, metrics, future::Future};

pub fn collect_metrics() -> Result<String, IndyError> {
    metrics::collect_metrics().wait()
}