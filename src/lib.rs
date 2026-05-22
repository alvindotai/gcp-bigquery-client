//! [<img alt="github" src="https://img.shields.io/badge/github-lquerel/gcp_bigquery_client-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/lquerel/gcp-bigquery-client)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/gcp_bigquery_client.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/gcp-bigquery-client)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/lquerel/gcp-bigquery-client/Rust/main?style=for-the-badge" height="20">](https://github.com/lquerel/gcp-bigquery-client/actions?query=branch%3Amain)
//!
//! An ergonomic async client library for GCP BigQuery.
//! * Support for dataset, table, streaming API and query (see [status section](#status) for an exhaustive list of supported API endpoints)
//! * Support Service Account Key authentication (other OAuth flows will be added later)
//! * Create tables and rows via builder patterns
//! * Persist complex Rust structs in structured BigQuery tables
//! * Async API
//!
//! <br>
//!
//! Other OAuth flows will be added later.
//!
//! For a detailed tutorial on the different ways to use GCP BigQuery Client please check out the [GitHub repository](https://github.com/lquerel/gcp-bigquery-client).
#[macro_use]
extern crate serde;
extern crate serde_json;

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use client_builder::ClientBuilder;
use reqwest::{header, Response};
use serde::Deserialize;
use storage::StorageApi;
use yup_oauth2::ServiceAccountKey;

use crate::auth::Authenticator;
use crate::dataset::DatasetApi;
use crate::error::BQError;
use crate::job::JobApi;
use crate::model_api::ModelApi;
use crate::project::ProjectApi;
use crate::routine::RoutineApi;
use crate::table::TableApi;
use crate::tabledata::TableDataApi;

/// Since yup_oauth2 structs are used as parameters in public functions there is already semver
/// coupling, as it is an error if consumer uses different version of yup_oauth than gcp-bigquery-client
/// Export yup_oauth2 so consumers don't need to carefully keep their dependency versions in sync.
/// (see https://github.com/lquerel/gcp-bigquery-client/pull/86)
pub use yup_oauth2;

pub mod auth;
pub mod client_builder;
pub mod dataset;
pub mod error;
pub mod job;
pub mod model;
pub mod model_api;
pub mod project;
pub mod routine;
pub mod storage;
pub mod table;
pub mod tabledata;

const BIG_QUERY_V2_URL: &str = "https://bigquery.googleapis.com/bigquery/v2";
const BIG_QUERY_AUTH_URL: &str = "https://www.googleapis.com/auth/bigquery";

/// An asynchronous BigQuery client.
#[derive(Clone)]
pub struct Client {
    dataset_api: DatasetApi,
    table_api: TableApi,
    job_api: JobApi,
    tabledata_api: TableDataApi,
    routine_api: RoutineApi,
    model_api: ModelApi,
    project_api: ProjectApi,
    storage_api: StorageApi,
}

impl Client {
    pub async fn from_authenticator(auth: Arc<dyn Authenticator>) -> Result<Self, BQError> {
        let user_agent = "alvin-connect-service/1.0 (GPN:Alvin.ai;)";
        let mut headers = header::HeaderMap::new();
        headers.insert(header::USER_AGENT, header::HeaderValue::from_static(user_agent));

        let mut builder = reqwest::Client::builder().default_headers(headers);
        if std::env::var("GCP_MAX_TLS_VERSION").as_deref() == Ok("1.2") {
            builder = builder.max_tls_version(reqwest::tls::Version::TLS_1_2);
        }
        let client = builder.build()?;
        let write_client = StorageApi::new_write_client_with_user_agent(user_agent).await?;
        let read_client = StorageApi::new_read_client_with_user_agent(user_agent).await?;

        Ok(Self {
            dataset_api: DatasetApi::new(client.clone(), Arc::clone(&auth)),
            table_api: TableApi::new(client.clone(), Arc::clone(&auth)),
            job_api: JobApi::new(client.clone(), Arc::clone(&auth)),
            tabledata_api: TableDataApi::new(client.clone(), Arc::clone(&auth)),
            routine_api: RoutineApi::new(client.clone(), Arc::clone(&auth)),
            model_api: ModelApi::new(client.clone(), Arc::clone(&auth)),
            project_api: ProjectApi::new(client, Arc::clone(&auth)),
            storage_api: StorageApi::new(write_client, read_client, auth),
        })
    }

