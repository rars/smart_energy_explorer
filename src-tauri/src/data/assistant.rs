use std::sync::{Arc, Mutex, MutexGuard};

use sqlx::sqlite::SqliteValueRef;
use sqlx::{query, sqlite::SqliteRow, Column, Row, SqlitePool};

use super::RepositoryError;

type RepositoryResult<T> = Result<T, RepositoryError>;

pub struct SqliteAssistantRepository {
    conn: SqlitePool,
}

impl SqliteAssistantRepository {
    pub fn new(conn: SqlitePool) -> Self {
        Self { conn }
    }

    fn get_connection(&self) -> RepositoryResult<SqlitePool> {
        Ok(self.conn.clone())
    }

    pub async fn execute_query(&self, query_expression: String) -> RepositoryResult<String> {
        let connection = self.get_connection()?;

        // Execute the query asynchronously
        // Note: `query` returns a Query type, `.execute()` returns a future
        let rows = query(&query_expression)
            .fetch_all(&connection)
            .await
            .map_err(|e| RepositoryError::SqliteQueryExecutionError(e))?;

        let answer = self.to_markdown_table(&rows).await?;

        Ok(answer)
    }

    async fn to_markdown_table(&self, rows: &Vec<SqliteRow>) -> RepositoryResult<String> {
        if rows.is_empty() {
            return Ok("No rows returned.".to_string());
        }

        // Get column names from the first row's columns
        let columns = rows[0].columns();

        // Build header row
        let headers: Vec<&str> = columns.iter().map(|col| col.name()).collect();

        // Markdown header line
        let mut markdown = String::new();
        markdown.push('|');
        for header in &headers {
            markdown.push_str(&format!(" {} |", header));
        }
        markdown.push('\n');

        // Markdown separator line
        markdown.push('|');
        for _ in &headers {
            markdown.push_str(" --- |");
        }
        markdown.push('\n');

        // Add data rows
        for row in rows {
            markdown.push('|');
            for col in columns {
                let val_str = if let Ok(opt_int) = row.try_get::<Option<i64>, _>(col.name()) {
                    if let Some(int_val) = opt_int {
                        int_val.to_string()
                    } else {
                        // Try float
                        match row.try_get::<Option<f64>, _>(col.name()) {
                            Ok(Some(float_val)) => float_val.to_string(),
                            _ => {
                                // Try string
                                match row.try_get::<Option<String>, _>(col.name()) {
                                    Ok(Some(s)) => s.replace('|', "\\|"),
                                    _ => "NULL".to_string(),
                                }
                            }
                        }
                    }
                } else {
                    // Try float
                    if let Ok(opt_float) = row.try_get::<Option<f64>, _>(col.name()) {
                        if let Some(float_val) = opt_float {
                            float_val.to_string()
                        } else {
                            // Try string
                            match row.try_get::<Option<String>, _>(col.name()) {
                                Ok(Some(s)) => s.replace('|', "\\|"),
                                _ => "NULL".to_string(),
                            }
                        }
                    } else {
                        // If failed to get Option<i64>, fallback to string
                        match row.try_get::<Option<String>, _>(col.name()) {
                            Ok(Some(s)) => s.replace('|', "\\|"),
                            _ => "NULL".to_string(),
                        }
                    }
                };

                markdown.push_str(&format!(" {} |", val_str));
            }
            markdown.push('\n');
        }

        Ok(markdown)
    }
}
