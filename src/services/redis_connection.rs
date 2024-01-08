use once_cell::sync::Lazy;
use redis::{Client as RedisClient, Connection};
use std::{env, sync::Mutex};

static REDIS_CONNECTION: Lazy<Mutex<Connection>> = Lazy::new(|| {
    let redis_connection = RedisClient::open(env::var("REDIS_URL").unwrap().as_str())
        .unwrap()
        .get_connection()
        .unwrap();

    Mutex::new(redis_connection)
});

pub fn get_redis_connection() -> std::sync::MutexGuard<'static, Connection> {
    REDIS_CONNECTION.lock().unwrap()
}