    /// Constructs a new BigQuery client.
    /// # Argument
    /// * `sa_key_file` - A GCP Service Account Key file.
    pub async fn from_service_account_key_file(sa_key_file: &str) -> Result<Self, BQError> {
        ClientBuilder::new()
            .build_from_service_account_key_file(sa_key_file)
            .await
    }

    /// Constructs a new BigQuery client from a [`ServiceAccountKey`].
    /// # Argument
    /// * `sa_key` - A GCP Service Account Key `yup-oauth2` object.
    /// * `readonly` - A boolean setting whether the acquired token scope should be readonly.
    ///
    /// [`ServiceAccountKey`]: https://docs.rs/yup-oauth2/*/yup_oauth2/struct.ServiceAccountKey.html
    pub async fn from_service_account_key(sa_key: ServiceAccountKey, readonly: bool) -> Result<Self, BQError> {
        ClientBuilder::new()
            .build_from_service_account_key(sa_key, readonly)
            .await
    }

    pub async fn with_workload_identity(readonly: bool) -> Result<Self, BQError> {
        ClientBuilder::new().build_with_workload_identity(readonly).await
    }

    pub(crate) fn v2_base_url(&mut self, base_url: String) -> &mut Self {
        self.dataset_api.with_base_url(base_url.clone());
        self.table_api.with_base_url(base_url.clone());
        self.job_api.with_base_url(base_url.clone());
        self.tabledata_api.with_base_url(base_url.clone());
        self.routine_api.with_base_url(base_url.clone());
        self.model_api.with_base_url(base_url.clone());
        self.project_api.with_base_url(base_url.clone());
        self.storage_api.with_base_url(base_url);
        self
    }

    pub async fn from_installed_flow_authenticator<S: AsRef<[u8]>, P: Into<PathBuf>>(
        secret: S,
        persistant_file_path: P,
    ) -> Result<Self, BQError> {
        ClientBuilder::new()
            .build_from_installed_flow_authenticator(secret, persistant_file_path)
            .await
    }

    pub async fn from_installed_flow_authenticator_from_secret_file<P: Into<PathBuf>>(
        secret_file: &str,
        persistant_file_path: P,
    ) -> Result<Self, BQError> {
        Self::from_installed_flow_authenticator(
            tokio::fs::read(secret_file)
                .await
                .expect("expecting a valid secret file."),
            persistant_file_path,
        )
        .await
    }

    pub async fn from_application_default_credentials() -> Result<Self, BQError> {
        ClientBuilder::new().build_from_application_default_credentials().await
    }

    pub async fn from_authorized_user_secret(secret: &str) -> Result<Self, BQError> {
        ClientBuilder::new()
            .build_from_authorized_user_authenticator(secret)
            .await
    }

    /// Returns a dataset API handler.
    pub fn dataset(&self) -> &DatasetApi {
        &self.dataset_api
    }

    /// Returns a table API handler.
    pub fn table(&self) -> &TableApi {
        &self.table_api
    }

    /// Returns a job API handler.
    pub fn job(&self) -> &JobApi {
        &self.job_api
    }

    /// Returns a table data API handler.
    pub fn tabledata(&self) -> &TableDataApi {
        &self.tabledata_api
    }

    /// Returns a routine API handler.
    pub fn routine(&self) -> &RoutineApi {
        &self.routine_api
    }

    /// Returns a model API handler.
    pub fn model(&self) -> &ModelApi {
        &self.model_api
    }

    /// Returns a project API handler.
    pub fn project(&self) -> &ProjectApi {
        &self.project_api
    }

    /// Returns a storage API handler.
    pub fn storage(&self) -> &StorageApi {
        &self.storage_api
    }

