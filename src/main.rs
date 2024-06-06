use actix_files::NamedFile;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Deserialize)]
struct TranslateRequest {
    text: String,
    source_lang: String,
    target_lang: String,
    tone: String,
}

#[derive(Serialize)]
struct TranslateResponse {
    translation: String,
}

async fn translate(req: web::Json<TranslateRequest>, client: web::Data<Client>) -> impl Responder {
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");

    let translation_prompt = format!(
        "Translate the following text from {} to {} with a {} tone: \"{}\"",
        req.source_lang, req.target_lang, req.tone, req.text
    );

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&serde_json::json!({
            "model": "gpt-3.5-turbo",
            "messages": [{ "role": "user", "content": translation_prompt }],
            "max_tokens": 100,
        }))
        .send()
        .await;

    match response {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => {
                if let Some(translation) = json["choices"][0]["message"]["content"].as_str() {
                    HttpResponse::Ok().json(TranslateResponse {
                        translation: translation.trim().to_string(),
                    })
                } else {
                    HttpResponse::InternalServerError().json("Failed to get a valid response from OpenAI")
                }
            }
            Err(_) => HttpResponse::InternalServerError().json("Failed to parse response from OpenAI"),
        },
        Err(_) => HttpResponse::InternalServerError().json("Failed to connect to OpenAI"),
    }
}

async fn index() -> Result<NamedFile> {
    Ok(NamedFile::open("public/index.html")?)
}

async fn static_files(req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    let path_str = format!("public/{}", path.display());
    Ok(NamedFile::open(path_str)?)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let client = Client::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .service(web::resource("/translate").route(web::post().to(translate)))
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/{filename:.*}").route(web::get().to(static_files)))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
