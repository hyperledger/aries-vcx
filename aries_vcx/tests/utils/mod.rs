use futures::{Future, FutureExt};

pub mod devsetup_agent;
pub mod scenarios;
pub mod test_macros;

pub async fn force_debug_stack<F: Future>(f: F) -> F::Output {
    let result = async move {
        // AssertUnwindSafe moved to the future
        std::panic::AssertUnwindSafe(f).catch_unwind().await
    }
    .await;

    match result {
        Ok(x) => return x,
        Err(e) => {
            let panic_information = match e.downcast::<String>() {
                Ok(v) => *v,
                Err(e) => match e.downcast::<&str>() {
                    Ok(v) => v.to_string(),
                    _ => "Unknown Source of Error".to_owned(),
                },
            };
            println!("{}", panic_information);
            panic!("force debug stack failed")
        }
    }
}