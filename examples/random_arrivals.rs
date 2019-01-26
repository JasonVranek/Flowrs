extern crate flow_rs;
use flow_rs::simulation::trader::Traders;
use flow_rs::controller::Controller;
use flow_rs::simulation::random_behavior::RandBehavior;

use std::sync::Arc;

pub fn main() {
    // Initialize the new Trader struct
    let traders = Arc::new(Traders::new());

    let mut controller = Controller::new();

    let address = format!("127.0.0.1:6142");

    // Establish the async tasks to repeatedly send orders over tcp
    let arrivals = RandBehavior::arrival_interval(Arc::clone(&traders), 500, address.clone());
    let updates = RandBehavior::update_interval(Arc::clone(&traders), 1000, address.clone());
    let cancels = RandBehavior::cancel_interval(Arc::clone(&traders), 2000, address.clone());

    controller.push(arrivals);
    controller.push(updates);
    controller.push(cancels);

    // Start the tokio runtime which will repeatedly send orders over tcp
    controller.run();
}

