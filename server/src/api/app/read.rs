use super::{get_uuid, into_uuid};
use crate::states::{AddressType, Runtimes};
use libseis::{instruction_set::Decode, types::Word};
use rocket::{get, http, post, serde::json::Json, State};
use serde_json::Value;
use std::{collections::HashMap, hash::BuildHasher};

#[get("/<uuid>/page/<page_id>?<hash>")]
pub async fn read_page(
    runtimes: &State<Runtimes>,
    uuid: &str,
    page_id: usize,
    hash: Option<u64>,
) -> Result<Json<Option<Value>>, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let runtime = runtime_arc.read().await;

    let page = page_id / 4;
    let subpage = page_id % 4;

    let data = runtime
        .state
        .memory_module()
        .memory()
        .get_page(page)
        .and_then(|d| {
            use serde_json::Map;

            let mut values = Map::default();
            let data = Vec::from(&d[subpage * 16384..(subpage + 1) * 16384]);
            let new_hash = runtime.hash_state.hash_one(&data);

            if let Some(hash) = hash {
                if hash == new_hash {
                    return None;
                }
            }

            values.insert("hash".into(), new_hash.to_string().into());
            values.insert("data".into(), data.into());

            Some(values.into())
        });

    Ok(Json(data))
}

#[get("/<uuid>/page/<page_id>?<hash>&disasm")]
pub async fn read_page_disasm(
    runtimes: &State<Runtimes>,
    uuid: &str,
    page_id: usize,
    hash: Option<u64>,
) -> Result<Json<Option<Value>>, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let runtime = runtime_arc.read().await;

    let page = page_id / 4;
    let subpage = page_id % 4;

    let data = runtime
        .state
        .memory_module()
        .memory()
        .get_page(page)
        .and_then(|d| {
            use libseis::instruction_set::Instruction;
            use serde_json::Map;

            let mut values = Map::default();
            let data = Vec::from(&d[subpage * 16384..(subpage + 1) * 16384]);
            let new_hash = runtime.hash_state.hash_one(&data);

            if let Some(hash) = hash {
                if hash == new_hash {
                    return None;
                }
            }

            values.insert("hash".into(), new_hash.to_string().into());
            values.insert(
                "data".into(),
                data.chunks(4)
                    .map(|c| {
                        Instruction::decode(Word::from_be_bytes([c[0], c[1], c[2], c[3]]))
                            .map(|i| i.to_string())
                            .unwrap_or("<INVALID>".into())
                    })
                    .collect::<Vec<String>>()
                    .into(),
            );

            Some(values.into())
        });

    Ok(Json(data))
}

#[get("/<uuid>/configuration")]
pub async fn read_configuration(
    runtimes: &State<Runtimes>,
    uuid: &str,
) -> Result<Json<Value>, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let runtime = runtime_arc.read().await;

    Ok(Json(runtime.config.to_json()))
}

#[get("/<uuid>/watchlist")]
pub async fn read_watchlist(
    runtimes: &State<Runtimes>,
    uuid: &str,
) -> Result<Json<HashMap<Word, String>>, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let runtime = runtime_arc.read().await;

    Ok(Json(runtime.read_watchlist()))
}

#[post("/<uuid>/watchlist", data = "<addresses>")]
pub async fn set_watchlist(
    runtimes: &State<Runtimes>,
    uuid: &str,
    addresses: Json<HashMap<Word, Option<AddressType>>>,
) -> Result<Json<HashMap<Word, (AddressType, String)>>, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let mut runtime = runtime_arc.write().await;

    for (&address, &mode) in addresses.iter() {
        if let Some(r#type) = mode {
            runtime.watchlist.insert(address, r#type);
        } else {
            runtime.watchlist.remove(&address);
        }
    }

    Ok(Json(
        runtime
            .watchlist
            .iter()
            .map(|(&a, &t)| {
                use AddressType::*;

                let v = match t {
                    Byte => runtime
                        .state
                        .memory_module()
                        .memory()
                        .read_byte(a)
                        .to_string(),
                    Short => runtime
                        .state
                        .memory_module()
                        .memory()
                        .read_short(a)
                        .to_string(),
                    Word => runtime
                        .state
                        .memory_module()
                        .memory()
                        .read_word(a)
                        .to_string(),
                    Float => f32::from_bits(runtime.state.memory_module().memory().read_word(a))
                        .to_string(),
                };

                (a, (t, v))
            })
            .collect(),
    ))
}

#[get("/<uuid>/registers")]
pub async fn read_registers(
    runtimes: &State<Runtimes>,
    uuid: &str,
) -> Result<Json<Value>, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let runtime = runtime_arc.read().await;

    Ok(Json(runtime.read_regs()))
}

#[get("/<uuid>/memory/<address>/<type>")]
pub async fn read_address(
    runtimes: &State<Runtimes>,
    uuid: &str,
    address: Word,
    r#type: &str,
) -> Result<String, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let runtime = runtime_arc.read().await;

    match r#type.to_lowercase().as_str() {
        "byte" => Ok(runtime
            .state
            .memory_module()
            .memory()
            .read_byte(address)
            .to_string()),
        "short" => Ok(runtime
            .state
            .memory_module()
            .memory()
            .read_short(address)
            .to_string()),
        "word" => Ok(runtime
            .state
            .memory_module()
            .memory()
            .read_word(address)
            .to_string()),
        "float" => Ok(
            f32::from_bits(runtime.state.memory_module().memory().read_word(address)).to_string(),
        ),
        t => Err((
            http::Status::BadRequest,
            format!("\"{t}\" is not a valid type"),
        )),
    }
}

#[get("/<uuid>/pipeline")]
pub async fn read_pipeline_state(
    runtimes: &State<Runtimes>,
    uuid: &str,
) -> Result<Json<Value>, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let runtime = runtime_arc.read().await;

    Ok(Json(serde_json::to_value(runtime.state.stages()).unwrap()))
}

#[get("/<uuid>/cache/names")]
pub async fn read_cache_names(
    runtimes: &State<Runtimes>,
    uuid: &str,
) -> Result<Json<Value>, (http::Status, String)> {
    use serde_json::Map;
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let runtime = runtime_arc.read().await;

    Ok(Json(
        runtime
            .config
            .cache
            .iter()
            .map(|(k, v)| (k.clone(), v.to_json().into()))
            .collect::<Map<String, Value>>()
            .into(),
    ))
}

#[get("/<uuid>/cache/read/<name>")]
pub async fn read_cache(
    runtimes: &State<Runtimes>,
    uuid: &str,
    name: &str,
) -> Result<Json<Value>, (http::Status, String)> {
    let uuid = into_uuid(uuid)?;

    let runtime_arc = get_uuid(runtimes, uuid).await?;
    let runtime = runtime_arc.read().await;

    Ok(Json(
        runtime
            .state
            .memory_module()
            .cache_state()
            .iter()
            .find(|c| c.name == name.to_lowercase())
            .map(|c| serde_json::to_value(&c.lines).unwrap())
            .ok_or_else(|| (http::Status::NotFound, format!("Cache {name} not found")))?,
    ))
}
