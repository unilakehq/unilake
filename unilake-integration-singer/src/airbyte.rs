// TODO: create an integration test for this module to make sure this process works as expected. Can make use of the random source
use std::cmp::max;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::error::Error;
use std::io::BufRead;
use std::io::BufReader;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use duct::cmd;
use log::debug;
use log::error;
use log::info;
use log::trace;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio::time::interval;

use crate::backend::Backend;
use crate::flatten::Flattener;
use crate::model::*;
use crate::parquet::InnerMessage;
use crate::parquet::Normalization;
use crate::parquet::ParquetFile;
use crate::parquet::StreamProcess;
use crate::schema::SchemaGenerator;
use crate::utils;
use crate::AirbyteCommand;

pub struct AirbyteSource {
    run_id: String,
    normalize: Normalization,
    destination_dir: String,
    max_pending_bytes: usize,
}

impl Default for AirbyteSource {
    fn default() -> Self {
        AirbyteSource::new()
    }
}

impl AirbyteSource {
    pub fn new() -> Self {
        Self {
            run_id: env::var("UNILAKE_RUN_ID")
                .unwrap_or(format!("{}", utils::get_current_time_in_seconds())),
            normalize: env::var("UNILAKE_NORMALIZE")
                .unwrap_or("None".to_string())
                .parse::<Normalization>()
                .expect("UNILAKE_NORMALIZE should be either true or false"),
            destination_dir: env::var("UNILAKE_DESTINATION_DIR")
                .expect("UNILAKE_DESTINATION_DIR environment variable is not defined"),
            max_pending_bytes: {
                max(
                    if let Ok(system_memory) = sys_info::mem_info() {
                        (system_memory.total as f64 * 0.5) as usize
                    } else {
                        0
                    },
                    2e+9 as usize, // 2GB
                )
            },
        }
    }

    pub async fn execute(
        airbyte_command: AirbyteCommand,
        schema_generator: Arc<SchemaGenerator>,
        backend: Arc<Box<dyn Backend>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut handles = Vec::with_capacity(3);
        let (tx_process, rx_process) = mpsc::channel::<AirbyteMessage>(100);
        let (tx_data, rx_data) = mpsc::channel::<(Normalization, AirbyteRecordMessage)>(100);

        let wrapped = Arc::new(AirbyteSource::new());
        let this = wrapped.clone();

        let entrypoint = env::var("AIRBYTE_ENTRYPOINT")?;
        info!("Using entrypoint: {}", entrypoint);
        info!("Normalizing data: {:?}", wrapped.normalize);

        handles.push(tokio::spawn(async move {
            let args = match airbyte_command {
                AirbyteCommand::Spec => {
                    info!("Executing Spec Command");
                    vec!["spec".to_owned()]
                }
                AirbyteCommand::Check { config } => {
                    info!("Executing Check Command");

                    vec![
                        "check".to_owned(),
                        "--config".to_owned(),
                        config.to_str().unwrap().to_owned(),
                    ]
                }
                AirbyteCommand::Discover { config } => {
                    info!("Executing Discover Command");

                    vec![
                        "discover".to_owned(),
                        "--config".to_owned(),
                        config.to_str().unwrap().to_owned(),
                    ]
                }
                AirbyteCommand::Read {
                    config,
                    catalog,
                    state,
                } => {
                    info!("Executing Read Command");
                    let mut items = vec![
                        "read".to_owned(),
                        "--config".to_owned(),
                        config.to_str().unwrap().to_owned(),
                        "--catalog".to_owned(),
                        catalog.to_str().unwrap().to_owned(),
                    ];
                    if let Some(state) = state {
                        items.splice(
                            1..1,
                            vec!["--state".to_owned(), state.to_str().unwrap().to_owned()],
                        );
                    }
                    items
                }
            };

            let entrypoint: Vec<&str> = entrypoint.split_whitespace().collect();
            let computed_args = {
                let mut computed_args = entrypoint[1..].to_vec();
                for r in &args {
                    computed_args.push(r.as_str());
                }
                computed_args
            };

            debug!(
                "Executing command {} with args {:?}",
                entrypoint[0], computed_args
            );
            let exec_cmd = cmd(entrypoint[0], computed_args);
            let reader = exec_cmd.stderr_to_stdout().reader().unwrap();
            let lines = BufReader::new(reader).lines();
            for line in lines {
                if this
                    .process(line, &this.normalize, tx_process.clone(), tx_data.clone())
                    .await
                    .is_err()
                {
                    break;
                }
            }
            debug!("Done executing command");
            tx_process.downgrade();
            tx_data.downgrade();
        }));
        let this = wrapped.clone();
        let backend_msg = backend.clone();
        handles.push(tokio::spawn(async move {
            this.process_message(rx_process, backend_msg).await.unwrap();
        }));
        let this = wrapped.clone();
        handles.push(tokio::spawn(async move {
            this.process_data(rx_data, schema_generator, backend)
                .await
                .unwrap();
        }));
        futures::future::join_all(handles).await;
        Ok(())
    }

