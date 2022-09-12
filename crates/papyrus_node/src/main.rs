use log::info;
use papyrus_gateway::run_server;
use papyrus_monitoring_gateway::run_server as monitoring_run_server;
use papyrus_node::config::load_config;
use papyrus_storage::open_storage;
use papyrus_sync::{CentralSource, StateSync};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("Booting up.");

    let config = load_config("config/config.ron")?;

    let (storage_reader, storage_writer) = open_storage(config.storage.db_config)?;

    // Network interface.
    let central_source = CentralSource::new(config.central)?;

    // Sync.
    let mut sync =
        StateSync::new(config.sync, central_source, storage_reader.clone(), storage_writer);
    let sync_thread = tokio::spawn(async move { sync.run().await });

    // Pass reader to storage.
    let (_, server_handle) = run_server(config.gateway, storage_reader.clone()).await?;
    let (_, monitoring_server_handle) =
        monitoring_run_server(config.monitoring_gateway, storage_reader.clone()).await?;
    let (_, _, sync_thread_res) =
        tokio::join!(server_handle, monitoring_server_handle, sync_thread);
    sync_thread_res??;

    Ok(())
}
