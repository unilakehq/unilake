use core::panic;
use std::any::Any;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::Cursor;
use std::str::FromStr;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::vec::IntoIter;

use anyhow::anyhow;
use arrow2::array::Array;
use arrow2::array::MutableArray;
use arrow2::array::MutableBooleanArray;
use arrow2::array::MutableListArray;
use arrow2::array::MutablePrimitiveArray;
use arrow2::array::MutableStructArray;
use arrow2::array::MutableUtf8Array;
use arrow2::chunk::Chunk;
use arrow2::datatypes::DataType;
use arrow2::datatypes::Field;
use arrow2::datatypes::Schema;
use arrow2::io::parquet::write::transverse;
use arrow2::io::parquet::write::CompressionOptions;
use arrow2::io::parquet::write::Encoding;
use arrow2::io::parquet::write::FileWriter;
use arrow2::io::parquet::write::RowGroupIterator;
use arrow2::io::parquet::write::Version;
use arrow2::io::parquet::write::WriteOptions;
use log::debug;
use log::error;
use log::trace;
use opendal::Operator;
use serde_json::json;
use serde_json::Value;
use tokio::spawn;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use crate::model::AirbyteRecordMessage;
use crate::schema::SchemaGenerator;
use crate::schema::UNILAKE_AB_ID;
use crate::schema::UNILAKE_DATA;
use crate::schema::UNILAKE_EMITTED_AT;
use crate::utils;

macro_rules! deserialize_typeid_into {
    ($ty:ty, $target:ident, $value:expr) => {
        let items: &mut $ty = $target.downcast_mut().unwrap();
        items.push($value);
    };
}

/// Parquet file that needs to be created, automatically creates multiple parts when `max_file_size_kb` is reached
pub struct ParquetFile {
    pub path_destination: String,
    pub stream_name: String,
    pub max_file_size_kb: usize,
    pub processed_bytes: usize,
    pub processed_records: usize,
    pub processed_last_record: Arc<AtomicU64>,
    pub pending_bytes: Arc<AtomicUsize>,
    pub parts: HashMap<u16, Option<ParquetFilePart>>,
    current_part: u16,
    run_id: String,
    pub schema: Option<Schema>,
    normalization: Normalization,
    schema_generator: Arc<SchemaGenerator>,
    tx_record: Option<Sender<InnerMessage>>,
    rx_record: Receiver<InnerMessage>,
}

/// Current stream process information
pub struct StreamProcess {
    pub sender: Sender<InnerMessage>,
    pub pending_bytes: Arc<AtomicUsize>,
    pub processed_last_record: Arc<AtomicU64>,
}

/// Inner messages when processing a stream of data
#[derive(Debug)]
pub enum InnerMessage {
    /// Airbyte Record Message that should be processed
    Record(AirbyteRecordMessage),
    /// Flush the stream to the target destination
    Flush,
}

/// When normalizing the records, specify which type of normalization should be used
#[derive(Debug, PartialEq)]
pub enum Normalization {
    /// No normalization should be applied
    None,
    /// Normalization should be applied, based on the configured catalog
    Strict,
    /// Flatten the records, regardless of the configured catalog
    Flatten,
}

impl FromStr for Normalization {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Normalization::None),
            "strict" => Ok(Normalization::Strict),
            "flatten" => Ok(Normalization::Flatten),
            _ => Err("Invalid value for Normalization"),
        }
    }
}

pub struct ParquetFilePartRecord {
    pub data: Value,
    id: String,
    emitted_at: u64,
}

pub struct ParquetFilePart {
    pub file_name: String,
    pub file_size_b: usize,
    pub records: VecDeque<ParquetFilePartRecord>,
}

impl ParquetFile {
    pub fn new(
        path_destination: String,
        stream_name: String,
        max_file_size_kb: usize,
        run_id: String,
        normalization: Normalization,
        schema_generator: Arc<SchemaGenerator>,
    ) -> Self {
        let (tx_record, rx_record) = mpsc::channel::<InnerMessage>(100_000);
        Self {
            path_destination,
            stream_name,
            max_file_size_kb,
            processed_bytes: 0,
            processed_records: 0,
            processed_last_record: Arc::new(AtomicU64::new(0)),
            pending_bytes: Arc::new(AtomicUsize::new(0)),
            parts: HashMap::new(),
            current_part: 0,
            run_id,
            schema: None,
            normalization,
            schema_generator,
            tx_record: Some(tx_record),
            rx_record,
        }
    }

    pub fn get_process(&mut self) -> StreamProcess {
        StreamProcess {
            sender: self.tx_record.take().unwrap(),
            pending_bytes: self.pending_bytes.clone(),
            processed_last_record: self.processed_last_record.clone(),
        }
    }

