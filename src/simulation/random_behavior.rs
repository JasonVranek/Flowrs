use crate::simulation::trader_behavior;
use crate::order::{Order};
use crate::simulation::trader::Traders;
use crate::exchange::order_processing::JsonOrder;
use crate::controller::Task;
use crate::io::tcp_json;
use crate::utility::get_time;

use std::sync::Arc;




pub struct RandBehavior {}


impl RandBehavior {
	// Generates a random number of new traders on a fixed interval over tcp
	pub fn arrival_interval(traders: Arc<Traders>, duration: u64, address: String) -> Task {
		Task::rpt_task(move || {
			// Make new random orders
	            let orders: Vec<Order> = trader_behavior::rand_enters(10);
	            println!("{} new arrivals!", orders.len());

	            // Send them over JSON
	            for order in &orders {
	                // Don't want a full clone of the order, just params to make json
	                let json_order = JsonOrder::order_to_json(order);
	                // Spawn the task to send json over tcp
	                // tokio::spawn(tcp_json::tcp_send_json(json_order, address.clone()));
	                let json_send_task = tcp_json::tcp_send_json(json_order, address.clone()).task;
                    tokio::spawn(json_send_task);
	            }
	            // Save new traders in the traders HashMap
	            traders.new_traders(orders);
	            println!("num_traders: {}", traders.num_traders());
	        }, duration)
	}

	// Updates a random number of existing traders on a fixed interval over tcp
	pub fn update_interval(traders: Arc<Traders>, duration: u64, address: String) -> Task {
		Task::rpt_task(move || {
			let rng_upper = 10;
            let update_orders = trader_behavior::gen_rand_updates(Arc::clone(&traders), rng_upper);
            println!("updating {} traders", update_orders.len());
            for order in update_orders {
            	let json_order = JsonOrder::params_to_json(order);
                // tokio::spawn(tcp_json::tcp_send_json(json_order, address.clone()));
                let json_send_task = tcp_json::tcp_send_json(json_order, address.clone()).task;
                tokio::spawn(json_send_task);
            }
		}, duration)
	}

	// Cancels a random number of existing traders on a fixed interval over tcp
	pub fn cancel_interval(traders: Arc<Traders>, duration: u64, address: String) -> Task {
		Task::rpt_task(move || {
			println!("cancel trader!");
            let rng_upper = 10;
            let cancel_orders = trader_behavior::gen_rand_cancels(Arc::clone(&traders), rng_upper);
            println!("cancelling {} traders", cancel_orders.len());
            for order in cancel_orders {
                println!("time: {:?}, cancelling: {:?} ", get_time(), order.0);
                let addr = address.clone();
                // Send a cancel message after a delay
                let send_cancel = Task::delay_task(move || {
                	let json_order = JsonOrder::params_to_json(order.clone());
                	let json_send_task = tcp_json::tcp_send_json(json_order, addr.clone()).task;
                    tokio::spawn(json_send_task);
                }, 1000).task;

                tokio::spawn(send_cancel);
            }
		}, duration)
	}
}










