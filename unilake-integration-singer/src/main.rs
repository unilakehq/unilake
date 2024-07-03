use std::env;
use std::fs;
use std::os::linux::raw::stat;
use std::sync::Arc;

use bmi::airbyte::AirbyteSource;
use bmi::backend::BackendFactory;
use bmi::docker::Docker;
use bmi::model::ProcessResultStatus;
use bmi::schema::SchemaGenerator;
use bmi::utils;
use bmi::AirbyteCommand;
use bmi::Cli;
use bmi::Command;
use clap::Parser;

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Cli::parse();
    match args.command {
        Command::Build {
            source_image,
            target_image,
            base_image,
        } => {
            Docker::build_image(source_image, target_image, base_image);
        }
        Command::Airbyte(cmd) => {
            let backend = Arc::new(BackendFactory::from_env().unwrap());
            {
                let r = backend.get_result();
                let mut s = r.write().await;
                s.process_start = utils::get_current_time_in_seconds();
                s.process_source_id = env::var("PROCESS_SOURCE_ID").unwrap_or_else("unknown");
                s.process_run_id = env::var("PROCESS_RUN_ID").unwrap_or_else("unknown");
            }

            let (_, configured_catalog, _) = match &cmd {
                AirbyteCommand::Read {
                    config,
                    catalog,
                    state,
                } => {
                    let backend_config = backend.get_config().await?;
                    utils::save_json_to_file(&backend_config, config);
                    let backend_state = backend.get_state().await?;
                    if !backend_state.is_null() && state.is_some() {
                        utils::save_json_to_file(&backend_state, state.into());
                    }
                    let backend_catalog = backend.get_catalog().await?;
                    utils::save_json_to_file(&backend_catalog, catalog);
                    (backend_config, backend_catalog, backend_state)
                }
                _ => serde_json::Value::Null,
            };
            let schema_generator = Arc::new(SchemaGenerator::new(configured_catalog));

            match AirbyteSource::execute(cmd, schema_generator, backend.clone()).await {
                Ok(()) => backend
                    .push_logs(&vec!["Airbyte command executed successfully".to_string()])
                    .await
                    .unwrap(),
                Err(e) => {
                    let msg = format!("Error executing Airbyte command: {e}");
                    {
                        let r = backend.get_result();
                        let mut s = r.write().await;
                        s.status = ProcessResultStatus::FAILED;
                        s.error_message = Some(msg.clone());
                    }
                    backend.push_logs(&vec![msg]).await.unwrap();
                }
            }
            {
                let r = backend.get_result();
                let mut s = r.write().await;
                s.process_end = utils::get_current_time_in_seconds();
            }
            if let Err(e) = backend.flush().await {
                backend
                    .push_logs(&vec![format!("Error flushing backend: {e}")])
                    .await
                    .unwrap();
            }
            if let Err(e) = backend.push_result().await {
                backend
                    .push_logs(&vec![format!("Error pushing results: {e}")])
                    .await
                    .unwrap();
            }
        }
    }
}