    /// Returns a mutable storage API handler.
    pub fn storage_mut(&mut self) -> &mut StorageApi {
        &mut self.storage_api
    }
}

pub(crate) fn urlencode<T: AsRef<str>>(s: T) -> String {
    url::form_urlencoded::byte_serialize(s.as_ref().as_bytes()).collect()
}

use crate::error::{NestedResponseError, ResponseError};

async fn process_response<T: for<'de> Deserialize<'de>>(resp: Response) -> Result<T, BQError> {
    let status = resp.status();
    let body = resp.text().await.map_err(reqwest::Error::from)?;

    if status.is_success() {
        serde_json::from_str(&body).map_err(BQError::SerializationError)
    } else {
        // Try to parse the error body as a structured ResponseError.
        // If that fails (empty body, HTML error page, etc.), synthesize a
        // ResponseError that includes the HTTP status and raw body text
        // so the caller can diagnose the issue (e.g., permission errors).
        match serde_json::from_str::<ResponseError>(&body) {
            Ok(error) => Err(BQError::ResponseError { error }),
            Err(_) => Err(BQError::ResponseError {
                error: ResponseError {
                    error: NestedResponseError {
                        code: status.as_u16() as i64,
                        errors: vec![],
                        message: if body.is_empty() {
                            format!(
                                "HTTP {} {} (empty response body)",
                                status.as_u16(),
                                status.canonical_reason().unwrap_or("")
                            )
                        } else {
                            format!(
                                "HTTP {} {}: {}",
                                status.as_u16(),
                                status.canonical_reason().unwrap_or(""),
                                body
                            )
                        },
                        status: status.to_string(),
                    },
                },
            }),
        }
    }
}

pub fn env_vars() -> (String, String, String, String) {
    let project_id = env::var("PROJECT_ID").expect("Environment variable PROJECT_ID");
    let dataset_id = env::var("DATASET_ID").expect("Environment variable DATASET_ID");
    let table_id = env::var("TABLE_ID").expect("Environment variable TABLE_ID");
    let gcp_sa_key =
        env::var("GOOGLE_APPLICATION_CREDENTIALS").expect("Environment variable GOOGLE_APPLICATION_CREDENTIALS");

    (project_id, dataset_id, table_id, gcp_sa_key)
}

pub mod google {
    #![allow(clippy::all)]
    #[path = "google.api.rs"]
    pub mod api;

    #[path = ""]
    pub mod cloud {
        #[path = ""]
        pub mod bigquery {
            #[path = ""]
            pub mod storage {
                #![allow(clippy::all)]
                #[path = "google.cloud.bigquery.storage.v1.rs"]
                pub mod v1;
            }
        }
    }

    #[path = "google.rpc.rs"]
    pub mod rpc;
}

#[cfg(test)]
mod process_response_tests {
    use super::*;
    use crate::error::BQError;
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Helper: start a mock server, register a response, make a GET request,
    /// and pass the reqwest::Response to process_response.
    async fn call_process_response(status: u16, body: &str) -> Result<serde_json::Value, BQError> {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(status).set_body_string(body.to_string()))
            .mount(&server)
            .await;

