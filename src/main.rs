extern crate flow_rs;

use flow_rs::io::order;
use flow_rs::io::trader;
use flow_rs::exchange::order_book;
use flow_rs::exchange::auction;

fn main() {
    order::test_order_mod();
    trader::test_trader_mod();
    order_book::test_order_book_mod();
    auction::test_auction_mod();
}