    /// Start the process of processing records, takes ownership of self until the process is finished
    pub async fn start(mut self) -> JoinHandle<Result<Self, anyhow::Error>> {
        spawn(async move {
            trace!("Starting process for stream {}", self.stream_name);
            loop {
                tokio::select! {
                    Some(msg) = self.rx_record.recv() => {
                        match msg {
                            InnerMessage::Record(record) => {
                                if let Err(e) = self.process_record(record).await {
                                    error!("Error when processing stream {}: {}", self.stream_name, e);
                                }
                            }
                            InnerMessage::Flush => {
                                if let Err(e) = self.flush().await {
                                    error!("Error while flushing stream {}: {}", self.stream_name, e);
                                }
                            }
                        }
                    }
                    else => break, // exit loop when channel is closed
                }
            }

            trace!(
                "Finished processing messages for stream {}, flushing last data to storage.",
                self.stream_name
            );
            if let Err(e) = self.flush().await {
                error!("Error when flushing stream {}: {}", self.stream_name, e);
            }

            trace!("Done process for stream {}", self.stream_name);

            Ok(self)
        })
    }

    async fn process_record(&mut self, record: AirbyteRecordMessage) -> anyhow::Result<()> {
        trace!("record received: {:?}", &record);
        let data = if let Ok(data) = serde_json::to_vec(&record.data) {
            data
        } else {
            return Err(anyhow!("Failed to serialize data"));
        };
        let size = data.len();
        let id = Self::generate_hash_id(&data);

        self.processed_records += 1;
        self.processed_bytes += size;
        {
            self.processed_last_record
                .store(utils::get_current_time_in_seconds(), Ordering::Relaxed);
            self.pending_bytes.fetch_add(size, Ordering::Relaxed);
        }

        let part = self
            .parts
            .entry(self.current_part)
            .or_insert_with(|| {
                Some(ParquetFilePart {
                    file_name: format!(
                        "{}-{}-part-{}.parquet",
                        self.run_id, self.stream_name, self.current_part
                    ),
                    file_size_b: 0,
                    records: VecDeque::new(),
                })
            })
            .as_mut()
            .unwrap();
        part.file_size_b += size;
        part.records.push_back(ParquetFilePartRecord {
            data: record.data,
            id,
            emitted_at: record.emitted_at,
        });
        if (part.file_size_b / 1_000) >= self.max_file_size_kb {
            self.flush().await?;
        }

        Ok(())
    }

    fn generate_hash_id(s: &[u8]) -> String {
        let mut m = sha1_smol::Sha1::new();
        m.update(s);
        m.digest().to_string()
    }

    // Notes, datatypes: https://jorgecarleitao.github.io/arrow2/high_level.html
    fn create_file(&mut self, f: &mut ParquetFilePart) -> anyhow::Result<Cursor<Vec<u8>>> {
        let schema = self
            .schema
            .get_or_insert(match self.normalization {
                Normalization::None => SchemaGenerator::get_raw_schema(),
                Normalization::Strict => self
                    .schema_generator
                    .get_configured_schema(&self.stream_name)
                    .expect(&format!(
                        "Could not get configured schema for stream {}",
                        self.stream_name
                    )),
                Normalization::Flatten => SchemaGenerator::get_inferred_schema(&f),
            })
            .clone();

        let options = WriteOptions {
            write_statistics: true,
            version: Version::V2,
            compression: CompressionOptions::Snappy,
            data_pagesize_limit: None,
        };

        let encodings = schema
            .fields
            .iter()
            .map(|f| transverse(&f.data_type, |_| Encoding::Plain))
            .collect();

        let iter = vec![Ok(Self::deserialize_records(
            &mut f.records,
            &schema,
            &self.normalization,
        ))];
        let row_groups = RowGroupIterator::try_new(iter.into_iter(), &schema, options, encodings)?;

        let c = Cursor::new(Vec::new());
        let mut writer = FileWriter::try_new(c, schema, options)?;
        for group in row_groups {
            writer.write(group?)?;
        }
        let _size = writer.end(None)?;
        Ok(writer.into_inner())
    }

    fn deserialize_records(
        records: &mut VecDeque<ParquetFilePartRecord>,
        schema: &Schema,
        normalization: &Normalization,
    ) -> Chunk<Box<dyn Array>> {
        let mut results = schema
            .fields
            .iter()
            .map(|f| (f.name.as_str(), Self::allocate_array(f)))
            .collect::<HashMap<_, _>>();

        for mut v in records.drain(..) {
            let inner_data = if *normalization == Normalization::None {
                vec![
                    (UNILAKE_AB_ID, json!(v.id)),
                    (UNILAKE_EMITTED_AT, json!(v.emitted_at)),
                    (UNILAKE_DATA, v.data),
                ]
            } else {
                let data_iter = v
                    .data
                    .as_object_mut()
                    .unwrap()
                    .iter_mut()
                    .map(|(k, v)| (k.as_str(), v.take()))
                    .collect::<Vec<(_, _)>>();
                let data = vec![
                    (UNILAKE_AB_ID, json!(v.id)),
                    (UNILAKE_EMITTED_AT, json!(v.emitted_at)),
                ];
                data.into_iter().chain(data_iter.into_iter()).collect()
            };

            Self::deserialize_record(&mut inner_data.into_iter(), &mut results);
        }

        Chunk::new(results.into_values().map(|mut ma| ma.as_box()).collect())
    }

