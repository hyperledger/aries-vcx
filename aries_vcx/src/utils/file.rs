use std::{
    fs::{DirBuilder, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use crate::errors::error::prelude::*;

pub fn write_file<P: AsRef<Path>>(file: P, content: &str) -> VcxResult<()>
where
    P: std::convert::AsRef<std::ffi::OsStr>,
{
    let path = PathBuf::from(&file);

    if let Some(parent_path) = path.parent() {
        DirBuilder::new().recursive(true).create(parent_path).map_err(|err| {
            AriesVcxError::from_msg(
                AriesVcxErrorKind::UnknownError,
                format!("Can't create the file: {}", err),
            )
        })?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .map_err(|err| {
            AriesVcxError::from_msg(AriesVcxErrorKind::UnknownError, format!("Can't open the file: {}", err))
        })?;

    file.write_all(content.as_bytes()).map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::UnknownError,
            format!("Can't write content: \"{}\" to the file: {}", content, err),
        )
    })?;

    file.flush().map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::UnknownError,
            format!("Can't write content: \"{}\" to the file: {}", content, err),
        )
    })?;

    file.sync_data().map_err(|err| {
        AriesVcxError::from_msg(
            AriesVcxErrorKind::UnknownError,
            format!("Can't write content: \"{}\" to the file: {}", content, err),
        )
    })
}
