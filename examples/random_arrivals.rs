extern crate flow_rs;
use flow_rs::simulation::trader::Traders;
use flow_rs::controller::Controller;
use flow_rs::simulation::random_behavior::RandBehavior;

use std::sync::Arc;

pub fn main() {
    // Initialize the new Trader struct
    let traders = Arc::new(Traders::new());

    // Initialize the dispatcher controller
    let mut controller = Controller::new();

    // Establish the async tasks to repeatedly send orders over tcp
    let tcp_address = format!("127.0.0.1:5000");

    let tcp_arrivals = RandBehavior::tcp_arrival_interval(Arc::clone(&traders), 500, tcp_address.clone()); 
    let tcp_updates = RandBehavior::tcp_update_interval(Arc::clone(&traders), 1000, tcp_address.clone());
    let tcp_cancels = RandBehavior::tcp_cancel_interval(Arc::clone(&traders), 2000, tcp_address.clone());

    controller.push(tcp_arrivals);
    controller.push(tcp_updates);
    controller.push(tcp_cancels);


    // Establish the async tasks to repeatedly send orders over websocket
    env_logger::init();
    let ws_address: &'static str = "ws://127.0.0.1:3015";

    let ws_arrivals = RandBehavior::ws_arrival_interval(Arc::clone(&traders), 500, &ws_address); 
    let ws_updates = RandBehavior::ws_update_interval(Arc::clone(&traders), 1000, &ws_address);
    let ws_cancels = RandBehavior::ws_cancel_interval(Arc::clone(&traders), 2000, &ws_address);

    controller.push(ws_arrivals);
    controller.push(ws_updates);
    controller.push(ws_cancels);

    // Start the controller which will asynchronously dispatch the git push
    controller.run();
}

