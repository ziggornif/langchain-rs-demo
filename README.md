# Langchain-rs demo

Let's build a Rust teacher with langchain-rs !

![feris logo](./src/public/assets/img/ferris_teacher.png)

## Requirements

- [Ollama](https://ollama.com/)
- [LLM model of your choice (default: llama3)](https://ollama.com/library)

## Quickstart

First we need to define environment variables values :
- OLLAMA_BASE_URL : Your Ollama API endpoint URL (default : http://localhost:11434)
- LLM_MODEL : the model of your choice (default : llama3)

Then, run the application with :

```sh
cargo run
```

The application should run at http://localhost:8080.

