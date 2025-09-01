use my_zero2prod::run;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();
    run(listener).await;
}