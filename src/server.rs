use std::fmt::{Display, Formatter};

use actix_web::http::header::{HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN};
use actix_web::http::{header, StatusCode};
use actix_web::middleware::Logger;
use actix_web::{
    dev::Service as _, get, options, post, web, App, HttpResponse, HttpServer, ResponseError,
};
use futures_util::future::FutureExt;
use log::LevelFilter;
use redis::{Commands, RedisError};

use crate::shared::Flag;

pub async fn run() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .init();
    let make_app = || {
        App::new()
            .wrap(Logger::default())
            .wrap_fn(|req, srv| {
                srv.call(req).map(|res| {
                    res.map(|mut res| {
                        res.headers_mut().insert(
                            ACCESS_CONTROL_ALLOW_ORIGIN,
                            HeaderValue::from_str("*").unwrap(),
                        );
                        res
                    })
                })
            })
            .service(
                web::scope("/flags")
                    .service(get_flags)
                    .service(post_flags)
                    .service(flag_options),
            )
    };
    HttpServer::new(make_app)
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[derive(Debug)]
enum Error {
    Redis(RedisError),
    Internal(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Redis(err) => write!(f, "redis error: {err}"),
            Error::Internal(err) => write!(f, "internal error: {err}"),
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Redis(_) | Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

async fn save_flag(Flag { value, name }: Flag) -> Result<(), Error>
where
{
    let mut conn = redis_connection()?;
    let key = format!("flags:{name}");
    conn.set(key, value)?;
    Ok(())
}

#[post("/")]
async fn post_flags(web::Json(flag): web::Json<Flag>) -> Result {
    save_flag(flag).await?;
    Ok(HttpResponse::Ok().finish())
}

#[get("/")]
async fn get_flags() -> Result {
    let mut conn = redis_connection()?;
    let keys: Vec<String> = conn.keys("flags:*")?;
    let flag_map = keys
        .iter()
        .map(|key| -> Result<Flag> {
            let value = conn
                .get::<_, String>(key)?
                .as_str()
                .try_into()
                .map_err(|msg| Error::Internal(format!("key '{key}': {msg}")))?;
            Ok(Flag {
                name: key[6..].to_string(),
                value,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(HttpResponse::Ok().json(flag_map))
}

#[options("/")]
async fn flag_options() -> Result {
    Ok(HttpResponse::Ok()
        .insert_header((header::ACCESS_CONTROL_ALLOW_HEADERS, "content-type"))
        .finish())
}