    fn add_value(target: &mut dyn Any, v: Value) {
        match v {
            Value::String(s) => {
                deserialize_typeid_into!(MutableUtf8Array<i32>, target, Some(s));
            }
            Value::Bool(b) => {
                deserialize_typeid_into!(MutableBooleanArray, target, Some(b));
            }
            Value::Number(n) => {
                if target.is::<MutablePrimitiveArray<i64>>() {
                    deserialize_typeid_into!(MutablePrimitiveArray<i64>, target, n.as_i64());
                } else if target.is::<MutablePrimitiveArray<u64>>() {
                    deserialize_typeid_into!(MutablePrimitiveArray<u64>, target, n.as_u64());
                } else {
                    deserialize_typeid_into!(MutablePrimitiveArray<f64>, target, n.as_f64());
                }
            }
            Value::Null => {
                if target.is::<MutablePrimitiveArray<i64>>() {
                    deserialize_typeid_into!(MutablePrimitiveArray<i64>, target, None);
                } else if target.is::<MutablePrimitiveArray<u64>>() {
                    deserialize_typeid_into!(MutablePrimitiveArray<u64>, target, None);
                } else if target.is::<MutablePrimitiveArray<f64>>() {
                    deserialize_typeid_into!(MutablePrimitiveArray<f64>, target, None);
                } else if target.is::<MutableUtf8Array<i32>>() {
                    deserialize_typeid_into!(MutableUtf8Array<i32>, target, None::<String>);
                } else if target.is::<MutableBooleanArray>() {
                    deserialize_typeid_into!(MutableBooleanArray, target, None);
                }
            }
            _ => unreachable!(),
        }
    }

    fn deserialize_record(
        values: &mut IntoIter<(&str, Value)>,
        results: &mut HashMap<&str, Box<dyn MutableArray>>,
    ) {
        for (k, v) in values {
            let results = results.get_mut(k).unwrap();
            let target = results.as_mut_any();
            if let Some(inner_data) =
                target.downcast_mut::<MutableListArray<i32, Box<dyn MutableArray>>>()
            {
                if let Value::Array(a) = v {
                    for v in a {
                        Self::add_value(inner_data.mut_values(), v);
                    }
                } else {
                    panic!("expected array");
                }
            } else if let Some(inner_data) = target.downcast_mut::<MutableStructArray>() {
                if let Value::Object(o) = v {
                    let mut item = 0;
                    for (_, v) in o {
                        let target = inner_data.mut_values()[item].as_mut_any();
                        Self::add_value(target, v);
                        item += 1;
                    }
                } else {
                    panic!("expected object");
                }
            } else {
                Self::add_value(target, v);
            }
        }
    }

    fn allocate_array(f: &Field) -> Box<dyn MutableArray> {
        match f.data_type() {
            DataType::Float64 => Box::new(MutablePrimitiveArray::<f64>::new()),
            DataType::UInt64 => Box::new(MutablePrimitiveArray::<u64>::new()),
            DataType::Int64 => Box::new(MutablePrimitiveArray::<i64>::new()),
            DataType::Boolean => Box::new(MutableBooleanArray::new()),
            DataType::Utf8 => Box::new(MutableUtf8Array::<i32>::new()),
            DataType::List(inner) if matches!(inner.data_type(), DataType::List(_)) => {
                Box::new(MutableListArray::<i32, _>::new_from(
                    Self::allocate_array(inner),
                    inner.data_type().clone(),
                    0,
                ))
            }
            DataType::List(inner) => Self::allocate_array(inner),
            DataType::Struct(f) => {
                let mut values = Vec::new();
                for i in 0..f.len() {
                    values.push(Self::allocate_array(&f[i]));
                }
                Box::new(MutableStructArray::new(DataType::Utf8, values))
            }
            _ => panic!("Unsupported data type {f:?}"),
        }
    }

    async fn send_file(file: Vec<u8>, file_destination: &std::path::Path) -> anyhow::Result<()> {
        let op = Operator::from_env(utils::get_target_fs())?;
        let o = op.object(file_destination.to_str().unwrap());
        o.write(file).await?;
        Ok(())
    }

    async fn flush(&mut self) -> anyhow::Result<()> {
        let pending_bytes = self.pending_bytes.load(Ordering::Relaxed);
        if pending_bytes == 0 {
            return Ok(());
        }

        debug!("Flushing data for stream {}", self.stream_name);

        let mut part = self
            .parts
            .get_mut(&self.current_part)
            .expect("Part not found")
            .take()
            .expect("Part already taken");

        let path_destination = {
            let mut path = std::path::PathBuf::new();
            path.push(&self.path_destination);
            path.push(&self.stream_name);
            path.push(&part.file_name);
            path
        };

        Self::send_file(self.create_file(&mut part)?.into_inner(), &path_destination).await?;

        self.current_part += 1;
        self.pending_bytes.store(0, Ordering::Relaxed);

        debug!("Done flushing data for stream {}", self.stream_name);
        Ok(())
    }
}
