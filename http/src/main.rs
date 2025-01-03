use std::{collections::HashMap, path::PathBuf, sync::Arc};

use actix_web::{
    delete, get, post,
    web::{self, Bytes},
    App, HttpResponse, HttpServer, Responder, Scope,
};
use bitcask_rs::{db::Engine, options::Options};

#[post("/put")]
async fn put_handler(
    eng: web::Data<Arc<Engine>>,
    data: web::Json<HashMap<String, String>>,
) -> impl Responder {
    for (key, value) in data.iter() {
        if let Err(_) = eng.put(Bytes::from(key.to_string()), Bytes::from(value.to_string())) {
            return HttpResponse::InternalServerError().body("failed to put value in engine");
        }
    }

    HttpResponse::Ok().body("OK")
}

#[get("/get/{key}")]
async fn get_handler(eng: web::Data<Arc<Engine>>, key: web::Path<String>) -> impl Responder {
    let value = match eng.get(Bytes::from(key.to_string())) {
        Ok(val) => val,
        Err(e) => {
            if e != bitcask_rs::errors::Errors::KeyNotFound {
                return HttpResponse::InternalServerError().body("failed to get value in engine");
            }
            return HttpResponse::NotFound().body("key not found");
        }
    };
    HttpResponse::Ok().body(value)
}

#[delete("/delete/{key}")]
async fn delete_handler(eng: web::Data<Arc<Engine>>, key: web::Path<String>) -> impl Responder {
    if let Err(e) = eng.delete(Bytes::from(key.to_string())) {
        if e != bitcask_rs::errors::Errors::KeyIsEmpty {
            return HttpResponse::InternalServerError().body("failed to delete value in engine");
        }
    }
    HttpResponse::Ok().body("OK")
}

#[get("/listkeys")]
async fn listkeys_handler(eng: web::Data<Arc<Engine>>) -> impl Responder {
    let keys = match eng.list_keys() {
        Ok(keys) => keys,
        Err(_) => return HttpResponse::InternalServerError().body("failed to list keys in engine"),
    };

    let keys = keys
        .iter()
        .map(|key| String::from_utf8(key.to_vec()).unwrap())
        .collect::<Vec<String>>();

    let result = serde_json::to_string(&keys).unwrap();
    HttpResponse::Ok()
        .content_type("application/json")
        .body(result)
}

#[get("/stat")]
async fn stat_handler(eng: web::Data<Arc<Engine>>) -> impl Responder {
    let stat = match eng.stat() {
        Ok(stat) => stat,
        Err(_) => return HttpResponse::InternalServerError().body("failed to get stats in engine"),
    };

    let mut result = HashMap::new();
    result.insert("key_num", stat.key_num);
    result.insert("data_file_num", stat.data_file_num);
    result.insert("reclaim_size", stat.reclaim_size);
    result.insert("disk_size", stat.disk_size as usize);
    HttpResponse::Ok().body(serde_json::to_string(&result).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 启动 Engine 实例
    let mut opts = Options::default();
    opts.dir_path = PathBuf::from("/tmp/bitcask-rs-http");
    let engine = Arc::new(Engine::open(opts).unwrap());

    // 启动 HTTP 服务
    HttpServer::new(move || {
        App::new().app_data(web::Data::new(engine.clone())).service(
            Scope::new("/bitcask")
                .service(put_handler)
                .service(get_handler)
                .service(delete_handler)
                .service(listkeys_handler)
                .service(stat_handler),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
