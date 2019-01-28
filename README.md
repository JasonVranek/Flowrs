# Flowrs
Flow Market implemented in Rust based on the paper by **Kyle and Lee**:

"Kyle, Albert (Pete) S. and Lee, Jeongmin, Toward a Fully Continuous Exchange (July 4, 2017)"

The purpose of this project is to explore the potential benefits of this new market design in dealing with the problems associated with high-frequency trading, and to test this market's viability as a decentralized exchange that leverages Frequent Batch Auctions (FBA) in a blockchain environment.

This was also largely to serve as practice in implementing a server in Rust. Libraries like rayon were used to quickly and safely compute the market clearing price over hundreds of orders in parallel. I wrote a wrapper around Tokio to easily dispatch asynchronous tasks that can safely mutate a shared state across many threads, as well as accept hundreds of incoming connections simultaneously. 

My inelegant prototype version written in Python as a way to learn OOP can be found here: <https://github.com/JasonVranek/Flow>

Email me at <jasonvranek@gmail.com>

### Usage
By default the exchange accepts TCP connections on localhost:5000 and WebSocket connections via localhost:3015.
- Setup Rust: <https://www.rust-lang.org/tools/install>
- Make sure binary is compiled to your operating system, with "cargo build".
- Run "cargo run" in one terminal to start the Flow Market exchange server.
- Run "cargo run --example random_arrivals" in another terminal to start a simulation that sends random trader events (Enter, Update, and Cancel orders) to the exchange server. 

It is possible to run your own simulation by sending JSON orders over either 'localhost:5000' for TCP, or 
'ws://localhost:3015' for a websocket stream. Orders must adhere to the following JSON format:

Order JSON format:
{
	"trader_id": String,
	"order_type": String,
	"trade_type": String,
	"p_low": f64,
	"p_high": f64,
	"u_max": f64,
}
where **order_type** is "enter", "update", or "cancel".
where **trade_type** is "bid" or "ask".

### Modules
#### Simulation Module: 
- Responsible for random trader behavior: entering, updating, and cancelling orders within the exchange.
- **Traders**: a data structure to keep track of the orders that have been sent and received by the exchange.

#### IO Module:
- Allows two-way communication to and from the exchange through TCP and websockets.

#### Controller Module:
- A wrapper around Tokio.
- Allows asynchronous tasks to be generated from a single closure without having to worry about getting the Future types in the right format.
- Capable of generating one-off tasks, delayed tasks, and repeated tasks on an interval. 

#### Exchange Module:
The exchange receives orders in JSON format over a communication method found in the IO module. The JSON is parsed and converted to an internal Order struct, and added to a Queue. The Queue is processed on an interval, and by default an auction will occur every 3000ms.
- Data Structures:
		- Order: internal data structure for running auctions
		- Queue: FIFO queue for buffering incoming orders
		- Order Book: threadsafe holder of bids and asks
- Submodules:
		- Order Processor: Front-facing input to the exchange. Asynchronously receives orders in JSON format and converts it to internal Order data structure. The order is then pushed onto a  Queue that is shared among different threads.
		- Queue Processor: Periodically drains the order queue and processes each order across multiple threads. Each order either Enters, Updates, or Cancels an order in the respective bids or asks book.
		- Auction: A module to calculate the market clearing price for two given bid and ask order books. Uses parallel iterators to quickly calculate each order's custom closure safely in parallel to find the market clearing price.





