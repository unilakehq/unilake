use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;

use anyhow::Error;
use anyhow::Result;
use async_trait::async_trait;
use opendal::Scheme;
use reqwest::header::HeaderValue;
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use reqwest::Method;
use reqwest::RequestBuilder;
use reqwest::Url;
use serde_json::json;
use serde_json::to_value;
use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::error::BackendError;
use crate::model::AirbyteMessage;
use crate::model::ProcessResult;
use crate::utils;

const MAX_RETRIES: u32 = 5;
const INITIAL_BACKOFF: u64 = 1000; // 1 second

pub struct BackendFactory {}
impl BackendFactory {
    fn get_target_backend() -> BackendType {
        match env::var_os("FS_BACKEND") {
            Some(v) => match v.to_ascii_lowercase().to_str() {
                Some("local") => BackendType::Local,
                Some("remote") => BackendType::OpenDal,
                Some("unilake") => BackendType::Unilake,
                // TODO: preferably we should provide a warning or error here
                _ => BackendType::Local,
            },
            _ => BackendType::Local,
        }
    }

    fn get_source_id() -> String {
        // TODO: we need to get the source id from an environment variable, this id represents the source as stored by the orchestration process
        todo!()
    }

    fn get_run_id() -> String {
        // TODO: we need to get the run id from an environment variable, this id represents the run as stored by the orchestration process
        todo!()
    }

    pub fn from_env() -> Option<Box<dyn Backend>> {
        let envs: HashMap<String, String> = env::vars()
            .filter_map(|(k, v)| {
                k.to_lowercase()
                    .strip_prefix("FS_BACKEND_")
                    .map(|k| (k.to_string(), v))
            })
            .collect();

        match Self::get_target_backend() {
            BackendType::Local => Some(Box::new(LocalBackend::new(
                &envs,
                Self::get_source_id(),
                Self::get_run_id(),
            ))),
            BackendType::OpenDal => Some(Box::new(OpenDalBackend::new(
                &envs,
                Self::get_source_id(),
                Self::get_run_id(),
            ))),
            BackendType::Unilake => Some(Box::new(UnilakeBackend::new(
                &envs,
                Self::get_source_id(),
                Self::get_run_id(),
            ))),
        }
    }
}

#[async_trait]
pub trait Backend: Send + Sync {
    /// Push log messages to the backend
    async fn push_logs(&self, logs: &Vec<String>) -> Result<()>;

    /// Push any airbytemessage protocol message to the backend
    async fn push_messages(&self, messages: &Vec<AirbyteMessage>) -> Result<()>;

    /// Get the configured catalog from the backend
    async fn get_configured_catalog(&self) -> Result<Value>;

    /// Get the connector config from the backend
    async fn get_config(&self) -> Result<Value>;

    /// Get the current process result
    fn get_result(&self) -> Arc<RwLock<ProcessResult>>;

    /// Pushes the end result of this source process onto the backend
    async fn push_result(&self) -> Result<()>;

    /// Push the current state file to the backend
    async fn push_state(&self, state: Option<serde_json::Value>) -> Result<()>;

    /// Get the current state file, will return an error if another process has the lock
    async fn get_state(&self) -> Result<Value>;

    /// Execute a request to extend the current state lock
    async fn extend_state_lock(&self) -> Result<u32>;

    /// Returns the number of seconds left before the lock on the state file expires.
    async fn state_lock_remaining_seconds(&self) -> Result<u32>;

    /// Flush any pending messages and logs
    async fn flush(&self) -> Result<()>;

    /// Returns the currently implemented backend
    fn get_backend_type(&self) -> BackendType;
}

#[derive(Debug)]
pub enum BackendType {
    Local,
    OpenDal,
    Unilake,
}

struct BaseBackend {
    source_id: String,
    run_id: String,
    // TODO: properly initialize state from a backend and send state to backend where needed
    _state_path: String,
    log_path: String,
    log_local: RwLock<tokio::fs::File>,
    result: Arc<RwLock<ProcessResult>>,
}

