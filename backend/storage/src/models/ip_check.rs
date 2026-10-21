use serde::{Deserialize, Serialize};
use sqlx::types::ipnetwork::IpNetwork;
use utoipa::ToSchema;

/// The services an ip address can be checked with.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, ToSchema, PartialEq, Eq)]
#[sqlx(type_name = "ip_check_provider_enum", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IpCheckProvider {
    IpApi,
    Proxycheck,
    IpapiIs,
    Ipqualityscore,
    Vpnapi,
}

/// What a provider tells about an ip address. Each field is `None` when the provider does not
/// report it.
#[derive(Debug, Default, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct IpCheckResult {
    /// ISO 3166-1 alpha-2
    pub country_code: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    /// e.g. `AS15169`
    pub asn: Option<String>,
    pub organization: Option<String>,
    pub is_vpn: Option<bool>,
    pub is_proxy: Option<bool>,
    pub is_tor: Option<bool>,
    /// The address belongs to a datacenter or a hosting company.
    pub is_hosting: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq)]
pub struct IpCheck {
    pub provider: IpCheckProvider,
    pub result: IpCheckResult,
}

/// The registration ip of a user and its stored check, every check field is `None` when it was
/// not checked.
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserRegistrationIp {
    #[schema(value_type = String)]
    pub registered_from_ip: IpNetwork,
    pub provider: Option<IpCheckProvider>,
    pub country_code: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub asn: Option<String>,
    pub organization: Option<String>,
    pub is_vpn: Option<bool>,
    pub is_proxy: Option<bool>,
    pub is_tor: Option<bool>,
    pub is_hosting: Option<bool>,
}
