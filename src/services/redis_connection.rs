use once_cell::sync::Lazy;
use redis::{Client, Connection};
use std::sync::Mutex;

static REDIS_CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let client = Client::open("redis://127.0.0.1:6379/").unwrap();
    let connection = client.get_connection().unwrap();
    Mutex::new(connection)
});

pub fn get_redis_connection() -> std::sync::MutexGuard<'static, Connection> {
    REDIS_CONNECTION.lock().unwrap()
}
