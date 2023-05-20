use std::fmt::{Display, Formatter};

use actix_web::http::header::{HeaderValue, ACCESS_CONTROL_ALLOW_ORIGIN};
use actix_web::http::StatusCode;
use actix_web::middleware::Logger;
use actix_web::{
    dev::Service as _, App, HttpResponse, HttpServer, ResponseError,
};
use futures_util::future::FutureExt;
use log::LevelFilter;
use redis::{Commands, RedisError};

use crate::shared::Flag;

type Result<T = HttpResponse, E = Error> = std::result::Result<T, E>;

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
            .service(flags::service())
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
    NotFound(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Redis(err) => write!(f, "redis error: {err}"),
            Error::Internal(err) => {
                write!(f, "internal error: {err}")
            }
            Error::NotFound(item) => {
                write!(f, "not found: '{item}'")
            }
        }
    }
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::Redis(_) | Error::Internal(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Error::NotFound(_) => StatusCode::NOT_FOUND,
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

async fn save_flag(Flag { value, name }: Flag) -> Result<(), Error> {
    let mut conn = redis_connection()?;
    let key = format!("flags:{name}");
    conn.set(key, value)?;
    Ok(())
}

mod flags {
    use actix_web::dev::HttpServiceFactory;
    use actix_web::http::header;
    use actix_web::{get, options, post, web, HttpResponse};
    use redis::Commands;

    use crate::server::Error::NotFound;
    use crate::server::{redis_connection, save_flag, Error, Result};
    use crate::shared::Flag;

    fn decode_value_error(name: &str, msg: &str) -> Error {
        Error::Internal(format!("flag '{name}': {msg}"))
    }

    pub fn service() -> impl HttpServiceFactory + 'static {
        web::scope("/flags")
            .service(post)
            .service(get_all)
            .service(get)
            .service(options)
    }

    #[post("/")]
    async fn post(web::Json(flag): web::Json<Flag>) -> Result {
        save_flag(flag).await?;
        Ok(HttpResponse::Ok().finish())
    }

    #[get("/")]
    async fn get_all() -> Result {
        let mut conn = redis_connection()?;
        let keys: Vec<String> = conn.keys("flags:*")?;
        let flag_map = keys
            .iter()
            .map(|key| {
                let value = conn
                    .get::<_, String>(key)?
                    .as_str()
                    .try_into()
                    .map_err(|msg| decode_value_error(key, msg))?;
                Ok(Flag {
                    name: key[6..].to_string(),
                    value,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(HttpResponse::Ok().json(flag_map))
    }

    #[get("/{name}")]
    async fn get(path: web::Path<(String,)>) -> Result {
        let (name,) = path.into_inner();
        let key = format!("flags:{name}");
        let value = redis_connection()?
            .get::<_, Option<String>>(&key)?
            .ok_or(NotFound(key))?
            .as_str()
            .try_into()
            .map_err(|msg| decode_value_error(&name, msg))?;
        Ok(HttpResponse::Ok().json(Flag { name, value }))
    }

    #[options("/")]
    async fn options() -> Result {
        Ok(HttpResponse::Ok()
            .insert_header((
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                "content-type",
            ))
            .finish())
    }
}
