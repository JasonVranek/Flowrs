pub mod io;
pub mod exchange;
pub mod simulation;

use crate::exchange::auction;
use crate::exchange::order_book::*;
use crate::exchange::order::*;
use crate::exchange::order_processing::*;
use crate::exchange::queue_processing::*;
use crate::exchange::queue::*;

use std::sync::{Mutex, Arc};
use tokio::prelude::*;
use tokio::timer::Interval;
use std::time::{Duration, Instant, SystemTime};
use tokio::net::{TcpListener};

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate tokio_serde_json;

use tokio::codec::{FramedRead, LengthDelimitedCodec};
use serde_json::Value;
use tokio_serde_json::ReadJson;


#[derive(Debug)]
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

pub fn auction_interval(bids: Arc<Book>, asks: Arc<Book>, state: Arc<Mutex<State>>, duration: u64) 
-> Box<Future<Item = (), Error = ()> + Send> 
{
	let task = Interval::new(Instant::now(),  Duration::from_millis(duration))
	    .for_each(move |_| {
	    	{
	    		// Obtain lock on the global state and switch to Auction mode, will stop
	    		// the queue from being processed.
	    		let mut state = state.lock().unwrap();
	    		*state = State::Auction;
	    	}
	    	println!("Starting Auction @{:?}", get_time());
	    	if let Some(cross_price) = auction::bs_cross(Arc::clone(&bids), Arc::clone(&asks)) {
	    		println!("Found Cross at @{:?} \nP = {}\n", get_time(), cross_price);
	    	} else {
	    		println!("Error, Cross not found\n");
	    	}
	    	
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
					// println!("Processing order queue");
    			},
    			State::Auction => println!("Can't process order queue because auction!"),
    			State::PreAuction => println!("Can't process order queue because pre-auction!"),
			}
			Ok(())
        })
        .map_err(|e| println!("Error processing order iterval: {}", e));

    Box::new(task)
}

pub fn tcp_server(queue: Arc<Queue>, address: String) -> Box<Future<Item = (), Error = ()> + Send> 
{	
	 // Bind a TcpListener to a local port
	let addr = address.parse().unwrap();
	let listener = TcpListener::bind(&addr).unwrap();

	// start a tcp server that accepts JSON objects 
	let tcp_server = listener.incoming().for_each(move |socket| {
		// Clone the queue into the closure
		let queue = Arc::clone(&queue);

		// Delimit frames using a length header
        let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());

        // Deserialize frames
        let deserialized = ReadJson::<_, Value>::new(length_delimited)
            .map_err(|e| println!("ERR: {:?}", e));

        // Spawn a task that converts JSON to an Order and adds to queue
        tokio::spawn(deserialized.for_each(move |msg| {
            // println!("GOT: {:?} @ {:?}", msg, flow_rs::get_time());
            process_new(msg, Arc::clone(&queue));
            Ok(())
        }));

        Ok(())
    })
    .map_err(|_| ());


	println!("Running server on localhost:6142");
    

    Box::new(tcp_server)
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


pub fn get_time() -> Duration {
	SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
                         .expect("SystemTime::duration_since failed")
}















