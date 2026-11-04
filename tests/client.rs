use kppay::{Client, Error, MutationOptions};
use reqwest::StatusCode;
use serde_json::json;
use std::time::Duration;
use wiremock::matchers::{basic_auth, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn reads_sandbox_identity_with_basic_auth() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/gateway/sandbox/v1/me"))
        .and(basic_auth("pk_test", "sk_test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "data": {
                "merchant_id": "merchant_1",
                "application_id": "application_1",
                "environment": "sandbox",
                "scopes": ["payments:create"],
                "payment_configuration_ids": ["configuration_1"]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let gateway = Client::new(
        format!("{}/api/gateway/sandbox/v1", server.uri()),
        "pk_test",
        "sk_test",
    )
    .unwrap();
    let identity = gateway.sandbox_identity().await.unwrap();

    assert_eq!(identity.data.merchant_id, "merchant_1");
    assert_eq!(identity.data.application_id, "application_1");
}

#[tokio::test]
async fn sends_idempotency_key_for_cancellation() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/payment-intents/intent_1/cancel"))
        .and(header("Idempotency-Key", "cancel-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payment_intent()))
        .expect(1)
        .mount(&server)
        .await;

    let gateway = Client::new(format!("{}/v1", server.uri()), "pk_test", "sk_test").unwrap();
    let intent = gateway
        .cancel_payment_intent("intent_1", &MutationOptions::new("cancel-1"))
        .await
        .unwrap();

    assert_eq!(intent.id, "intent_1");
}

#[tokio::test]
async fn exposes_structured_api_errors() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/me"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "7")
                .set_body_json(json!({
                    "type": "about:blank",
                    "title": "Request budget exhausted",
                    "status": 429,
                    "code": "rate_limited",
                    "request_id": "request_1",
                    "retryable": true
                })),
        )
        .mount(&server)
        .await;

    let gateway = Client::new(format!("{}/v1", server.uri()), "pk_test", "sk_test").unwrap();
    let error = gateway.sandbox_identity().await.unwrap_err();

    assert_eq!(error.status(), Some(StatusCode::TOO_MANY_REQUESTS));
    assert_eq!(error.code(), Some("rate_limited"));
    assert_eq!(error.request_id(), Some("request_1"));
    assert_eq!(error.retryable(), Some(true));
    assert_eq!(error.retry_after(), Some(Duration::from_secs(7)));
}

#[tokio::test]
async fn validates_security_sensitive_configuration() {
    let insecure = Client::new("http://gateway.example.com/v1", "pk", "sk").unwrap_err();
    assert!(matches!(insecure, Error::Configuration(_)));

    let colon = Client::new("https://gateway.example.com/v1", "pk:bad", "sk").unwrap_err();
    assert!(matches!(colon, Error::Configuration(_)));
}

#[tokio::test]
async fn validates_origin_limit_without_sending_a_request() {
    let gateway = Client::new(
        "https://gateway.example.com/api/gateway/sandbox/v1",
        "pk",
        "sk",
    )
    .unwrap();
    let error = gateway.list_origins(Some(0)).await.unwrap_err();
    assert!(matches!(error, Error::Configuration(_)));
}

fn payment_intent() -> serde_json::Value {
    json!({
        "id": "intent_1",
        "merchant_id": "merchant_1",
        "application_id": "application_1",
        "environment": "sandbox",
        "payment_configuration_id": "configuration_1",
        "amount_minor": "1000",
        "currency": "XOF",
        "collection_country": "BJ",
        "merchant_reference": "order_1",
        "customer_reference": null,
        "payment_method_types": ["mobile_money"],
        "status": "cancelled",
        "accounting_status": "not_posted",
        "funds_availability": "not_eligible",
        "settlement_status": "not_eligible",
        "refunded_amount_minor": "0",
        "dispute_status": "none",
        "resource_version": 2,
        "created_at": "2026-10-22T10:00:00Z",
        "metadata": {},
        "latest_attempt": null
    })
}
