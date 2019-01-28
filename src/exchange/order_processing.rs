use tokio::net::tcp::TcpStream;
use crate::order::{Order, OrderType, TradeType, p_wise_dem, p_wise_sup};
use crate::exchange::queue::Queue;

use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

extern crate serde;
extern crate serde_json;
extern crate tokio_serde_json;

use tokio::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use serde_json::Value;
use tokio_serde_json::{ReadJson, WriteJson};

// Handles JSON serialization/deserialization functions and new message processing
pub struct OrderProcessor {}


impl OrderProcessor {
	// Preprocess message in a new thread and append to queue
	// order is the trader's order that this function takes ownership of
	// queue is an Arc clone of the Queue stored on the heap
	pub fn conc_recv_order(order: Order, queue: Arc<Queue>) -> JoinHandle<()> {
	    thread::spawn(move || {
	    	// The add function acquires the lock
	    	queue.add(order);
	    })
	}
}

// Type alias for returning JSON stream
type DeserializedStream = ReadJson<FramedRead<TcpStream, LengthDelimitedCodec>, serde_json::Value>;
type SerializedStream = WriteJson<FramedWrite<TcpStream, LengthDelimitedCodec>, serde_json::Value>;

// A struct for providing stong types to deserialize the incoming JSONs
#[derive(Deserialize, Debug)]
pub struct JsonOrder{
	trader_id: String,        
    order_type: String,    
    trade_type: String,  
    p_low: f64,              
    p_high: f64, 
    u_max: f64,       
}

impl JsonOrder {
	pub fn serializer(socket: TcpStream) -> SerializedStream{
		// Delimit frames using a length header
	    let length_delimited = FramedWrite::new(socket, LengthDelimitedCodec::new());

	    // Serialize frames
	    let serializer = WriteJson::new(length_delimited);

	    serializer
	}

	pub fn deserialize(socket: TcpStream) ->  DeserializedStream {
		// Delimit frames using a length header
	    let length_delimited = FramedRead::new(socket, LengthDelimitedCodec::new());

	    // Deserialize frames
	    let deserialized = ReadJson::<_, Value>::new(length_delimited);

	    deserialized
	}
	// Deserialize the JSON, create an Order type, and push onto the queue
	pub fn process_new(msg: serde_json::Value, queue: Arc<Queue>) {
		// create Order from JSON
		let order = JsonOrder::order_from_json(msg);

		if let Some(o) = order {
			// add message to queue with conc_recv_order()
			let handle = OrderProcessor::conc_recv_order(o, Arc::clone(&queue));
			handle.join().unwrap();
		} else {
			println!("Unsuccessful json parsing");
		}
	}

	// Make an Order from a JSON
	fn order_from_json(msg: serde_json::Value) -> Option<Order> {
		let typed_json: JsonOrder = serde_json::from_value(msg).expect("Couldn't make JSON");
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

		let func = match tt {
			TradeType::Bid => p_wise_dem(typed_json.p_low, typed_json.p_high, typed_json.u_max),
			TradeType::Ask => p_wise_sup(typed_json.p_low, typed_json.p_high, typed_json.u_max),
		};

		Some(Order::new(
			typed_json.trader_id,
			ot, 
			tt, 
			typed_json.p_low, 
			typed_json.p_high, 
			typed_json.u_max,
			func,
			))
	}

	// Turn an order into JSON from its params
	pub fn order_to_json(order: &Order) -> serde_json::Value {
		let ot = match order.order_type {
            OrderType::Enter => "enter",
            OrderType::Update => "update",
            OrderType::Cancel => "cancel",
        };

        let tt = match order.trade_type {
            TradeType::Bid => "bid",
            TradeType::Ask => "ask",
        };

		json!({
                "trader_id": order.trader_id.clone(),
                "order_type": ot,
                "trade_type": tt,
                "p_low": order.p_low.clone(),
                "p_high": order.p_high.clone(),
                "u_max": order.u_max.clone(),
            })
	}

	pub fn params_to_json(order_params: (String, OrderType, TradeType, f64, f64, f64)) 
	-> serde_json::Value {
		let (t_id, ot, tt, pl, ph, u) = order_params;

		let ot = match ot {
            OrderType::Enter => "enter",
            OrderType::Update => "update",
            OrderType::Cancel => "cancel",
        };

        let tt = match tt {
            TradeType::Bid => "bid",
            TradeType::Ask => "ask",
        };

		json!({
                "trader_id": t_id,
                "order_type": ot,
                "trade_type": tt,
                "p_low": pl,
                "p_high": ph,
                "u_max": u,
            })
	}
}




