mod read;

use std::{str::FromStr, sync::Arc, time::Instant};

use crate::{
    config::SimulationConfiguration,
    states::{Runtime, Runtimes},
    PAGES,
};
use libasm::{compile, Input};
use libseis::pages::PAGE_SIZE;
use rocket::{
    get, http, post,
    response::content::{RawHtml, RawText},
    routes,
    serde::json::Json,
    Route, State,
};
use serde_json::Value;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Gets a session corresponding to the `uuid`. If possible, updates the time since last use.
async fn get_uuid(
    runtimes: &Runtimes,
    uuid: Uuid,
) -> Result<Arc<RwLock<Runtime>>, (http::Status, String)> {
    let lock = runtimes.read().await;

    lock.get(&uuid)
        .map(|(r, i)| {
            if let Ok(mut lock) = i.try_lock() {
                *lock = Instant::now();
            }

            r.clone()
        })
        .ok_or((
            http::Status::NotFound,
            format!("UUID {uuid} is not an active simulation"),
        ))
}

#[inline]
fn into_uuid(uuid: &str) -> Result<Uuid, (http::Status, String)> {
    Uuid::from_str(uuid).map_err(|e| (http::Status::BadRequest, e.to_string()))
}

#[post("/", data = "<config>")]
pub async fn init(
    runtimes: &State<Runtimes>,
    config: Json<Value>,
) -> Result<RawText<String>, (http::Status, RawText<String>)> {
    let mut uuid = Uuid::new_v4();

    let mut lock = runtimes.write().await;

    while lock.contains_key(&uuid) {
        uuid = Uuid::new_v4();
    }

    let files = config
        .get("files")
        .ok_or_else(|| {
            (
                http::Status::BadRequest,
                RawText("Expected key `files`".to_owned()),
            )
        })?
        .as_object()
        .ok_or_else(|| {
            (
                http::Status::BadRequest,
                RawText("Expected key `files` to be a dictionary".into()),
            )
        })?;

    if files.len() == 0 {
        return Err((
            http::Status::BadRequest,
            RawText("`files` requires at least one entry".into()),
        ));
    }

    let input: Vec<Input> = files
        .iter()
        .map(|(k, v)| {
            Ok(libasm::Input {
                path: &k,
                data: v.as_str().ok_or_else(|| {
                    (
                        http::Status::BadRequest,
                        RawText(format!("Expected value for {k} to be a string")),
                    )
                })?,
            })
        })
        .collect::<Result<Vec<Input>, (http::Status, RawText<String>)>>()?;

    let config = SimulationConfiguration::from_json(&config)
        .map_err(|e| (http::Status::BadRequest, RawText(e.to_string())))?;

    let bin =
        compile(input).map_err(|e| (http::Status::InternalServerError, RawText(e.to_string())))?;

    lock.insert(uuid, Runtime::new(uuid, config, bin));

    println!("Created a new configuration");

    Ok(RawText(uuid.to_string()))
}

#[get("/<uuid>")]
pub async fn dashboard(
    runtimes: &State<Runtimes>,
    uuid: &str,
) -> Result<RawHtml<String>, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    get_uuid(runtimes, uuid).await?;

    Ok(RawHtml(format!(
        include_str!("application.html"),
        uuid = uuid,
        pages = PAGES,
        page_size = PAGE_SIZE,
    )))
}

#[post("/<uuid>/clock", data = "<count>")]
pub async fn clock(
    runtimes: &State<Runtimes>,
    uuid: &str,
    count: Json<usize>,
) -> Result<String, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let mut runtime = runtime_arc.write().await;

    Ok(format!("{:#?}", runtime.state.clock(*count)))
}

pub fn exports() -> Vec<Route> {
    routes![
        init,
        dashboard,
        clock,
        read::read_page,
        read::read_address,
        read::read_watchlist,
        read::set_watchlist,
        read::read_registers,
        read::read_configuration,
        read::read_pipeline_state,
    ]
}
