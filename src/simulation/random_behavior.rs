use crate::simulation::trader_behavior;
use crate::order::{Order};
use crate::simulation::trader::Traders;
use crate::exchange::order_processing::JsonOrder;
use crate::controller::AsyncTask;
use crate::io::tcp_json;


use std::sync::Arc;

use tokio::prelude::*;
use tokio::timer::{Interval, Delay};
use std::time::{Duration, Instant, SystemTime};


pub struct RandBehavior {}


impl RandBehavior {
	// Generates a random number of new traders on a fixed interval
	pub fn arrival_interval(traders: Arc<Traders>, duration: u64, address: String) -> AsyncTask {
	    let task = Interval::new(Instant::now(), Duration::from_millis(duration))
	        .for_each(move |_| {
	            // Make new random orders
	            let orders: Vec<Order> = trader_behavior::rand_enters(10);
	            println!("{} new arrivals!", orders.len());

	            // Send them over JSON
	            for order in &orders {
	                // Don't want a full clone of the order, just params to make json
	                let json_order = JsonOrder::order_to_json(order);
	                // Spawn the task to send json over tcp
	                tokio::spawn(tcp_json::tcp_send_json(json_order, address.clone()));
	            }
	            // Save new traders in the traders HashMap
	            traders.new_traders(orders);
	            println!("num_traders: {}", traders.num_traders());
	            Ok(())
	        })
	        .map_err(|e| println!("Error processing order iterval: {}", e));

	    Box::new(task)
	}

	// Updates a random number of existing traders on a fixed interval
	pub fn update_interval(traders: Arc<Traders>, duration: u64, address: String) -> AsyncTask {
	    let task = Interval::new(Instant::now(), Duration::from_millis(duration))
	        .for_each(move |_| {
	            let rng_upper = 10;
	            let update_orders = trader_behavior::gen_rand_updates(Arc::clone(&traders), rng_upper);
	            println!("updating {} traders", update_orders.len());
	            for order in update_orders {
	            	let json_order = JsonOrder::params_to_json(order);
	                tokio::spawn(tcp_json::tcp_send_json(json_order, address.clone()));
	            }
	            Ok(())
	        })
	        .map_err(|e| println!("Error processing order iterval: {}", e));

	    Box::new(task)
	}

	// Cancels a random number of existing traders on a fixed interval
	pub fn cancel_interval(traders: Arc<Traders>, duration: u64, address: String) -> AsyncTask {
	    let task = Interval::new(Instant::now(), Duration::from_millis(duration))
	        .for_each(move |_| {
	            println!("cancel trader!");
	            let rng_upper = 10;
	            let cancel_orders = trader_behavior::gen_rand_cancels(Arc::clone(&traders), rng_upper);
	            println!("cancelling {} traders", cancel_orders.len());
	            for order in cancel_orders {
	                println!("time: {:?}, cancelling: {:?} ", get_time(), order.0);
	                let addr = address.clone();
	                // Wait to send cancel in case enter hasn't been processed
	                tokio::spawn(Delay::new(Instant::now() + Duration::from_millis(1000))
	                    .and_then(|_| {
	                    	let json_order = JsonOrder::params_to_json(order);
	                        tokio::spawn(tcp_json::tcp_send_json(json_order, addr));
	                        Ok(())
	                    })
	                    .map_err(|e| panic!("delay errored; err={:?}", e)));
	                
	            }
	            Ok(())
	        })
	        .map_err(|e| println!("Error processing order iterval: {}", e));

	    Box::new(task)
	}
}

pub fn get_time() -> Duration {
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                         .expect("SystemTime::duration_since failed")
}