impl BaseBackend {
    pub fn new(
        v: &HashMap<String, String>,
        source_id: String,
        run_id: String,
        t: BackendType,
    ) -> Result<Self> {
        let rt = tokio::runtime::Handle::current();
        let log_file_path = Self::get_log_file_path(&source_id, &run_id)?;
        let log_local = RwLock::new(rt.block_on(tokio::fs::File::create(&log_file_path))?);

        let state_path = v.get("FS_BACKEND_STATE_PATH").unwrap_or_else(|| {
            // TODO: for any of these panics, we need to implement proper errors and provide it as an end-state or logging message
            panic!("{:?}Backend: missing env variable FS_BACKEND_STATE_PATH", t)
        });

        Ok(Self {
            log_local,
            source_id,
            run_id,
            _state_path: state_path.clone(),
            log_path: log_file_path.display().to_string(),
            result: Arc::new(RwLock::new(ProcessResult::default())),
        })
    }

    fn get_log_file_path(source_id: &str, run_id: &str) -> Result<PathBuf> {
        let mut path = env::current_dir()?;
        path.push(format!("log_{}_{}.txt", source_id, run_id));
        Ok(path)
    }

    pub async fn save_log_lines(&self, lines: &[String]) -> Result<()> {
        let mut log_local = self.log_local.write().await;
        for line in lines {
            log_local.write(line.as_bytes()).await?;
            log_local.write(b"\n").await?;
        }
        Ok(())
    }

    pub fn print_log_lines(&self, lines: &[String]) {
        for line in lines {
            println!("{}", line);
        }
    }
}

struct OpenDalBackend {
    base: BaseBackend,
    _scheme: Scheme,
}
impl OpenDalBackend {
    pub fn new(v: &HashMap<String, String>, source_id: String, run_id: String) -> Self {
        Self {
            base: BaseBackend::new(&v, source_id, run_id, BackendType::OpenDal).unwrap(),
            _scheme: utils::get_target_fs(),
        }
    }
}
#[async_trait]
impl Backend for OpenDalBackend {
    async fn push_logs(&self, logs: &Vec<String>) -> Result<()> {
        self.base.print_log_lines(logs);
        self.base.save_log_lines(logs).await
    }
    async fn get_configured_catalog(&self) -> Result<Value> {
        todo!();
    }
    async fn push_messages(&self, _messages: &Vec<AirbyteMessage>) -> Result<()> {
        todo!();
    }
    async fn get_config(&self) -> Result<serde_json::Value> {
        todo!();
    }
    fn get_result(&self) -> Arc<RwLock<ProcessResult>> {
        self.base.result.clone()
    }
    async fn push_result(&self) -> Result<()> {
        todo!();
    }
    async fn push_state(&self, _state: Option<serde_json::Value>) -> Result<()> {
        todo!();
    }
    async fn get_state(&self) -> Result<serde_json::Value> {
        todo!();
    }
    async fn extend_state_lock(&self) -> Result<u32> {
        todo!();
    }
    async fn state_lock_remaining_seconds(&self) -> Result<u32> {
        todo!();
    }
    async fn flush(&self) -> Result<()> {
        todo!();
    }
    fn get_backend_type(&self) -> BackendType {
        BackendType::OpenDal
    }
}

pub struct UnilakeBackend {
    base: BaseBackend,
    base_url: Url,
    application_id: String,
    application_secret: String,
    client: Client,
    access_token: Option<String>,
    access_token_expires: Option<Duration>,
}
impl UnilakeBackend {
    pub fn new(v: &HashMap<String, String>, source_id: String, run_id: String) -> Self {
        Self {
            base: BaseBackend::new(&v, source_id, run_id, BackendType::Unilake).unwrap(),
            base_url: Url::parse(
                &v.get("FS_BACKEND_BASE_URL")
                    .expect("UnilakeBackend: missing env variable FS_BACKEND_BASE_URL")
                    .clone(),
            )
            .expect("Could not parse base URL"),
            application_id: v
                .get("FS_BACKEND_APPLICATION_ID")
                .expect("UnilakeBackend: missing env variable FS_BACKEND_APPLICATION_ID")
                .clone(),
            application_secret: v
                .get("FS_BACKEND_APPLICATION_SECRET")
                .expect("UnilakeBackend: missing env variable FS_BACKEND_APPLICATION_SECRET")
                .clone(),
            client: Client::new(),
            access_token: None,
            access_token_expires: None,
        }
    }

    fn is_access_token_expired(&self) -> bool {
        match self.access_token {
            None => true,
            Some(_) => {
                if let Some(expires) = self.access_token_expires {
                    let now = SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .expect("Time went backwards");
                    expires <= now
                } else {
                    false
                }
            }
        }
    }

