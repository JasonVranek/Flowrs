use crate::io::trader::*;
use std::sync::{Mutex, Arc};
use tokio::prelude::*;
use tokio::timer::Interval;
use std::time::{Duration, Instant, SystemTime};

use crate::exchange::order_book::*;
use crate::io::order::*;
use crate::exchange::auction;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod io;
pub mod exchange;

pub enum State {
	Process,
	PreAuction,
	Auction,
}


pub fn setup() -> (Arc<Queue>, Arc<Book>, Arc<Book>, Arc<Mutex<State>>) {
	let queue = Arc::new(Queue::new());
	let bids_book = Arc::new(Book::new(TradeType::Bid));
	let asks_book = Arc::new(Book::new(TradeType::Ask));
	(queue, bids_book, asks_book, Arc::new(Mutex::new(State::Process)))
}

pub fn setup_orders() -> (Vec<Order>, Vec<Order>) {
	let mut bids = Vec::<Order>::new();
	let mut asks = Vec::<Order>::new();
	for i in 0..100 {
		bids.push(Order::new(
			gen_order_id(), 
    		OrderType::Enter, 
    		TradeType::Bid, 
    		i as f64, 
    		100.0, 
    		500.0,
    		p_wise_dem(i as f64, 100.0, 500.0),
		));
		asks.push(Order::new(
			gen_order_id(), 
    		OrderType::Enter, 
    		TradeType::Ask, 
    		i as f64, 
    		100.0, 
    		500.0,
    		p_wise_sup(i as f64, 100.0, 500.0),
		));
	}
	(bids, asks)
}

pub fn auction_interval(bids: Arc<Book>, asks: Arc<Book>, state: Arc<Mutex<State>>, duration: u64) 
-> Box<Future<Item = (), Error = ()> + Send> 
{
	let task = Interval::new(Instant::now(),  Duration::from_millis(duration))
	    .for_each(move |_| {
	    	{
	    		// Obtain lock on the global state and switch to Auction mode
	    		let mut state = state.lock().unwrap();
	    		*state = State::Auction;
	    	}
	    	let now = SystemTime::now();
	    	println!("Starting Auction");
	    	let cross_price = auction::bs_cross(Arc::clone(&bids), Arc::clone(&asks)).unwrap();
	    	let done = now.elapsed().unwrap().subsec_nanos();
	    	println!("Found Cross at @{} \nP = {}", done, cross_price);
	    	{
	    		// Change the state back to process to allow the books to be mutated again
	    		let mut state = state.lock().unwrap();
	    		*state = State::Process;
	    	}
	    	Ok(())
	    })
	    .map_err(|e| panic!("Auction Delay error; err={:?}", e));

	Box::new(task)
}

pub fn process_queue_interval(queue: Arc<Queue>, bids: Arc<Book>, asks: Arc<Book>, state: Arc<Mutex<State>>, duration: u64) 
-> Box<Future<Item = (), Error = ()> + Send> 
{
    let task = Interval::new(Instant::now(), Duration::from_millis(duration))
        .for_each(move |_| {
    		// Obtain lock on the global state and only process if in Process state
    		match *state.lock().unwrap() {
    			State::Process => {
    				let handles = conc_process_order_queue(Arc::clone(&queue), 
								Arc::clone(&bids),
								Arc::clone(&asks));

					for h in handles {
						h.join().unwrap();
					}
					println!("Processing order queue");
    			},
    			State::Auction => println!("Can't process order queue because auction!"),
    			State::PreAuction => println!("Can't process order queue because pre-auction!"),
			}
			Ok(())
        })
        .map_err(|e| println!("Error processing order iterval: {}", e));

    Box::new(task)
}


// A struct for providing stong types to deserialize the incoming JSONs
#[derive(Deserialize, Debug)]
struct JsonOrder{
	trader_id: String,        
    order_type: String,    
    trade_type: String,  
    p_low: f64,              
    p_high: f64, 
    u_max: f64,       
}

// Deserialize the JSON, create an Order type, and push onto the queue
pub fn process_new(msg: serde_json::Value, queue: Arc<Queue>) {
	// create Order from JSON
	let order = order_from_json(msg);

	if let Some(o) = order {
		// add message to queue with conc_recv_order()
		let handle = conc_recv_order(o, Arc::clone(&queue));
		handle.join().unwrap();
	} else {
		println!("Unsuccessful json parsing");
	}
}

// Make an Order from a JSON
pub fn order_from_json(msg: serde_json::Value) -> Option<Order> {
	let typed_json: JsonOrder = serde_json::from_value(msg).unwrap();
	// Parse JSON body into enums compatible with flow market
	let ot = match typed_json.order_type.to_lowercase().as_ref() {
		"enter" => OrderType::Enter,
		"update" => OrderType::Update,
		"cancel" => OrderType::Cancel,
		_ => {
			println!("Entered an invalid ordertype!");
			return None;
			},
	};

	let tt = match typed_json.trade_type.to_lowercase().as_ref() {
		"bid" => TradeType::Bid,
		"ask" => TradeType::Ask,
		_ => {
			println!("Entered an invalid tradetype");
			return None;
		},
	};

	Some(Order::new(
		typed_json.trader_id,
		ot, 
		tt, 
		typed_json.p_low, 
		typed_json.p_high, 
		typed_json.u_max,
		p_wise_dem(typed_json.p_low, typed_json.p_high, typed_json.u_max)
		))
}

















