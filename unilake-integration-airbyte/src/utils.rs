use std::env;
use std::ffi::OsString;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use opendal::Scheme;
use serde_json::Value;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

pub fn get_target_fs() -> Scheme {
    let msg = "FS_SCHEME environment variable is not defined or has an invalid value (fs, azblob, azdfs, gcs, s3)";
    match env::var_os("FS_SCHEME") {
        Some(v) => match v.to_ascii_lowercase().to_str() {
            Some("fs") => Scheme::Fs,
            Some("azblob") => Scheme::Azblob,
            Some("azdfs") => Scheme::Azdfs,
            Some("gcs") => Scheme::Gcs,
            Some("s3") => Scheme::S3,
            _ => panic!("{}", msg),
        },
        None => panic!("{}", msg),
    }
}

pub async fn save_json_to_file(value: &Value, path: &OsString) -> anyhow::Result<()> {
    let file = File::create(path).await?;
    let mut writer = tokio::io::BufWriter::new(file);

    let json_string = serde_json::to_string_pretty(value)?;
    writer.write_all(json_string.as_bytes()).await?;

    Ok(())
}

pub fn get_current_time_in_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}
