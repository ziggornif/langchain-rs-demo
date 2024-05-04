use std::{fmt::Error, sync::Arc};

use actix_web::web::Data;
use langchain_rust::chain::ConversationalChain;

use crate::conv_chain_builder::new_conv_chain;

pub struct State {
    pub chain: Arc<ConversationalChain>,
}

/**
 * Load actix application state
 *
 * This state allows to actix endpoint to access business functions (here conversational instance)
 */
pub fn load_state(ollama_base_url: &str, model: &str) -> Result<Data<State>, Error> {
    let chain = new_conv_chain(ollama_base_url, model);

    Ok(Data::new(State {
        chain: Arc::new(chain),
    }))
}
