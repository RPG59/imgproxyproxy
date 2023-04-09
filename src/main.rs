use actix_proxy::{IntoHttpResponse, SendRequestError};
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use awc::Client;
use mongodb::{
    bson::doc,
    options::{ClientOptions, ServerApi, ServerApiVersion},
};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;

struct AppConfig {
    proxypath: String,
    presets_api_url: String,
    presets_map: std::sync::RwLock<HashMap<String, String>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Preset<'a> {
    #[serde(borrow)]
    testPreset: &'a str,
}

async fn connect_to_db() -> mongodb::error::Result<()> {
    let db_uri = std::env::var("DB_URI").expect("Error: DB_URI not found");
    let db_name = std::env::var("DB_NAME").expect("Error: DB_NAME not found");

    let mut client_options = ClientOptions::parse(db_uri).await?;
    // Set the server_api field of the client_options object to Stable API version 1
    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);
    // Create a new client and connect to the server
    let client = mongodb::Client::with_options(client_options)?;
    // Send a ping to confirm a successful connection
    // client
    //     .database(&db_name)
    //     .run_command(doc! {"find": {}}, None)
    //     .await?;

    let db = client.database(&db_name);
    let collection = db.collection::<Preset>("presets");
    let mut cursor = collection.find(None, None).await.unwrap();

    println!("Collection: {:?}", cursor.deserialize_current().unwrap());

    Ok(())
}

async fn fetch_presets(
    client: &Client,
    presets_api_url: &String,
) -> Option<HashMap<String, String>> {
    let mut res = match client.get(presets_api_url).send().await {
        Err(error) => {
            println!(
                "Fetch present on url: {} failed, {}",
                presets_api_url, error
            );
            return None;
        }
        Ok(data) => data,
    };

    match res.json::<HashMap<String, String>>().await {
        Err(error) => {
            println!("Presets parsing failed, {}", error);
            return None;
        }
        Ok(data) => Some(data),
    }
}

#[get("/proxy/{preset}/{path}")]
async fn proxy(
    client: web::Data<Client>,
    app_config: web::Data<AppConfig>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, SendRequestError> {
    let presets_map = app_config.presets_map.read().unwrap();

    match presets_map.get(&path.0) {
        Some(preset_data) => {
            let url = format!("{}/{}/{}.jpg", &app_config.proxypath, &preset_data, &path.1);
            Ok(client.get(&url).send().await?.into_http_response())
        }
        None => Ok(HttpResponse::NotFound().finish()),
    }
}

#[post("/update-config")]
async fn update_config(
    client: web::Data<Client>,
    app_config: web::Data<AppConfig>,
) -> Result<HttpResponse, SendRequestError> {
    match fetch_presets(&client, &app_config.presets_api_url).await {
        None => Ok(HttpResponse::NotModified().finish()),
        Some(updated_presets_map) => {
            let mut presets_map = app_config.presets_map.write().unwrap();
            presets_map.clear();
            presets_map.extend(updated_presets_map);

            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let proxypath = std::env::var("PROXYPATH").expect("Error: PROXYPATH not found");
    let presets_api_url =
        std::env::var("PRESETS_API_URL").expect("Error: PRESETS_API_URL not found");
    let client = Client::default();

    let foo = connect_to_db().await.unwrap();

    // let presets_map = fetch_presets(&client, &presets_api_url)
    //     .await
    //     .unwrap_or_default();
    let presets_map = HashMap::from([("testImg".to_string(), "testImg".to_string())]);

    println!("PRESETS MAP: {:?}", presets_map);

    let app_config = web::Data::new(AppConfig {
        presets_map: std::sync::RwLock::new(presets_map),
        proxypath,
        presets_api_url,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_config.clone())
            .app_data(web::Data::new(Client::default()))
            .service(update_config)
            .service(proxy)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
