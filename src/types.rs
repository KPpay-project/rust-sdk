use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GatewayEnvironment {
    Sandbox,
    Production,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    RequiresPaymentMethod,
    RequiresAction,
    Processing,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptStatus {
    Created,
    Dispatching,
    Pending,
    RequiresAction,
    OutcomeUnknown,
    CaptureConfirmed,
    DefinitivelyFailed,
    Cancelled,
    Succeeded,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectionScenario {
    Success,
    Decline,
    Pending,
    Unknown,
    LostResponse,
    Duplicate,
    OutOfOrder,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferScenario {
    Success,
    Decline,
    Pending,
    Unknown,
    LostResponse,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferPurpose {
    SupplierPayment,
    ContractorPayment,
    EmployeePayment,
    CustomerPayout,
    OtherDisbursement,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    MtnMomo,
    MoovMoney,
    #[serde(untagged)]
    Other(String),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    MobileMoney,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckoutSessionRequest {
    pub amount_minor: String,
    pub currency: String,
    pub collection_country: String,
    pub merchant_reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_configuration_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_types: Option<Vec<PaymentMethod>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_reference: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_url_alias: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CollectionRequest {
    pub amount_minor: String,
    pub currency: String,
    pub collection_country: String,
    pub merchant_reference: String,
    pub operator: String,
    pub buyer_phone: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_configuration_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payment_method_types: Option<Vec<PaymentMethod>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_reference: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_scenario: Option<CollectionScenario>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CheckoutSession {
    pub id: String,
    pub payment_intent_id: String,
    pub url: String,
    pub expires_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PaymentAttempt {
    pub id: String,
    pub status: AttemptStatus,
    pub operator: String,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PaymentIntent {
    pub id: String,
    pub merchant_id: String,
    pub application_id: String,
    pub environment: GatewayEnvironment,
    pub payment_configuration_id: String,
    pub amount_minor: String,
    pub currency: String,
    pub collection_country: String,
    pub merchant_reference: String,
    pub customer_reference: Option<String>,
    pub payment_method_types: Vec<PaymentMethod>,
    pub status: PaymentStatus,
    pub accounting_status: String,
    pub funds_availability: String,
    pub settlement_status: String,
    pub refunded_amount_minor: String,
    pub dispute_status: String,
    pub resource_version: u64,
    pub created_at: String,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
    pub latest_attempt: Option<PaymentAttempt>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct PaymentIntentFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merchant_reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub currency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<PaymentStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PaymentIntentPage {
    pub data: Vec<PaymentIntent>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Recipient {
    pub name: String,
    pub phone: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DirectTransferRequest {
    pub payment_configuration_id: String,
    pub operator: String,
    pub recipient: Recipient,
    pub amount_minor: String,
    pub merchant_transfer_reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<TransferPurpose>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_scenario: Option<TransferScenario>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ApprovedTransferRequest {
    pub transfer_grant_id: String,
    pub beneficiary_id: String,
    pub amount_minor: String,
    pub purpose: TransferPurpose,
    pub merchant_transfer_reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_scenario: Option<TransferScenario>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum TransferRequest {
    Direct(DirectTransferRequest),
    Approved(ApprovedTransferRequest),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Transfer {
    pub id: String,
    pub merchant_id: String,
    pub application_id: String,
    pub payment_configuration_id: String,
    pub transfer_grant_id: Option<String>,
    pub beneficiary_id: String,
    pub reservation_id: String,
    pub environment: GatewayEnvironment,
    pub purpose: TransferPurpose,
    pub merchant_transfer_reference: String,
    pub amount_minor: String,
    pub fee_minor: String,
    pub pricing_policy: String,
    pub currency: String,
    pub country: String,
    pub operator: String,
    pub status: String,
    pub funding_status: String,
    pub resource_version: u64,
    pub created_at: String,
    pub disbursement_policy_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TransferPage {
    pub data: Vec<Transfer>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RefundRequest {
    pub payment_intent_id: String,
    pub amount_minor: String,
    pub merchant_refund_reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Refund {
    pub id: String,
    pub payment_intent_id: String,
    pub amount_minor: String,
    pub currency: String,
    pub merchant_refund_reference: String,
    pub status: String,
    pub resource_version: u64,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct GatewayIdentity {
    pub merchant_id: String,
    pub application_id: String,
    pub environment: GatewayEnvironment,
    pub scopes: Vec<String>,
    pub payment_configuration_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Data<T> {
    pub data: T,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CapabilityOperator {
    pub operator: String,
    #[serde(default)]
    pub required_inputs: Vec<String>,
    #[serde(default)]
    pub amount_ranges: Vec<AmountRange>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AmountRange {
    pub minimum_minor: String,
    pub maximum_minor: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Capability {
    pub payment_configuration_id: String,
    pub country: String,
    pub currency: String,
    pub method: String,
    pub available: bool,
    #[serde(default)]
    pub operators: Vec<CapabilityOperator>,
    pub currency_precision: Option<u32>,
    pub profile_version: Option<u64>,
    pub restriction_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Capabilities {
    pub capabilities: Vec<Capability>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Limit {
    pub payment_configuration_id: String,
    pub minimum_minor: Option<String>,
    pub maximum_minor: Option<String>,
    pub daily_amount_minor: Option<String>,
    pub daily_count: Option<u64>,
    pub reserved_amount_minor: Option<String>,
    pub reserved_count: Option<u64>,
    pub currency_precision: Option<u32>,
    pub period: Option<String>,
    pub restriction_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Limits {
    pub limits: Vec<Limit>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Origin {
    pub id: String,
    pub application_id: String,
    pub environment: GatewayEnvironment,
    pub payment_configuration_id: String,
    pub collection_country: String,
    pub currency: String,
    pub inserted_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OriginList {
    pub data: Vec<Origin>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct OriginCount {
    pub count: u64,
}