    async fn get_access_token(&self) -> Result<String> {
        let request = Client::new().post(self.base_url.join("")?).body(
            json!({
                "grant_type": "refresh_token",
                "client_id": self.application_id,
                "client_secret": self.application_secret,
            })
            .to_string(),
        );
        let response = Self::send_request_with_retry(request).await?;
        // TODO: we are not setting the duration of the access token here!
        return if let Some(v) = response {
            if let Some(token) = v.get("access_token") {
                Ok(token.to_string())
            } else {
                Err(Error::new(BackendError::RequestFailed(format!(
                    "Failed to get access token, no access token returned"
                ))))
            }
        } else {
            Err(Error::new(BackendError::RequestFailed(format!(
                "Failed to get access token, empty response"
            ))))
        };
    }

    async fn get_request(&self, path: &str) -> Result<serde_json::Value> {
        if self.is_access_token_expired() {
            self.get_access_token().await?;
        }
        let request = self.prepare_request(Method::GET, path, None)?;
        return if let Some(r) = Self::send_request_with_retry(request).await? {
            Ok(r)
        } else {
            Ok(serde_json::Value::Null)
        };
    }

    async fn post_request(
        &self,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<Option<serde_json::Value>> {
        if self.is_access_token_expired() {
            self.get_access_token().await?;
        }
        let request = self.prepare_request(Method::POST, path, body)?;
        Self::send_request_with_retry(request).await
    }

    async fn upload_file(&self, path: &str, buffer: Vec<u8>) -> Result<()> {
        if self.is_access_token_expired() {
            self.get_access_token().await?;
        }

        let response = self
            .prepare_request(Method::POST, path, None)?
            .header("Content-Type", "application/octet-stream")
            .body(reqwest::Body::from(buffer))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(Error::new(BackendError::RequestFailed(format!(
                "Failed to upload file with path: '{}', due to error: {}",
                path,
                response.status()
            ))))
        }
    }

    fn prepare_request(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<RequestBuilder> {
        let url = self.base_url.join(path)?;
        let mut builder = self.client.request(method, url);
        builder = self.add_access_token_params(builder);
        builder = self.add_query_params(builder);
        if let Some(body) = body {
            builder = builder.json(&body);
        }

        Ok(builder)
    }

    fn add_query_params(&self, request_builder: RequestBuilder) -> RequestBuilder {
        request_builder.query(&[
            ("sourceid", &self.base.source_id),
            ("runid", &self.base.run_id),
        ])
    }

    fn add_access_token_params(&self, request_builder: RequestBuilder) -> RequestBuilder {
        request_builder.header(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.access_token.as_ref().unwrap()))
                .unwrap(),
        )
    }

    async fn send_request_with_retry(
        request_builder: RequestBuilder,
    ) -> Result<Option<serde_json::Value>> {
        let mut retries = 0;
        let mut backoff = INITIAL_BACKOFF;

        loop {
            let response = request_builder.try_clone().unwrap().send().await?;
            if response.status().is_success() {
                return if let Ok(body) = response.text().await {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                        Ok(Some(json))
                    } else {
                        Err(Error::new(BackendError::RequestFailed(format!(
                            "Failed to parse response body: {}",
                            body
                        ))))
                    }
                } else {
                    Ok(None)
                };
            } else {
                if retries >= MAX_RETRIES || response.status().is_server_error() {
                    return Err(Error::new(BackendError::RequestFailed(format!(
                        "Failed to send request: {}",
                        response.status()
                    ))));
                }
            }

            retries += 1;
            // Back off before retrying
            let now = Instant::now();
            tokio::time::sleep(Duration::from_millis(backoff)).await;
            let elapsed = now.elapsed().as_millis() as u64;
            backoff *= 2;
            backoff = backoff.saturating_sub(elapsed);
        }
    }
}
#[async_trait]
impl Backend for UnilakeBackend {
    async fn push_logs(&self, logs: &Vec<String>) -> Result<()> {
        self.base.print_log_lines(logs);
        self.base.save_log_lines(logs).await?;
        self.post_request("", Some(to_value(logs).unwrap())).await?;
        Ok(())
    }
    async fn get_config(&self) -> Result<Value> {
        todo!();
    }
    async fn push_messages(&self, messages: &Vec<AirbyteMessage>) -> Result<()> {
        self.post_request("", Some(to_value(messages).unwrap()))
            .await?;
        Ok(())
    }
    fn get_result(&self) -> Arc<RwLock<ProcessResult>> {
        self.base.result.clone()
    }
    async fn get_configured_catalog(&self) -> Result<Value> {
        todo!();
    }
    async fn push_result(&self) -> Result<()> {
        // self.post_request("", Some(to_value(result).unwrap()))
        //     .await?;
        //Ok(())
        todo!();
    }
    async fn push_state(&self, state: Option<serde_json::Value>) -> Result<()> {
        if state.is_none() {
            // TODO: log warning
            return Ok(());
        }
        self.post_request("", state).await?;
        Ok(())
    }
    async fn get_state(&self) -> Result<serde_json::Value> {
        self.get_request("").await
    }
    async fn extend_state_lock(&self) -> Result<u32> {
        todo!();
        // if let Some(r) = self.put_request("", Some(json!({}))).await? {
        //     //r.get("")?.as_u64()?
        // }
    }
    async fn state_lock_remaining_seconds(&self) -> Result<u32> {
        // Check current state lock locally, extend and get via API
        todo!();
    }
    async fn flush(&self) -> Result<()> {
        let mut file = tokio::fs::OpenOptions::new()
            .open(&self.base.log_path)
            .await?;
        let mut buffer = Vec::new();
        let _b = file.read_to_end(&mut buffer).await?;
        // TODO: log info for size _b
        self.upload_file("", buffer).await
    }
    fn get_backend_type(&self) -> BackendType {
        BackendType::Unilake
    }
}

