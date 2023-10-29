use std::{error::Error, fmt};

#[derive(Debug, thiserror::Error)]
pub struct HttpError {
    msg: String,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Error: {}\n", self.msg)?;
        let mut current = self.source();
        while let Some(cause) = current {
            writeln!(f, "Caused by:\n\t{cause}")?;
            current = cause.source();
        }
        Ok(())
    }
}

impl HttpError {
    pub fn from_msg<D>(msg: D) -> HttpError
    where
        D: fmt::Display + fmt::Debug + Send + Sync + 'static,
    {
        HttpError {
            msg: msg.to_string(),
        }
    }
}

pub fn err_msg<D>(msg: D) -> HttpError
where
    D: fmt::Display + fmt::Debug + Send + Sync + 'static,
{
    HttpError::from_msg(msg)
}

pub type HttpResult<T> = Result<T, HttpError>;
