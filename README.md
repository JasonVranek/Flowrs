# Flowrs
Flow Market implemented in Rust

Exchange runs on localhost:6142
- Make sure binary is compiled to your operating system.
- Run "cargo run" in one terminal to start the Flow exchange.
- Run "cargo run --example random_arrivals" in another terminal to start a simulation that sends random trader behavior over TCP.


Order JSON format:
{
	"trader_id": <String>,
	"order_type": <String>,
	"trade_type": <String>,
	"p_low": <f64>,
	"p_high": <f64>,
	"u_max": <f64>,
}

Modules:

Simulation Module: 
	Responsible for trader behavior: entering, updating, and cancelling orders within the exchange.
	Accepts an communication source from the Communication module to interact with the exchange. 
	Implementations for a real exchange will follow the same implementations as a simulation.
	- Submodules:
		- Traders: a data structure to keep track of the orders that have been sent and received by the exchange.

Communication Module:
	Allows two-way communication to and from the exchange through TCP and websockets.

Exchange Module:
	- A flow market implementation. The exchange receives orders in JSON format, and converts it 
	- Data Structures:
		- Order
		- Queue:
		- Order Book:
	- Submodules:
		- Order Processor: Front-facing input to the exchange. Asynchronously receives orders in JSON format
						   and converts it to internal Order data structure. The order is then pushed onto a
						   Queue that is shared amongst different threads.
		- Queue Processor: Periodically drains the order queue and processes each order across multiple threads.
						   Each order either Enters, Updates, or Cancels an order in the respective bids or asks book.
		- Auction: A module to calculate the market clearing price for two given bid and ask order books. 





