use actix_web::{
    http::header::ContentType,
    post,
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
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
use std::fmt::Error;
use std::{env, sync::Arc};

pub struct State {
    pub chain: Arc<ConversationalChain>,
}

#[derive(Deserialize, Debug, Clone)]
struct PromptRequest {
    pub question: String,
}

#[post("/prompt")]
async fn send_prompt(data: web::Data<State>, request: web::Json<PromptRequest>) -> impl Responder {
    // let data = format!("Hello world ! Asked question: {}", request.question);

    let input_variables = prompt_args! {
        "input" => request.question,
    };

    match data.chain.invoke(input_variables).await {
        Ok(result) => HttpResponse::Ok()
            .content_type(ContentType::plaintext())
            .body(result),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
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
            "You are a technical writer, specialist with the rustlang programming languages, you will write an answer to the question for the noobs, with some source code examples."
        )),
        // fmt_placeholder!("history"),
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

    // let input_variables = prompt_args! {
    //     "input" => "What is a Rust struct ?",
    // };

    // let mut stream = chain.stream(input_variables).await.unwrap();
    // while let Some(result) = stream.next().await {
    //     match result {
    //         Ok(data) => {
    //             //If you junt want to print to stdout, you can use data.to_stdout().unwrap();
    //             print!("{}", data.content);
    //             stdout().flush().unwrap();
    //         }
    //         Err(e) => {
    //             println!("Error: {:?}", e);
    //         }
    //     }
    // }

    // let input_variables = prompt_args! {
    //     "input" => "Add address to the person struct",
    // };
    // match chain.invoke(input_variables).await {
    //     Ok(result) => {
    //         println!("\n");
    //         println!("Result: {:?}", result);
    //     }
    //     Err(e) => panic!("Error invoking LLMChain: {:?}", e),
    // }

    server.await
}