        let resp = reqwest::get(&server.uri()).await.unwrap();
        process_response(resp).await
    }

    // ─── Success Path ───

    #[tokio::test]
    async fn test_success_with_valid_json_body() {
        // A normal success response with a JSON body should deserialize correctly.
        let result = call_process_response(200, r#"{"kind":"bigquery#job","status":{"state":"DONE"}}"#).await;

        assert!(result.is_ok(), "Should deserialize valid JSON body");
        let val = result.unwrap();
        assert_eq!(val["kind"], "bigquery#job");
    }

    #[tokio::test]
    async fn test_success_with_empty_body_returns_serialization_error() {
        // The BQ cancel_job API sometimes returns 200 OK with an empty body
        // (especially for script child jobs). process_response should return
        // a SerializationError so callers can distinguish this from real errors.
        let result = call_process_response(200, "").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_debug = format!("{:?}", err);
        assert!(
            err_debug.contains("SerializationError"),
            "Empty success body should produce SerializationError, got: {}",
            err_debug
        );
    }

    #[tokio::test]
    async fn test_success_with_invalid_json_body_returns_serialization_error() {
        // A success response with garbage body should return SerializationError.
        let result = call_process_response(200, "not valid json{{{").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        let err_debug = format!("{:?}", err);
        assert!(
            err_debug.contains("SerializationError"),
            "Invalid JSON body should produce SerializationError, got: {}",
            err_debug
        );
    }

    // ─── Error Path: Structured JSON ───

    #[tokio::test]
    async fn test_error_with_structured_json_body() {
        // A 403 response with a structured BQ error JSON body should parse
        // into a ResponseError with the actual error details.
        let body = r#"{
            "error": {
                "code": 403,
                "message": "Access Denied: Job project-123:job-456. The user does not have bigquery.jobs.update permission.",
                "errors": [{"message": "Access Denied", "domain": "global", "reason": "accessDenied"}],
                "status": "PERMISSION_DENIED"
            }
        }"#;
        let result = call_process_response(403, body).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BQError::ResponseError { error } => {
                assert_eq!(error.error.code, 403);
                assert!(
                    error.error.message.contains("Access Denied"),
                    "Should contain actual BQ error message, got: {}",
                    error.error.message
                );
                assert!(
                    error.error.message.contains("bigquery.jobs.update"),
                    "Should contain permission name, got: {}",
                    error.error.message
                );
            }
            other => panic!("Expected ResponseError, got: {:?}", other),
        }
    }

    // ─── Error Path: Empty or Non-JSON Body ───

    #[tokio::test]
    async fn test_error_with_empty_body_synthesizes_response_error() {
        // A 403 response with an empty body (can happen with some proxy configs)
        // should synthesize a ResponseError with the HTTP status code and a
        // descriptive message instead of a cryptic decode error.
        let result = call_process_response(403, "").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BQError::ResponseError { error } => {
                assert_eq!(error.error.code, 403);
                assert!(
                    error.error.message.contains("HTTP 403"),
                    "Should contain HTTP status in message, got: {}",
                    error.error.message
                );
                assert!(
                    error.error.message.contains("empty response body"),
                    "Should mention empty body, got: {}",
                    error.error.message
                );
            }
            other => panic!("Expected ResponseError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_error_with_html_body_synthesizes_response_error() {
        // Some load balancers/proxies return HTML error pages.
        // process_response should include the raw HTML in the error message
        // so operators can diagnose the issue.
        let html = "<html><body><h1>502 Bad Gateway</h1></body></html>";
        let result = call_process_response(502, html).await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BQError::ResponseError { error } => {
                assert_eq!(error.error.code, 502);
                assert!(
                    error.error.message.contains("502"),
                    "Should contain status code, got: {}",
                    error.error.message
                );
                assert!(
                    error.error.message.contains("Bad Gateway"),
                    "Should contain raw body, got: {}",
                    error.error.message
                );
            }
            other => panic!("Expected ResponseError, got: {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_error_404_with_empty_body() {
        // A 404 Not Found (e.g., job already deleted) with empty body.
        let result = call_process_response(404, "").await;
        assert!(result.is_err());

        match result.unwrap_err() {
            BQError::ResponseError { error } => {
                assert_eq!(error.error.code, 404);
                assert!(
                    error.error.message.contains("HTTP 404"),
                    "Message should include HTTP 404, got: {}",
                    error.error.message
                );
            }
            other => panic!("Expected ResponseError, got: {:?}", other),
        }
    }

    // ─── Display Format Verification ───

    #[tokio::test]
    async fn test_error_display_format_is_human_readable() {
        // Verify that the Display impl produces a message useful for logs
        // (operators will see this in production error logs).
        let result = call_process_response(403, "").await;
        let err_display = format!("{}", result.unwrap_err());

        // Display should include "Response error" and the synthesized message
        assert!(
            err_display.contains("403"),
            "Display format should include status code for operators, got: {}",
            err_display
        );
    }
}
