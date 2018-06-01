// extern crate mysql;
// fn main() {
//     // user:pass@tcp(localhost:3306)/cas?charset=utf8&parseTime=false
//     let mut opts = mysql::OptsBuilder::from_opts("mysql://user:pass@localhost:3306/cas");
//     opts.stmt_cache_size(0);
//     opts.prefer_socket(false);
//     let mut conn = mysql::Conn::new(opts).unwrap();

//     loop {
//         let start = ::std::time::Instant::now();
//         conn.prepare("SELECT 1").unwrap();
//         println!("{:?}", start.elapsed());
//     }
// }

#[macro_use]
extern crate mysql;
extern crate redis;
extern crate log;
// extern crate slog;
// extern crate slog_term;

use mysql as my;
use redis::Commands;
// use slog::DrainExt;

#[derive(Debug, PartialEq, Eq)]
struct Payment {
    customer_id: i32,
    amount: i32,
    account_name: Option<String>,
}

fn fetch_an_integer() -> redis::RedisResult<isize> {
    // connect to redis
    let client = try!(redis::Client::open("redis://127.0.0.1/"));
    let con = try!(client.get_connection());
    // throw away the result, just make sure it does not fail
    let _ : () = try!(con.set("my_key", 42));
    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    let ret = con.get("my_key");
    ret
}

fn main() {
    let user = "user";
    let addr = "127.0.0.1";
    let pwd: String = String::from("pass");
    let port: u16 = 3306;
    let mut builder = my::OptsBuilder::default();
    builder.user(Some(user))
            .pass(Some(pwd))
            .ip_or_hostname(Some(addr))
            .tcp_port(port)
            // .database("cas")
            .prefer_socket(false);
    
    // min 10 and max 100 as defualt
    // let pool = my::Pool::new("mysql://user:pass@127.0.0.1/cas").unwrap();
    // let pool = my::Pool::new(builder).unwrap();
    let pool = my::Pool::new_manual(1, 1, builder).unwrap();

    // Let's create payment table.
    // It is temporary so we do not need `tmp` database to exist.
    // Unwap just to make sure no error happened.
    pool.prep_exec(r"CREATE TEMPORARY TABLE cas.payment (
                         customer_id int not null,
                         amount int not null,
                         account_name text
                     )", ()).unwrap();

    let payments = vec![
        Payment { customer_id: 1, amount: 2, account_name: None },
        Payment { customer_id: 3, amount: 4, account_name: Some("foo".into()) },
        Payment { customer_id: 5, amount: 6, account_name: None },
        Payment { customer_id: 7, amount: 8, account_name: None },
        Payment { customer_id: 9, amount: 10, account_name: Some("bar".into()) },
    ];

    // Let's insert payments to the database
    // We will use into_iter() because we do not need to map Stmt to anything else.
    // Also we assume that no error happened in `prepare`.
    for mut stmt in pool.prepare(r"INSERT INTO cas.payment
                                       (customer_id, amount, account_name)
                                   VALUES
                                       (:customer_id, :amount, :account_name)").into_iter() {
        for p in payments.iter() {
            // `execute` takes ownership of `params` so we pass account name by reference.
            // Unwrap each result just to make sure no errors happened.
            stmt.execute(params!{
                "customer_id" => p.customer_id,
                "amount" => p.amount,
                "account_name" => &p.account_name,
            }).unwrap();
        }
    }

    // Let's select payments from database
    let selected_payments: Vec<Payment> =
    pool.prep_exec("SELECT customer_id, amount, account_name from cas.payment", ())
    .map(|result| { // In this closure we will map `QueryResult` to `Vec<Payment>`
        // `QueryResult` is iterator over `MyResult<row, err>` so first call to `map`
        // will map each `MyResult` to contained `row` (no proper error handling)
        // and second call to `map` will map each `row` to `Payment`
        result.map(|x| x.unwrap()).map(|row| {
            let (customer_id, amount, account_name) = my::from_row(row);
            Payment {
                customer_id: customer_id,
                amount: amount,
                account_name: account_name,
            }
        }).collect() // Collect payments so now `QueryResult` is mapped to `Vec<Payment>`
    }).unwrap(); // Unwrap `Vec<Payment>`

    // Now make sure that `payments` equals to `selected_payments`.
    // Mysql gives no guaranties on order of returned rows without `ORDER BY`
    // so assume we are lukky.
    assert_eq!(payments, selected_payments);
    println!("Yay!");

    println!("{}", fetch_an_integer().unwrap());
    // fetch_an_integer();
}