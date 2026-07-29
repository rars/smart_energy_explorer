use ollama_rs::{
    generation::{
        completion::request::GenerationRequest,
        parameters::{FormatType, JsonSchema, JsonStructure},
    },
    Ollama,
};
use schemars::{Schema, SchemaGenerator};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, State};

use crate::{data::assistant::SqliteAssistantRepository, AppState};

use super::ApiError;

struct AiAgent {
    ollama: Ollama,
    model: String,
}

impl AiAgent {
    pub fn new(model: String) -> Self {
        Self {
            ollama: Ollama::default(), // Connects to http://localhost:11434 by default
            model,
        }
    }

    pub async fn ask(&self, prompt: &str) -> Result<String, ApiError> {
        let request = GenerationRequest::new(self.model.clone(), prompt);

        let response = self.ollama.generate(request).await?;

        Ok(response.response)
    }

    pub async fn ask_for_json<T>(&self, prompt: &str) -> Result<T, ApiError>
    where
        T: JsonSchema + DeserializeOwned,
    {
        let schema = schemars::schema_for!(T);
        let format_type = FormatType::StructuredJson(Box::new(JsonStructure::from(schema)));

        let request = GenerationRequest::new(self.model.clone(), prompt).format(format_type);

        let response = self.ollama.generate(request).await?;

        let parsed = serde_json::from_str::<T>(&response.response)?;

        Ok(parsed)
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryItem {
    query: String,
    label: String,
    comments: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct QueryResponse {
    pub queries: Vec<QueryItem>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AnswerResponse {
    pub text: String,
    #[schemars(schema_with = "chart_js_schema")]
    pub chart_configuration: Value,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AssistantResponse {
    pub prompt_history: Vec<String>,
    pub answer_history: Option<QueryResponse>,
    pub answer: AnswerResponse,
}

pub struct AssistantResponseBuilder {
    pub prompt_history: Vec<String>,
    pub answer_history: Option<QueryResponse>,
}

impl AssistantResponseBuilder {
    pub fn new() -> Self {
        Self {
            prompt_history: Vec::new(),
            answer_history: None,
        }
    }

    pub fn add_prompt(&mut self, prompt: String) {
        self.prompt_history.push(prompt);
    }

    pub fn set_query_response(&mut self, query_response: QueryResponse) {
        self.answer_history = Some(query_response);
    }

    pub fn get_queries(&self) -> Option<&Vec<QueryItem>> {
        self.answer_history.as_ref().map(|x| &x.queries)
    }

    pub fn build(self, answer: AnswerResponse) -> AssistantResponse {
        AssistantResponse {
            prompt_history: self.prompt_history,
            answer_history: self.answer_history,
            answer,
        }
    }
}

#[tauri::command]
pub async fn ask_assistant(
    app_handle: AppHandle,
    app_state: State<'_, AppState>,
    message: &str,
    prompt: &str,
    model: &str,
) -> Result<AssistantResponse, ApiError> {
    let agent = AiAgent::new(model.to_string());

    let mut builder = AssistantResponseBuilder::new();

    let initial_prompt = format!("{prompt} {message}");

    builder.add_prompt(initial_prompt.clone());

    app_handle.emit("assistant-update", "Gathering data...")?;
    let first_response = agent.ask_for_json::<QueryResponse>(&initial_prompt).await?;

    builder.set_query_response(first_response);

    if let Some(queries) = builder.get_queries() {
        // if let Ok(queries) = serde_json::from_str::<Vec<QueryItem>>(&first_response) {

        let repository = SqliteAssistantRepository::new(app_state.sqlx_pool.clone());

        // Collect results from executing each query
        let mut combined_results = Vec::new();

        for query in queries {
            // Execute the query and get the result as a string
            let result = repository.execute_query(&query.query).await?;
            combined_results.push(format!(
                "Label: {}\nQuery: {},\nResult:\n{}\n",
                query.label, query.query, result
            ));
        }

        // Join all results into one answer string
        let answer = combined_results.join("\n---\n");

        let answer_prompt = format!(
            r#"
            You are an intelligent AI assistant whose primary function is to answer user questions by extracting and presenting relevant information *solely* from the provided query results. You must not attempt to run new queries or access external information.

            You have the following data available to you, which represents the direct output of specific SQLite queries. Each data block includes a descriptive label, the original query, and its result presented in JSON.
            {answer}

            Your task is to:
            Formulate a clear, concise, and direct answer to the user's question, strictly using the information present in the 'Result' markdown table(s) from the relevant data blocks. **Do not explicitly mention which data blocks were identified as relevant.** Present this information in a natural, conversational paragraph format, avoiding bullet points or numbered lists.
            If the user's intent cannot be fully or partially answered using the provided data, respond with a statement indicating that the information is not available.

            Your response should not contain SQL and be suitable for a non-technical user. It should sound like a helpful assistant.

            Your response should be a valid JSON object with the following shape:
            ```json
            {{
              "text": "Your written response",
              "chartConfiguration": <chart.js config object if illustrating the data in a chart form would be suitable or null otherwise>,
            }}
            ```

            The chart configuration should be complete and contain a type.

            The user intent follows.

            User Intent:
            {message}
            "#
        );

        builder.add_prompt(answer_prompt.clone());

        app_handle.emit("assistant-update", "Generating final answer...")?;

        let next_response = agent.ask_for_json::<AnswerResponse>(&answer_prompt).await?;

        let assistant_response = builder.build(next_response);

        app_handle.emit("assistant-update", "Waiting for interaction...")?;

        return Ok(assistant_response);
    } else {
        return Err(ApiError::Custom(
            "Could not execute queries to retrieve data supporting question".into(),
        ));
    }
    /* else {
        app_handle.emit("assistant-update", "Waiting for interaction...")?;

        assistant_response.answer = format!(
            "I'm having trouble answering your query given this response {}",
            first_response
        );

        return Ok(assistant_response);
    }*/
}

fn chart_js_schema(_gen: &mut SchemaGenerator) -> Schema {
    serde_json::from_value(json!({
        "type": "object",
        "description": "Chart.js configuration object",
        "properties": {
            "type": {
                "type": "string",
                "enum": ["bar", "line", "pie", "doughnut", "radar", "polarArea", "bubble", "scatter"],
                "description": "Chart.js chart type"
            },
            "data": {
                "type": "object",
                "properties": {
                    "labels": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "datasets": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "label": { "type": "string" },
                                "data": {
                                    "type": "array",
                                    "items": {
                                        "oneOf": [
                                            { "type": "number" },
                                            { "type": "object" }
                                        ]
                                    }
                                },
                                "backgroundColor": {
                                    "oneOf": [
                                        { "type": "string" },
                                        { "type": "array", "items": { "type": "string" } }
                                    ]
                                },
                                "borderColor": {
                                    "oneOf": [
                                        { "type": "string" },
                                        { "type": "array", "items": { "type": "string" } }
                                    ]
                                }
                            },
                            "required": ["data"]
                        }
                    }
                },
                "required": ["datasets"]
            },
            "options": {
                "type": "object",
                "description": "Chart.js options configuration"
            }
        },
        "required": ["type", "data"]
    }))
    .unwrap()
}

#[tauri::command]
pub async fn get_ollama_models() -> Result<Vec<String>, ApiError> {
    let ollama = Ollama::default();
    let models = ollama.list_local_models().await?;
    let model_names = models.into_iter().map(|m| m.name).collect();
    Ok(model_names)
}
