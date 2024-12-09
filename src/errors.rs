use std::result;

use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum Errors {
    #[error("failed to read from data file")]
    FailedReadFromDataFile,

    #[error("failed to write to data file")]
    FailedWriteToDataFile,

    #[error("failed to sync data file")]
    FailedSyncDataFile,

    #[error("failed to open data file")]
    FailedToOpenDataFile,

    #[error("this key is empty")]
    KeyIsEmpty,

    #[error("memory index failed to update")]
    IndexUpdateFailed,

    #[error("key is not found in database")]
    KeyNotFound,

    #[error("datafile is not found in database")]
    DataFileNotFound,

    #[error("database directory path can not be empty")]
    DirPathIsEmpty,

    #[error("database data file size must be greater than 0")]
    DataFileSizeTooSmall,

    #[error("filed to create the database directory")]
    FailedToCreateDatabaseDir,

    #[error("database directory is corrupted")]
    DataDirectoryCorrupted,

    #[error("read data file eof")]
    ReadDataFileEOF,

    #[error("invalid crc value, log record maybe corrupted")]
    InvalidLogRecordCrc,
}

pub type Result<T> = result::Result<T, Errors>;
