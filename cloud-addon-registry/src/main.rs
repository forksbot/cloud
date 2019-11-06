//#![deny(warnings)]
#![feature(proc_macro_hygiene, decl_macro)]

pub mod responder_type;
pub mod routes;

use cloud_addon_registry::create_rocket;
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use signal_hook::{iterator::Signals, SIGHUP, SIGINT, SIGKILL, SIGQUIT, SIGTERM};

/// Start rocket. A few states need to be initialized first.
pub fn main() -> Result<(), failure::Error> {
    stackdriver_logger::init_with_cargo!();

    let signals = match std::env::args().count() {
        1 => Signals::new(&[SIGINT, SIGTERM, SIGHUP, SIGQUIT]),
        _ => Signals::new(&[SIGINT, SIGKILL, SIGTERM, SIGHUP, SIGQUIT]),
    }?;

    std::thread::spawn(move || {
        for sig in signals.forever() {
            warn!("Received signal {:?}", sig);
            std::process::exit(1);
        }
    });
    create_rocket(5u32)?.launch();
    Ok(())
}
