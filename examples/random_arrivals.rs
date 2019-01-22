extern crate flow_rs;
use flow_rs::simulation::trader::Traders;
use flow_rs::controller::Controller;
use flow_rs::simulation::random_behavior::RandBehavior;

// extern crate futures;
// extern crate tokio;
// extern crate tokio_serde_json;
// #[macro_use]
// extern crate serde_json;

use std::sync::Arc;

pub fn main() {
    // Initialize the new Trader struct
    let traders = Arc::new(Traders::new());

    let mut tasks = Vec::new();

    let address = format!("127.0.0.1:6142");

    // Establish the tokio::Interval's to repeatedly send orders
    let arrivals = RandBehavior::arrival_interval(Arc::clone(&traders), 500, address.clone());
    let updates = RandBehavior::update_interval(Arc::clone(&traders), 1000, address.clone());
    let cancels = RandBehavior::cancel_interval(Arc::clone(&traders), 2000, address.clone());

    tasks.push(arrivals);
    tasks.push(updates);
    tasks.push(cancels);

    // Start the tokio runtime which will repeatedly send orders over tcp
    Controller::run(tasks);
}
