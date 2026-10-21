pub mod ip_api;
pub mod ipapi_is;
pub mod ipqualityscore;
pub mod proxycheck;
pub mod vpnapi;

use arcadia_common::error::{Error, Result};
use arcadia_storage::models::ip_check::{IpCheck, IpCheckProvider, IpCheckResult};
use serde::Deserialize;
use std::{net::IpAddr, time::Duration};

/// A slow provider must not hold the next ones back for too long.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

type ProviderResult = std::result::Result<IpCheckResult, Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default, Deserialize)]
pub struct IpCheckProvidersConfig {
    pub ip_api: Option<ip_api::IpApiConfig>,
    pub proxycheck: Option<proxycheck::ProxycheckConfig>,
    pub ipapi_is: Option<ipapi_is::IpapiIsConfig>,
    pub ipqualityscore: Option<ipqualityscore::IpqualityscoreConfig>,
    pub vpnapi: Option<vpnapi::VpnapiConfig>,
}

enum Provider<'a> {
    IpApi,
    Proxycheck(&'a proxycheck::ProxycheckConfig),
    IpapiIs(&'a ipapi_is::IpapiIsConfig),
    Ipqualityscore(&'a ipqualityscore::IpqualityscoreConfig),
    Vpnapi(&'a vpnapi::VpnapiConfig),
}

impl Provider<'_> {
    fn kind(&self) -> IpCheckProvider {
        match self {
            Provider::IpApi => IpCheckProvider::IpApi,
            Provider::Proxycheck(_) => IpCheckProvider::Proxycheck,
            Provider::IpapiIs(_) => IpCheckProvider::IpapiIs,
            Provider::Ipqualityscore(_) => IpCheckProvider::Ipqualityscore,
            Provider::Vpnapi(_) => IpCheckProvider::Vpnapi,
        }
    }

    async fn check(&self, client: &reqwest::Client, ip: IpAddr) -> ProviderResult {
        match self {
            Provider::IpApi => ip_api::check(client, ip).await,
            Provider::Proxycheck(config) => proxycheck::check(client, config, ip).await,
            Provider::IpapiIs(config) => ipapi_is::check(client, config, ip).await,
            Provider::Ipqualityscore(config) => ipqualityscore::check(client, config, ip).await,
            Provider::Vpnapi(config) => vpnapi::check(client, config, ip).await,
        }
    }
}

impl IpCheckProvidersConfig {
    pub fn is_enabled(&self) -> bool {
        !self.providers_by_priority().is_empty()
    }

    fn providers_by_priority(&self) -> Vec<Provider<'_>> {
        let mut providers: Vec<(u32, Provider)> = [
            self.ip_api
                .as_ref()
                .map(|config| (config.priority, Provider::IpApi)),
            self.proxycheck
                .as_ref()
                .map(|config| (config.priority, Provider::Proxycheck(config))),
            self.ipapi_is
                .as_ref()
                .map(|config| (config.priority, Provider::IpapiIs(config))),
            self.ipqualityscore
                .as_ref()
                .map(|config| (config.priority, Provider::Ipqualityscore(config))),
            self.vpnapi
                .as_ref()
                .map(|config| (config.priority, Provider::Vpnapi(config))),
        ]
        .into_iter()
        .flatten()
        .collect();
        providers.sort_by_key(|(priority, _)| *priority);
        providers
            .into_iter()
            .map(|(_, provider)| provider)
            .collect()
    }

    /// The result of the first provider that succeeds.
    pub async fn check_ip(&self, client: &reqwest::Client, ip: IpAddr) -> Result<IpCheck> {
        if !is_public(ip) {
            return Err(Error::IpNotPublic);
        }
        for provider in self.providers_by_priority() {
            match provider.check(client, ip).await {
                Ok(result) => {
                    return Ok(IpCheck {
                        provider: provider.kind(),
                        result,
                    })
                }
                // the url of the request holds the api key of the provider
                Err(error) => match error.downcast::<reqwest::Error>() {
                    Ok(error) => log::warn!(
                        "ip check with {:?} failed: {}",
                        provider.kind(),
                        error.without_url()
                    ),
                    Err(error) => {
                        log::warn!("ip check with {:?} failed: {error}", provider.kind())
                    }
                },
            }
        }
        Err(Error::IpCheckFailed)
    }
}

/// Providers cannot tell anything about private, loopback, etc. addresses: checking them would
/// only use their quotas up.
pub fn is_public(ip: IpAddr) -> bool {
    match ip.to_canonical() {
        IpAddr::V4(ip) => {
            !(ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
                || ip.is_broadcast()
                || ip.is_documentation()
                // shared address space (carrier-grade NAT), 100.64.0.0/10
                || (ip.octets()[0] == 100 && ip.octets()[1] & 0b1100_0000 == 64))
        }
        IpAddr::V6(ip) => {
            !(ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_multicast())
        }
    }
}

/// Providers send empty strings for the information they do not have.
fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn providers_are_ordered_by_priority() {
        let config: IpCheckProvidersConfig = serde_json::from_str(
            r#"{
                "ip_api": {"priority": 3},
                "proxycheck": {"priority": 1},
                "vpnapi": {"priority": 2, "api_key": "key"}
            }"#,
        )
        .unwrap();

        let kinds: Vec<IpCheckProvider> = config
            .providers_by_priority()
            .iter()
            .map(Provider::kind)
            .collect();

        assert_eq!(
            kinds,
            [
                IpCheckProvider::Proxycheck,
                IpCheckProvider::Vpnapi,
                IpCheckProvider::IpApi
            ]
        );
    }

    #[test]
    fn only_public_ips_are_checked() {
        for ip in ["8.8.8.8", "2001:4860:4860::8888", "::ffff:8.8.8.8"] {
            assert!(is_public(ip.parse().unwrap()), "{ip}");
        }
        for ip in [
            "10.10.4.88",
            "172.18.0.2",
            "192.168.1.1",
            "127.0.0.1",
            "100.64.0.1",
            "169.254.1.1",
            "::1",
            "fd00::1",
            "fe80::1",
            "::ffff:10.0.0.1",
        ] {
            assert!(!is_public(ip.parse().unwrap()), "{ip}");
        }
    }

    #[test]
    fn no_provider_configured_is_disabled() {
        assert!(!IpCheckProvidersConfig::default().is_enabled());
    }
}
