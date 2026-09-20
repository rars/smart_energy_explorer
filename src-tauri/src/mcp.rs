use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    Router,
};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel::dsl::sql;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use crate::db::SqliteConnectionPool;
use crate::schema::{electricity_consumption, gas_consumption};

use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    },
    ErrorData, ServerHandler,
};

// Keep this stable because users copy the MCP configuration into their AI client.
// 55168 is in the OS-assigned dynamic/private port range.
const MCP_PORT: u16 = 55168;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInfo {
    pub url: String,
    pub token: String,
}

#[derive(Clone)]
pub struct McpServer {
    url: String,
    token: Arc<RwLock<String>>,
    cancellation_token: CancellationToken,
}

impl McpServer {
    pub fn info(&self) -> McpServerInfo {
        McpServerInfo {
            url: self.url.clone(),
            token: self.token.read().expect("MCP token lock poisoned").clone(),
        }
    }

    pub fn replace_token(&self, token: String) {
        *self.token.write().expect("MCP token lock poisoned") = token;
    }

    pub fn stop(&self) {
        self.cancellation_token.cancel();
    }
}

#[derive(Clone)]
struct EnergyMcpServer {
    db_pool: SqliteConnectionPool,
    #[allow(dead_code)]
    tool_router: rmcp::handler::server::router::tool::ToolRouter<Self>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct EnergySummaryRequest {
    #[schemars(description = "Inclusive date in YYYY-MM-DD format.")]
    start_date: String,
    #[schemars(description = "Exclusive date in YYYY-MM-DD format.")]
    end_date: String,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct EnergySummary {
    start_date: String,
    end_date: String,
    electricity: EnergyAmount,
    gas: EnergyAmount,
}

#[derive(Debug, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
struct EnergyAmount {
    watt_hours: i64,
    kilowatt_hours: f64,
}

impl EnergyMcpServer {
    fn new(db_pool: SqliteConnectionPool) -> Self {
        Self {
            db_pool,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router]
impl EnergyMcpServer {
    #[tool(
        name = "energy_summary",
        description = "Return total electricity and gas consumption for a London date range."
    )]
    async fn energy_summary(
        &self,
        Parameters(request): Parameters<EnergySummaryRequest>,
    ) -> Result<Json<EnergySummary>, ErrorData> {
        let start = parse_date(&request.start_date, "start_date")?;
        let end = parse_date(&request.end_date, "end_date")?;
        if end <= start {
            return Err(ErrorData::invalid_params(
                "end_date must be after start_date",
                None,
            ));
        }

        let start_time = NaiveDateTime::new(start, NaiveTime::MIN);
        let end_time = NaiveDateTime::new(end, NaiveTime::MIN);
        let pool = self.db_pool.clone();

        let result = tokio::task::spawn_blocking(move || {
            let mut connection = pool.get().map_err(|error| error.to_string())?;
            let electricity_wh = electricity_consumption::table
                .filter(electricity_consumption::timestamp.ge(start_time))
                .filter(electricity_consumption::timestamp.lt(end_time))
                .select(sql::<diesel::sql_types::BigInt>(
                    "COALESCE(SUM(energy_consumption_wh), 0)",
                ))
                .first::<i64>(&mut connection)
                .map_err(|error| error.to_string())?;
            let gas_wh = gas_consumption::table
                .filter(gas_consumption::timestamp.ge(start_time))
                .filter(gas_consumption::timestamp.lt(end_time))
                .select(sql::<diesel::sql_types::BigInt>(
                    "COALESCE(SUM(energy_consumption_wh), 0)",
                ))
                .first::<i64>(&mut connection)
                .map_err(|error| error.to_string())?;

            Ok::<_, String>(EnergySummary {
                start_date: start.to_string(),
                end_date: end.to_string(),
                electricity: EnergyAmount {
                    watt_hours: electricity_wh,
                    kilowatt_hours: electricity_wh as f64 / 1000.0,
                },
                gas: EnergyAmount {
                    watt_hours: gas_wh,
                    kilowatt_hours: gas_wh as f64 / 1000.0,
                },
            })
        })
        .await
        .map_err(|error| ErrorData::internal_error(error.to_string(), None))?
        .map_err(|error| ErrorData::internal_error(error, None))?;

        Ok(Json(result))
    }
}

#[tool_handler(name = "smart-energy-explorer")]
impl ServerHandler for EnergyMcpServer {}

pub async fn start(db_pool: SqliteConnectionPool, token: String) -> Result<McpServer, String> {
    let listener = TcpListener::bind(("127.0.0.1", MCP_PORT))
        .await
        .map_err(|error| format!("Failed to start MCP server on 127.0.0.1:{MCP_PORT}: {error}"))?;
    let address = listener
        .local_addr()
        .map_err(|error| format!("Failed to get MCP server address: {error}"))?;
    let token = Arc::new(RwLock::new(token));
    let cancellation_token = CancellationToken::new();
    let info = McpServer {
        url: format!("http://{address}/mcp"),
        token: token.clone(),
        cancellation_token: cancellation_token.clone(),
    };

    let service = StreamableHttpService::new(
        move || Ok(EnergyMcpServer::new(db_pool.clone())),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true)
            .with_cancellation_token(cancellation_token.clone()),
    );
    let router = Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn_with_state(token, authorize));

    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, router)
            .with_graceful_shutdown(cancellation_token.cancelled_owned())
            .await
        {
            log::error!("MCP server stopped: {error}");
        }
    });

    Ok(info)
}

async fn authorize(
    State(token): State<Arc<RwLock<String>>>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let expected = format!("Bearer {}", token.read().expect("MCP token lock poisoned"));
    let authorized = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == expected);

    if authorized {
        next.run(request).await
    } else {
        StatusCode::UNAUTHORIZED.into_response()
    }
}

fn parse_date(value: &str, name: &str) -> Result<NaiveDate, ErrorData> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ErrorData::invalid_params(format!("{name} must use YYYY-MM-DD format"), None))
}
