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

