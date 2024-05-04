use actix_web::{http::header::ContentType, post, web, App, HttpResponse, HttpServer, Responder};
use futures::stream::StreamExt;
use langchain_rust::chain::Chain;
use langchain_rust::prompt_args;
use langchain_rust_demo::state::{load_state, State};
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug, Clone)]
struct PromptRequest {
    pub question: String,
}

#[post("/prompt")]
async fn send_prompt(data: web::Data<State>, request: web::Json<PromptRequest>) -> impl Responder {
    let input_variables = prompt_args! {
        "input" => request.question,
    };

    let stream_result = data.chain.stream(input_variables).await;
    match stream_result {
        Ok(stream) => {
            let stream = Box::pin(stream);
            let transformed_stream = stream.map(|result| match result {
                Ok(data) => Ok(actix_web::web::Bytes::from(data.content)),
                Err(e) => Err(actix_web::error::ErrorInternalServerError(format!(
                    "Stream error: {:?}",
                    e
                ))),
            });

            HttpResponse::Ok()
                .content_type(ContentType::plaintext())
                .streaming(transformed_stream)
        }
        Err(e) => {
            HttpResponse::InternalServerError().body(format!("Error creating stream: {:?}", e))
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let ollama_base_url =
        env::var("OLLAMA_BASE_URL").unwrap_or("http://localhost:11434".to_string());

    let model = env::var("LLM_MODEL").unwrap_or("llama3".to_string());

    let port = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse::<u16>()
        .expect("Invalid port");

    let server = HttpServer::new(move || {
        App::new()
            .service(send_prompt)
            .app_data(load_state(&ollama_base_url, &model).unwrap())
            .service(
                actix_files::Files::new("/", "src/public")
                    .show_files_listing()
                    .index_file("index.html")
                    .use_last_modified(true),
            )
    })
    .bind(("127.0.0.1", port))?
    .run();

    println!("Application running on http://localhost:{}", port);

    server.await
}