struct LocalBackend {
    base: BaseBackend,
    msg_file: RwLock<tokio::fs::File>,
}
impl LocalBackend {
    pub fn new(v: &HashMap<String, String>, source_id: String, run_id: String) -> Self {
        let rt = tokio::runtime::Handle::current();
        let base = BaseBackend::new(&v, source_id, run_id, BackendType::Local).unwrap();
        let filepath = PathBuf::from(base.log_path.clone())
            .join("messages.txt")
            .display()
            .to_string();
        let msg_file = RwLock::new(rt.block_on(tokio::fs::File::create(&filepath)).unwrap());
        Self { base, msg_file }
    }

    pub async fn save_message_lines(&self, lines: &Vec<AirbyteMessage>) -> Result<()> {
        let mut local_file = self.msg_file.write().await;

        for line in lines {
            let json_str = serde_json::to_string(line)?;
            local_file.write_all(json_str.as_bytes()).await?;
            local_file.write_all(b"\n").await?;
        }

        Ok(())
    }
}
#[async_trait]
impl Backend for LocalBackend {
    async fn push_logs(&self, logs: &Vec<String>) -> Result<()> {
        self.base.print_log_lines(logs);
        self.base.save_log_lines(logs).await
    }
    async fn push_messages(&self, messages: &Vec<AirbyteMessage>) -> Result<()> {
        self.save_message_lines(messages).await
    }
    async fn get_config(&self) -> Result<Value> {
        todo!();
    }
    fn get_result(&self) -> Arc<RwLock<ProcessResult>> {
        self.base.result.clone()
    }
    async fn push_result(&self) -> Result<()> {
        // TODO: save this end-result somewhere, as a message or a log action, can be used for debugging purposes
        Ok(()) // Nothing to push
    }
    async fn push_state(&self, _state: Option<serde_json::Value>) -> Result<()> {
        Ok(()) // Nothing to push
    }
    async fn get_state(&self) -> Result<serde_json::Value> {
        Ok(serde_json::Value::Null)
    }
    async fn extend_state_lock(&self) -> Result<u32> {
        Ok(u32::MAX)
    }
    async fn get_configured_catalog(&self) -> Result<Value> {
        todo!();
    }
    async fn state_lock_remaining_seconds(&self) -> Result<u32> {
        self.extend_state_lock().await
    }
    async fn flush(&self) -> Result<()> {
        Ok(()) // Nothing to flush
    }

    fn get_backend_type(&self) -> BackendType {
        BackendType::Local
    }
}

#[cfg(test)]
mod tests {
    // TODO: we also need tests for these implementations, mocking the backend if needed
}
