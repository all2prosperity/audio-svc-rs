use oz_server::models::establish_connection;

async fn _main() {
    println!("oz_server");
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(_main());
    establish_connection();
}
