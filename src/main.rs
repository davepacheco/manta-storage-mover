/*
 * src/main.rs: driver program for playing with dependencies
 */

#[macro_use]
extern crate slog;
extern crate slog_bunyan;

use slog::Drain;
use std::sync::Mutex;

fn main() {
    let log_metadata = o!(
        "name" => "storage-mover"
    );
    let log_destination = std::io::stdout();
    let log_drain = Mutex::new(slog_bunyan::default(log_destination)).fuse();
    let log = slog::Logger::root(log_drain, log_metadata);

    info!(log, "test message!");
}
