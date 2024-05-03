use actix_web::{
    http::header::ContentType,
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use futures::stream::StreamExt;
use langchain_rust::{
    chain::{builder::ConversationalChainBuilder, Chain, ConversationalChain},
    fmt_message, fmt_template,
    llm::{openai::OpenAI, OpenAIConfig},
    memory::SimpleMemory,
    message_formatter,
    prompt::HumanMessagePromptTemplate,
    prompt_args,
    schemas::Message,
    template_fstring,
};
use serde::Deserialize;
use std::{env, fmt::Error, sync::Arc};

pub struct State {
    pub chain: Arc<ConversationalChain>,
}

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

pub fn bootstrap(ollama_base_url: &str, model: &str) -> Result<Data<State>, Error> {
    let llm = OpenAI::default()
        .with_config(
            OpenAIConfig::default()
                .with_api_base(format!("{}/v1", ollama_base_url))
                .with_api_key("ollama"),
        )
        .with_model(model);

    let memory = SimpleMemory::new();

    let prompt = message_formatter![
        fmt_message!(Message::new_system_message(
            "You are a technical writer, specialist in rustlang programming language, you will write answer to the question for the beginners with some source code examples."
        )),
        fmt_template!(HumanMessagePromptTemplate::new(template_fstring!(
            "{input}", "input"
        ))),
    ];

    let chain = ConversationalChainBuilder::new()
        .llm(llm)
        .prompt(prompt)
        .memory(memory.into())
        .build()
        .expect("Error building ConversationalChain");

    Ok(Data::new(State {
        chain: Arc::new(chain),
    }))
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
            .app_data(bootstrap(&ollama_base_url, &model).unwrap())
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
