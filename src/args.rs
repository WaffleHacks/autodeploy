use std::{net::SocketAddr, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "autodeploy", about = "Automatically deploy GitHub repos")]
pub struct Args {
    /// The listen address and port
    ///
    /// The port and address where the server should listen to receive webhooks
    #[structopt(short, long)]
    pub address: Option<SocketAddr>,

    /// The configuration file location
    ///
    /// Where the configuration file should be loaded from. The environment
    /// variable AUTODEPLOY_CONFIG can also be used.
    #[structopt(short, long, env = "AUTODEPLOY_CONFIG", default_value = "config.toml")]
    pub config: PathBuf,

    /// The minimum level to log at
    ///
    /// The minimum log level specification, supports the rust log format. The
    /// environment variable RUST_LOG can also be used.
    #[structopt(short, long, env = "RUST_LOG")]
    pub log_level: Option<String>,
}
