use std::collections::VecDeque;

use rand::{distributions::Uniform, prelude::Distribution, thread_rng};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct data {
    symbol: serde_json::Value,
    price: serde_json::Value,
}

#[derive(Debug)]
pub struct Order {
    id: u64,
    price: f64,
    quantity: f32,
    buy: bool,
}

#[derive(Debug)]
struct Trade {
    buy_order_id: u64,
    sell_order_id: u64,
    price: f64,
    quantity: f64,
}
pub struct TradeBook {
    buy_order: VecDeque<Order>,
    sell_order: VecDeque<Order>,
    trade: Vec<Trade>,
    order_id: u64,
}
#[derive(Debug)]
pub struct Orderbook {
    buy_order: VecDeque<Order>,  //buying price
    sell_order: VecDeque<Order>, //asking price
    trade: Vec<Trade>,
    order_id: u64,
}
//adding, modifying, and matching
impl Orderbook {
    pub fn new() -> Self {
        Self {
            buy_order: VecDeque::new(),
            sell_order: VecDeque::new(),
            trade: Vec::new(),
            order_id: 1,
        }
    }
    fn adding(&mut self, price: f64, quantity: f32, buy: bool) -> u64 {
        let a = Order {
            id: self.order_id + 1,
            price,
            quantity,
            buy,
        };
        if buy {
            self.buy_order.push_back(a);
            self.buy_order
                .make_contiguous()
                .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        } else {
            self.sell_order.push_back(a);
            self.sell_order
                .make_contiguous()
                .sort_by(|a, b| a.price.partial_cmp(&b.price).unwrap());
        }

        self.matching();
        self.order_id
    }
    fn modifying(
        &mut self,
        order_id: usize,
        new_price: Option<f64>,
        new_quantity: Option<f64>,
    ) -> Result<String, String> {
        let mut found = false;

        if let Some(order) = self.buy_order.get_mut(order_id) {
            if let Some(price) = new_price {
                order.price = price;
            }
            if let Some(quantity) = new_quantity {
                order.quantity = quantity as f32;
            }
            println!("Buy Order {} modified.", order_id);
            self.matching();
            return Ok("order cahnged".to_string());
        }

        if let Some(order) = self.sell_order.get_mut(order_id) {
            if let Some(price) = new_price {
                order.price = price;
            }
            if let Some(quantity) = new_quantity {
                order.quantity = quantity as f32;
            }
            println!("Sell Order {} modified.", order_id);
            self.matching();
            return Ok("order cahnged".to_string());
        }

        Err("order id is not present".to_string())
    }

    /*The matching engine should simulate the process of matching buy orders with sell orders. When a buy order price is greater than
    or equal to the sell order price, the order is matched. The matching process should remove the matched orders from the order book,
    and the transaction price should be the sell price. */

    fn matching(&mut self) {
        while let (Some(buy), Some(sell)) = (self.buy_order.front(), self.sell_order.front()) {
            if buy.price >= sell.price {
                let matched_quantity = if buy.quantity < sell.quantity {
                    buy.quantity
                } else {
                    sell.quantity
                };
                self.trade.push(Trade {
                    buy_order_id: buy.id,
                    sell_order_id: sell.id,
                    price: sell.price,
                    quantity: matched_quantity as f64,
                });
                if buy.quantity > matched_quantity {
                    self.buy_order[0].quantity -= matched_quantity;
                } else {
                    self.buy_order.pop_front();
                }
                if sell.quantity > matched_quantity {
                    self.sell_order[0].quantity -= matched_quantity;
                } else {
                    self.sell_order.pop_front();
                }
            } else {
                break;
            }
        }
    }
    fn print_status(&self) {
        println!(" Order Book:");
        println!("Buy Orders:");
        for order in &self.buy_order {
            println!(
                "  ID: {}, Price: {:.2}, Quantity: {:.2}",
                order.id, order.price, order.quantity
            );
        }

        println!("Sell Orders:");
        for order in &self.sell_order {
            println!(
                "  ID: {}, Price: {:.2}, Quantity: {:.2}",
                order.id, order.price, order.quantity
            );
        }

        println!("Trades:");
        for trade in &self.trade {
            println!(
                "  Buy Order {} matched Sell Order {} at {:.2} for {:.2} BTC",
                trade.buy_order_id, trade.sell_order_id, trade.price, trade.quantity
            );
        }
        println!("--------------------------------------\n");
    }

    pub fn generate_fake_orders(&mut self, num_orders: usize) {
        let mut rng = thread_rng();
        let price_range = Uniform::new(40000.0, 50000.0);
        let quantity_range = Uniform::new(0.1, 2.0);

        for _ in 0..num_orders / 2 {
            let price = price_range.sample(&mut rng);
            let quantity = quantity_range.sample(&mut rng);
            let _ = self.adding(price, quantity as f32, true);
        }

        for _ in 0..num_orders / 2 {
            let price = price_range.sample(&mut rng);
            let quantity = quantity_range.sample(&mut rng);
            let _ = self.adding(price, quantity as f32, false);
        }

        self.print_status();
    }
}
#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let resp = reqwest::get("https://api.binance.com/api/v3/ticker/price?symbol=BTCUSDT")
        .await?
        .json::<data>()
        .await?;

    println!("{:?}", resp);

    let mut order_book = Orderbook::new();

    order_book.generate_fake_orders(20);

    order_book.matching();

    order_book.print_status();

    Ok(())
    // .json::<data>()
    // .await?;
}
