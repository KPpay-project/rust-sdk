use crate::error::{Error, GatewayProblem, Result};
use crate::types::*;
use reqwest::{Client as HttpClient, Method, RequestBuilder, StatusCode, Url, header};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_TIMEOUT: Duration = Duration::from_millis(i32::MAX as u64);

#[derive(Clone)]
pub struct Client {
    client: HttpClient,
    base_url: Url,
    public_key: Arc<str>,
    secret_key: Arc<str>,
    timeout: Duration,
}

impl fmt::Debug for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Client")
            .field("base_url", &self.base_url)
            .field("public_key", &"[redacted]")
            .field("secret_key", &"[redacted]")
            .field("timeout", &self.timeout)
            .finish()
    }
}

#[derive(Clone, Debug)]
pub struct ClientBuilder {
    base_url: String,
    public_key: String,
    secret_key: String,
    timeout: Duration,
    client: Option<HttpClient>,
}

#[derive(Clone, Debug, Default)]
pub struct RequestOptions {
    pub timeout: Option<Duration>,
}

#[derive(Clone, Debug)]
pub struct MutationOptions {
    pub idempotency_key: String,
    pub timeout: Option<Duration>,
}

impl MutationOptions {
    pub fn new(idempotency_key: impl Into<String>) -> Self {
        Self {
            idempotency_key: idempotency_key.into(),
            timeout: None,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

impl ClientBuilder {
    pub fn new(
        base_url: impl Into<String>,
        public_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            public_key: public_key.into(),
            secret_key: secret_key.into(),
            timeout: DEFAULT_TIMEOUT,
            client: None,
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn client(mut self, client: HttpClient) -> Self {
        self.client = Some(client);
        self
    }

    pub fn build(self) -> Result<Client> {
        validate_timeout(self.timeout)?;
        validate_credentials(&self.public_key, &self.secret_key)?;
        let mut base_url = Url::parse(&self.base_url)
            .map_err(|error| Error::Configuration(format!("invalid base URL: {error}")))?;
        validate_base_url(&base_url)?;
        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }
        let client = match self.client {
            Some(client) => client,
            None => HttpClient::builder().build().map_err(Error::Transport)?,
        };
        Ok(Client {
            client,
            base_url,
            public_key: self.public_key.into(),
            secret_key: self.secret_key.into(),
            timeout: self.timeout,
        })
    }
}

impl Client {
    pub fn builder(
        base_url: impl Into<String>,
        public_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> ClientBuilder {
        ClientBuilder::new(base_url, public_key, secret_key)
    }

    pub fn new(
        base_url: impl Into<String>,
        public_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Result<Self> {
        Self::builder(base_url, public_key, secret_key).build()
    }

    pub async fn create_checkout_session(
        &self,
        request: &CheckoutSessionRequest,
    ) -> Result<CheckoutSession> {
        self.post("checkout-sessions", request, None, None).await
    }

    pub async fn create_collection(&self, request: &CollectionRequest) -> Result<PaymentIntent> {
        self.post("collections", request, None, None).await
    }

    pub async fn list_payment_intents(
        &self,
        filters: &PaymentIntentFilters,
    ) -> Result<PaymentIntentPage> {
        self.get_with_query("payment-intents", filters, None).await
    }

    pub async fn get_payment_intent(&self, intent_id: &str) -> Result<PaymentIntent> {
        self.get(&resource_path("payment-intents", intent_id)?, None)
            .await
    }

    pub async fn cancel_payment_intent(
        &self,
        intent_id: &str,
        options: &MutationOptions,
    ) -> Result<PaymentIntent> {
        let path = format!("{}/cancel", resource_path("payment-intents", intent_id)?);
        self.post_empty(&path, options).await
    }

    pub async fn create_transfer(
        &self,
        request: &TransferRequest,
        options: &MutationOptions,
    ) -> Result<Transfer> {
        self.post(
            "transfers",
            request,
            Some(&options.idempotency_key),
            options.timeout,
        )
        .await
    }

    pub async fn list_transfers(&self) -> Result<TransferPage> {
        self.get("transfers", None).await
    }

    pub async fn get_transfer(&self, transfer_id: &str) -> Result<Transfer> {
        self.get(&resource_path("transfers", transfer_id)?, None)
            .await
    }

    pub async fn create_refund(
        &self,
        request: &RefundRequest,
        options: &MutationOptions,
    ) -> Result<Refund> {
        self.post(
            "refunds",
            request,
            Some(&options.idempotency_key),
            options.timeout,
        )
        .await
    }

    pub async fn get_refund(&self, refund_id: &str) -> Result<Refund> {
        self.get(&resource_path("refunds", refund_id)?, None).await
    }

    pub async fn sandbox_identity(&self) -> Result<Data<GatewayIdentity>> {
        self.get("me", None).await
    }

    pub async fn sandbox_capabilities(&self) -> Result<Data<Capabilities>> {
        self.get("capabilities", None).await
    }

    pub async fn sandbox_limits(&self) -> Result<Data<Limits>> {
        self.get("limits", None).await
    }

    pub async fn list_origins(&self, limit: Option<u8>) -> Result<OriginList> {
        if matches!(limit, Some(0 | 101..=u8::MAX)) {
            return Err(Error::Configuration(
                "origin limit must be between 1 and 100".into(),
            ));
        }
        match limit {
            Some(limit) => {
                self.get_with_query("../origins", &[("limit", limit)], None)
                    .await
            }
            None => self.get("../origins", None).await,
        }
    }

    pub async fn count_origins(&self) -> Result<Data<OriginCount>> {
        self.get("../origins/count", None).await
    }

    pub async fn get_origin(&self, origin_id: &str) -> Result<Data<Origin>> {
        self.get(&resource_path("../origins", origin_id)?, None)
            .await
    }

    async fn get<T>(&self, path: &str, timeout: Option<Duration>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let request = self.request(Method::GET, path, timeout)?;
        self.execute(request, timeout).await
    }

    async fn get_with_query<T, Q>(
        &self,
        path: &str,
        query: &Q,
        timeout: Option<Duration>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        Q: Serialize + ?Sized,
    {
        let request = self.request(Method::GET, path, timeout)?.query(query);
        self.execute(request, timeout).await
    }

    async fn post<T, B>(
        &self,
        path: &str,
        body: &B,
        idempotency_key: Option<&str>,
        timeout: Option<Duration>,
    ) -> Result<T>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let mut request = self.request(Method::POST, path, timeout)?.json(body);
        if let Some(key) = idempotency_key {
            validate_idempotency_key(key)?;
            request = request.header("Idempotency-Key", key);
        }
        self.execute(request, timeout).await
    }

    async fn post_empty<T>(&self, path: &str, options: &MutationOptions) -> Result<T>
    where
        T: DeserializeOwned,
    {
        validate_idempotency_key(&options.idempotency_key)?;
        let request = self
            .request(Method::POST, path, options.timeout)?
            .header("Idempotency-Key", &options.idempotency_key);
        self.execute(request, options.timeout).await
    }

    fn request(
        &self,
        method: Method,
        path: &str,
        timeout: Option<Duration>,
    ) -> Result<RequestBuilder> {
        let timeout = timeout.unwrap_or(self.timeout);
        validate_timeout(timeout)?;
        let url = self
            .base_url
            .join(path)
            .map_err(|error| Error::Configuration(format!("invalid request path: {error}")))?;
        Ok(self
            .client
            .request(method, url)
            .basic_auth(self.public_key.as_ref(), Some(self.secret_key.as_ref()))
            .header(header::ACCEPT, "application/json, application/problem+json")
            .timeout(timeout))
    }

    async fn execute<T>(&self, request: RequestBuilder, timeout: Option<Duration>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let timeout = timeout.unwrap_or(self.timeout);
        let response = request.send().await.map_err(|error| {
            if error.is_timeout() {
                Error::Timeout(timeout)
            } else {
                Error::Transport(error)
            }
        })?;
        let status = response.status();
        let retry_after = response
            .headers()
            .get(header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .map(Duration::from_secs);
        let bytes = response.bytes().await.map_err(Error::Transport)?;
        if !status.is_success() {
            return Err(api_error(status, retry_after, &bytes));
        }
        if bytes.is_empty() {
            return Err(Error::EmptyResponse);
        }
        serde_json::from_slice(&bytes).map_err(Error::InvalidResponse)
    }
}

fn validate_credentials(public_key: &str, secret_key: &str) -> Result<()> {
    if public_key.is_empty() || secret_key.is_empty() || public_key.contains(':') {
        return Err(Error::Configuration(
            "a public key without a colon and a secret key are required".into(),
        ));
    }
    Ok(())
}

fn validate_base_url(url: &Url) -> Result<()> {
    let local_http =
        url.scheme() == "http" && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "::1"));
    if url.scheme() != "https" && !local_http {
        return Err(Error::Configuration(
            "base URL must use HTTPS outside local development".into(),
        ));
    }
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(Error::Configuration(
            "base URL cannot contain credentials, a query, or a fragment".into(),
        ));
    }
    Ok(())
}

fn validate_timeout(timeout: Duration) -> Result<()> {
    if timeout.is_zero() || timeout > MAX_TIMEOUT {
        return Err(Error::Configuration(
            "timeout must be positive and no greater than 2147483647 milliseconds".into(),
        ));
    }
    Ok(())
}

fn validate_idempotency_key(key: &str) -> Result<()> {
    if key.is_empty() || key.len() > 255 || !key.bytes().all(|byte| (b'!'..=b'~').contains(&byte)) {
        return Err(Error::Configuration(
            "idempotency key must contain 1 to 255 visible ASCII characters without spaces".into(),
        ));
    }
    Ok(())
}

fn resource_path(prefix: &str, id: &str) -> Result<String> {
    if id.is_empty()
        || id == "."
        || id == ".."
        || id.contains('/')
        || id.contains('?')
        || id.contains('#')
    {
        return Err(Error::Configuration("resource ID is invalid".into()));
    }
    Ok(format!("{prefix}/{id}"))
}

fn api_error(status: StatusCode, retry_after: Option<Duration>, bytes: &[u8]) -> Error {
    let details = serde_json::from_slice::<Value>(bytes).ok();
    let problem = details
        .as_ref()
        .and_then(|value| serde_json::from_value::<GatewayProblem>(value.clone()).ok());
    let message = problem
        .as_ref()
        .and_then(|problem| problem.title.clone().or(problem.error.clone()))
        .unwrap_or_else(|| format!("gateway request failed with HTTP {}", status.as_u16()));
    Error::Api {
        status,
        message,
        code: problem.as_ref().and_then(|problem| problem.code.clone()),
        request_id: problem
            .as_ref()
            .and_then(|problem| problem.request_id.clone()),
        retryable: problem.as_ref().and_then(|problem| problem.retryable),
        retry_after,
        details,
    }
}
