use ollama_rs::{generation::completion::request::GenerationRequest, Ollama};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::{data::assistant::SqliteAssistantRepository, AppState};

use super::ApiError;

#[derive(Debug, Deserialize)]
struct QueryItem {
    query: String,
    label: String,
    comments: Option<String>,
}

#[derive(Serialize, Debug)]
pub struct AssistantResponse {
    pub answer: String,
}

#[tauri::command]
pub async fn ask_assistant(
    app_state: State<'_, AppState>,
    message: &str,
    prompt: &str,
) -> Result<AssistantResponse, ApiError> {
    let ollama = Ollama::default(); // Connects to http://localhost:11434 by default

    let model = "mistral".to_string();
    // let prompt = message.to_string();

    let initial_prompt = format!(
        "{prompt} {message}. No need for an explanation, just the query please or the word false."
    );

    let request = GenerationRequest::new(model, initial_prompt);

    let first_response = ollama.generate(request).await?;

    if let Ok(queries) = serde_json::from_str::<Vec<QueryItem>>(&first_response.response) {
        let repository = SqliteAssistantRepository::new(app_state.sqlite_pool.clone());

        // Collect results from executing each query
        let mut combined_results = Vec::new();

        for query in queries {
            // Execute the query and get the result as a string
            let result = repository.execute_query(query.query.clone()).await?;
            combined_results.push(format!(
                "Label: {}\nQuery: {},\nResult:\n{}\n",
                query.label, query.query, result
            ));
        }

        // Join all results into one answer string
        let answer = combined_results.join("\n---\n");

        let response = AssistantResponse { answer };

        return Ok(response);
    } else {
        return Ok(AssistantResponse {
            answer: format!(
                "I'm having trouble answering your query given this response {}",
                first_response.response
            ),
        });
    }
}
