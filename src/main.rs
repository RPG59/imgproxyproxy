use actix_proxy::{IntoHttpResponse, SendRequestError};
use actix_web::{get, web, App, HttpResponse, HttpServer};
use awc::Client;
use base64::{self, Engine};

struct AppState {
    imgproxy_url: String,
}

#[get("/proxy/{preset}/{path}")]
async fn proxy(
    client: web::Data<Client>,
    app_state: web::Data<AppState>,
    path: web::Path<(String, String)>,
) -> Result<HttpResponse, SendRequestError> {
    let request = client
        .get(format!(
            "{}/{}/{}",
            app_state.imgproxy_url, &path.0, &path.1
        ))
        .send()
        .await
        .unwrap();

    println!("Status Code: {}", request.status());

    if !request.status().is_success() {
        let original_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&path.1)
            .unwrap();
        let original_url = std::str::from_utf8(&original_bytes).unwrap();

        return Ok(client.get(original_url).send().await?.into_http_response());
    }

    Ok(request.into_http_response())

    // Ok(HttpResponse::NotFound().finish())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                imgproxy_url: std::env::var("IMGPROXY_URL").expect("Error: IMGPROXY_URL not found"),
            }))
            .app_data(web::Data::new(Client::default()))
            .service(proxy)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
