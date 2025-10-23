use std::sync::{Arc, RwLock};

use actix_web::{
    App, HttpRequest, HttpResponse, HttpServer, Responder,
    dev::Service,
    get,
    http::header::{HeaderName, HeaderValue},
    post, web,
};
use arraydeque::ArrayDeque;
use constant_time_eq::constant_time_eq;
use dwd_fetch::Cache;
use jiff::tz::TimeZone;

use crate::{config::CONFIG, fonts::Fonts, paint::PaintCtx, render};

const BSEC_STATE_LEN: usize = 180;

#[derive(bincode::Decode, bincode::Encode)]
struct BodyData {
    pub secret: [u8; 512],
    pub state: [u8; BSEC_STATE_LEN],
    pub temperature: f32,
    pub iaq: f32,
    pub co2: f32,
}

struct AppState {
    pub cache: Arc<RwLock<Cache>>,
    pub paint_ctx: PaintCtx,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize)]
struct DataItem {
    pub timestamp: i64,
    pub temperature: f32,
    pub iaq: f32,
    pub co2: f32,
}
#[derive(Debug, Default)]
struct History(pub ArrayDeque<DataItem, 128, arraydeque::behavior::Wrapping>);

impl serde::Serialize for History {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(self.0.iter())
    }
}

#[derive(Debug, thiserror::Error, actix_web_error::Json)]
#[status(BAD_REQUEST)]
enum MyError {
    #[error("Silly input")]
    #[status(401)]
    SillyInput,
    #[error("Idk kev")]
    Other,
}

#[post("/refresh")]
async fn refresh(
    data: web::Bytes,
    state: web::Data<AppState>,
    hist: web::Data<RwLock<History>>,
) -> Result<Vec<u8>, MyError> {
    let data = bincode::decode_from_slice::<BodyData, _>(&data, bincode::config::standard())
        .map_err(|_| MyError::SillyInput)?
        .0;
    if !constant_time_eq::constant_time_eq_n(&data.secret, CONFIG.secret()) {
        return Err(MyError::SillyInput);
    }
    std::fs::write("state.bin", data.state).map_err(|_| MyError::Other)?;

    let item = DataItem {
        timestamp: jiff::Timestamp::now().as_millisecond(),
        temperature: data.temperature,
        iaq: data.iaq,
        co2: data.co2,
    };
    if let Ok(mut w) = hist.write() {
        w.0.push_back(item);
    }

    let res = actix_web::rt::task::spawn_blocking(move || -> anyhow::Result<Box<[u8]>> {
        Cache::refetch(&state.cache, CONFIG.dwd())?;

        render::full(
            &state.cache.read().unwrap(),
            &state.paint_ctx,
            data.temperature,
            data.iaq,
            data.co2,
        )
    })
    .await
    .map_err(|_| MyError::Other)?
    .map_err(|_| MyError::Other)?;

    Ok(res.into_vec())
}

#[get("/state")]
async fn get_state() -> impl Responder {
    std::fs::read("state.bin").map_err(|_| MyError::Other)
}

#[get("/history")]
async fn history(req: HttpRequest, hist: web::Data<RwLock<History>>) -> impl Responder {
    if req
        .headers()
        .get("Authorization")
        .and_then(|a| a.to_str().ok())
        .and_then(|a| a.strip_prefix("Bearer "))
        .is_none_or(|a| !constant_time_eq(a.as_bytes(), CONFIG.access_secret().as_bytes()))
    {
        return Err(MyError::SillyInput);
    };

    Ok(HttpResponse::Ok().json(&*hist.read().map_err(|_| MyError::Other)?))
}

fn is_night(tz: TimeZone) -> bool {
    use jiff::civil::Weekday;
    let now = jiff::Timestamp::now();
    let zoned = now.to_zoned(tz);
    match zoned.weekday() {
        Weekday::Saturday | Weekday::Sunday => zoned.hour() < 8 || zoned.hour() >= 23,
        _ => (zoned.hour() < 7) || zoned.hour() >= 23,
    }
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    let my_tz = jiff::tz::db().get("Europe/Berlin").unwrap();

    let history_data = Arc::new(RwLock::new(History::default()));
    HttpServer::new(move || {
        let tz = my_tz.clone();
        App::new()
            .app_data(web::Data::new(AppState {
                cache: Arc::new(RwLock::new(Cache::default())),
                paint_ctx: PaintCtx {
                    fonts: Fonts::new("./fonts/InterVariable.ttf"),
                },
            }))
            .app_data(web::Data::from(history_data.clone()))
            .wrap_fn(move |req, srv| {
                let fut = srv.call(req);
                let is_night = is_night(tz.clone());
                async move {
                    let mut res = fut.await?;
                    if is_night {
                        res.headers_mut().insert(
                            HeaderName::from_static("X-IsNight"),
                            HeaderValue::from_static("1"),
                        );
                    }
                    Ok(res)
                }
            })
            .service(refresh)
            .service(get_state)
            .service(history)
    })
    .bind((CONFIG.host(), CONFIG.port()))?
    .run()
    .await
}