    async fn process<T>(
        &self,
        msg: Result<String, T>,
        normalize: &Normalization,
        tx_process: mpsc::Sender<AirbyteMessage>,
        tx_data: mpsc::Sender<(Normalization, AirbyteRecordMessage)>,
    ) -> Result<(), Box<dyn Error>>
    where
        T: Error + 'static,
    {
        match msg {
            Ok(data) => {
                let v: AirbyteMessage = match serde_json::from_str(&data) {
                    Ok(v) => v,
                    Err(e) => AirbyteMessage {
                        t: AirbyteMessageType::LOG,
                        log: Option::from(AirbyteLogMessage {
                            level: AirbyteLogLevel::ERROR,
                            message: format!("Could not parse data: {data}. Due to error: {e}"),
                            stack_trace: None,
                        }),
                        ..Default::default()
                    },
                };

                match v.t {
                    AirbyteMessageType::RECORD => {
                        let mut record = v.record.unwrap();
                        let record_raw = serde_json::Value::String(record.data.to_string());

                        if *normalize == Normalization::Flatten {
                            let v = record.data.take();
                            let flattened_records = Flattener::new(record.stream.to_string(), None)
                                .flatten(v)
                                .unwrap();
                            for s in flattened_records {
                                for r in s.1 {
                                    tx_data
                                        .send((
                                            Normalization::Flatten,
                                            AirbyteRecordMessage {
                                                stream: s.0.to_string(),
                                                data: r,
                                                emitted_at: record.emitted_at,
                                                namespace: record.namespace.clone(),
                                            },
                                        ))
                                        .await?;
                                }
                            }
                        } else if *normalize == Normalization::Strict {
                            tx_data
                                .send((
                                    Normalization::Strict,
                                    AirbyteRecordMessage {
                                        stream: record.stream.to_string(),
                                        data: record.data.take(),
                                        emitted_at: record.emitted_at,
                                        namespace: record.namespace.clone(),
                                    },
                                ))
                                .await?;
                        }
                        record.data = record_raw;
                        record.stream = format!("{}_raw", record.stream);
                        tx_data.send((Normalization::None, record)).await?;
                    }
                    _ => {
                        tx_process.send(v).await?;
                    }
                }
            }
            Err(err) => {
                error!("Error while processing messages: {}", err);
                return Err(Box::new(err));
            }
        };
        Ok(())
    }

    fn get_utc_timestamp() -> String {
        let now: DateTime<Utc> = Utc::now();
        now.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    async fn process_message(
        &self,
        mut rx_process: mpsc::Receiver<AirbyteMessage>,
        backend: Arc<Box<dyn Backend>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut buff = VecDeque::<AirbyteMessage>::new();
        let mut interval = interval(Duration::from_secs(2));
        let mut state = None;

        loop {
            tokio::select! {
                Some(msg) = rx_process.recv() => {
                    buff.push_back(msg);
                }
                _ = interval.tick() => {
                    let items: Vec<AirbyteMessage> = buff.drain(..).collect();
                    let mut abmsgs = Vec::new();
                    let mut logmsgs = Vec::new();
                    for i in items {
                        match i.t {
                            AirbyteMessageType::LOG => {
                                let i = i.log.as_ref().unwrap();
                                let msg = format!("{} {:?} {:?} {}", Self::get_utc_timestamp(), i.level, i.stack_trace, i.message);
                                logmsgs.push(msg);
                            }
                            AirbyteMessageType::STATE => {
                                state = i.state.map(|s| s.data);
                            }
                            _ => {
                                abmsgs.push(i)
                            }
                        }
                    }
                    backend.push_logs(&logmsgs).await?;
                    backend.push_messages(&abmsgs).await?;

                    // Check state lock
                    if backend.state_lock_remaining_seconds().await? < 20 {
                        if let Err(err) = backend.extend_state_lock().await {
                            panic!("Failed to extend state lock due to error: {}", err);
                        }
                    }
                }
                else => {
                    break;
                }
            }
        }
        if state.is_some() {
            backend.push_state(state).await?;
        }
        backend.flush().await?;

        Ok(())
    }

    async fn process_data(
        &self,
        mut rx_data: mpsc::Receiver<(Normalization, AirbyteRecordMessage)>,
        schema_generator: Arc<SchemaGenerator>,
        backend: Arc<Box<dyn Backend>>,
    ) -> Result<(), Box<dyn Error>> {
        let mut streams = HashMap::<String, StreamProcess>::new();
        let mut set = JoinSet::new();

        while let Some(i) = rx_data.recv().await {
            let r = i.1;
            if !streams.contains_key(&r.stream) {
                let mut file = ParquetFile::new(
                    self.destination_dir.to_string(),
                    r.stream.clone(),
                    100_000,
                    self.run_id.clone(),
                    i.0,
                    schema_generator.clone(),
                );
                streams.insert(r.stream.clone(), file.get_process());
                set.spawn(file.start().await);
            }
            let stream = &streams[&r.stream];
            stream.sender.send(InnerMessage::Record(r)).await?;

            let pending_bytes: usize = streams
                .values()
                .map(|s| s.pending_bytes.load(Ordering::Relaxed))
                .sum();

            if self.max_pending_bytes > pending_bytes {
                continue;
            }

            let mut stream_values = streams.values().collect::<Vec<_>>();
            stream_values.sort_by(|x, y| {
                x.processed_last_record
                    .load(Ordering::Relaxed)
                    .cmp(&y.processed_last_record.load(Ordering::Relaxed))
            });
            let stream = stream_values[0];
            stream.sender.send(InnerMessage::Flush).await?;
        }
        trace!("Done processing data, dropping streams");
        drop(streams);

        let mut results = Vec::new();
        while let Some(res) = set.join_next().await {
            if let Ok(f) = res?.unwrap() {
                results.push(ProcessResultDataProcessed {
                    stream: f.stream_name.clone(),
                    schema: f.schema,
                    records: f.processed_records,
                    bytes: f.processed_bytes,
                    parts_num: f.parts.len(),
                    parts_path: Vec::new(),
                });
                backend
                    .push_logs(&vec![format!(
                        "Processed ({}): {} kB - {} records",
                        f.stream_name,
                        (f.processed_bytes / 1000),
                        f.processed_records
                    )])
                    .await?;
            }
        }
        {
            let r = backend.get_result();
            let mut s = r.write().await;
            s.data_processed = Some(results);
        }
        Ok(())
    }
}
