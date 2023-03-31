use std::fmt::{Display, Formatter};

use actix_web::http::StatusCode;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, ResponseError};
use redis::{Commands, RedisError};

use crate::Flag;

pub async fn run() -> std::io::Result<()> {
    let make_app = || App::new().service(web::scope("/flags").service(get_flags).service(set_flag));
    HttpServer::new(make_app)
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[derive(Debug)]
enum Error {
    Redis(RedisError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Redis(err) => write!(f, "redis error: {err}"),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<RedisError> for Error {
    fn from(error: RedisError) -> Self {
        Self::Redis(error)
    }
}

fn redis_connection() -> Result<redis::Connection, RedisError> {
    redis::Client::open("redis://127.0.0.1/")?.get_connection()
}

type Result<T = HttpResponse, E = Error> = std::result::Result<T, E>;

#[post("/")]
async fn set_flag(info: web::Json<Flag>) -> Result {
    let mut conn = redis_connection()?;
    let key = format!("flags:{}", info.name);
    conn.set(key, info.enabled)?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/")]
async fn get_flags() -> Result {
    let mut conn = redis_connection()?;
    let keys: Vec<String> = conn.keys("flags:*")?;
    let flag_map = keys
        .iter()
        .map(|key| {
            conn.get(key).map(|value: bool| Flag {
                name: key[6..].to_string(),
                enabled: value,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HttpResponse::Ok().json(flag_map))
}
