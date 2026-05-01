//! Shared bearer-token authentication helpers for PureReason services.

use std::net::{IpAddr, SocketAddr};

use subtle::ConstantTimeEq;

use crate::trust_ops::TrustRole;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiPrincipal {
    pub tenant: String,
    pub role: TrustRole,
}

impl ApiPrincipal {
    pub fn local_admin() -> Self {
        Self {
            tenant: "local".to_string(),
            role: TrustRole::Admin,
        }
    }

    pub fn actor_id(&self) -> String {
        format!("{}:{}", self.tenant, self.role)
    }
}

#[derive(Clone, Debug)]
struct KeyEntry {
    key_bytes: Vec<u8>,
    principal: ApiPrincipal,
}

#[derive(Clone, Debug)]
pub struct ApiKeyRegistry {
    entries: Vec<KeyEntry>,
    pub auth_enabled: bool,
}

impl ApiKeyRegistry {
    pub fn from_env() -> Self {
        Self::from_env_var("PURE_REASON_ACCESS_TOKENS")
    }

    pub fn from_env_var(var_name: &str) -> Self {
        let raw = std::env::var(var_name).unwrap_or_default();
        if raw.trim().is_empty() {
            return Self {
                entries: Vec::new(),
                auth_enabled: false,
            };
        }

        let mut entries = Vec::new();
        for pair in raw.split(',') {
            let pair = pair.trim();
            let parts: Vec<&str> = pair.split(':').map(str::trim).collect();
            let parsed = match parts.as_slice() {
                [tenant, key] if !tenant.is_empty() && !key.is_empty() => Some(ApiPrincipal {
                    tenant: (*tenant).to_string(),
                    role: TrustRole::Operator,
                }),
                [tenant, role, key]
                    if !tenant.is_empty() && !role.is_empty() && !key.is_empty() =>
                {
                    parse_role(role).map(|parsed_role| ApiPrincipal {
                        tenant: (*tenant).to_string(),
                        role: parsed_role,
                    })
                }
                _ => None,
            };

            if let Some(principal) = parsed {
                let key = parts
                    .last()
                    .expect("validated env entry must contain a key");
                entries.push(KeyEntry {
                    key_bytes: key.as_bytes().to_vec(),
                    principal,
                });
            }
        }

        let auth_enabled = !entries.is_empty();
        Self {
            entries,
            auth_enabled,
        }
    }

    pub fn validate(&self, key: &str) -> Option<ApiPrincipal> {
        let candidate = key.as_bytes();
        let mut matched = None;
        for entry in &self.entries {
            let is_match = constant_time_eq_bytes(&entry.key_bytes, candidate);
            if is_match {
                matched = Some(entry.principal.clone());
            }
        }
        matched
    }
}

pub fn ensure_auth_configuration(
    service_name: &str,
    bind: &str,
    auth_enabled: bool,
    allow_unauthenticated: bool,
) -> Result<(), String> {
    if auth_enabled {
        return Ok(());
    }

    let local_bind = is_loopback_bind(bind);
    if local_bind || allow_unauthenticated {
        return Ok(());
    }

    if !allow_unauthenticated {
        return Err(format!(
            "{service_name} requires PURE_REASON_ACCESS_TOKENS to be set for non-local binds. \
             For local loopback use, no tokens are required. For trusted local/dev environments on other bind addresses, rerun with --allow-unauthenticated."
        ));
    }

    Ok(())
}

pub fn is_loopback_bind(bind: &str) -> bool {
    if let Ok(addr) = bind.parse::<SocketAddr>() {
        return addr.ip().is_loopback();
    }

    bind.rsplit_once(':')
        .map(|(host, _)| matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "::1"))
        .unwrap_or(false)
}

pub fn is_disallowed_webhook_host(host: &str) -> bool {
    if host.eq_ignore_ascii_case("localhost") {
        return true;
    }

    host.parse::<IpAddr>()
        .map(is_disallowed_webhook_ip)
        .unwrap_or(false)
}

fn is_disallowed_webhook_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(addr) => {
            addr.is_private()
                || addr.is_loopback()
                || addr.is_link_local()
                || addr.is_broadcast()
                || addr.is_documentation()
                || addr.is_multicast()
                || addr.is_unspecified()
        }
        IpAddr::V6(addr) => {
            addr.is_loopback()
                || addr.is_unique_local()
                || addr.is_unicast_link_local()
                || addr.is_multicast()
                || addr.is_unspecified()
        }
    }
}

fn parse_role(raw: &str) -> Option<TrustRole> {
    match raw.to_lowercase().as_str() {
        "viewer" => Some(TrustRole::Viewer),
        "operator" => Some(TrustRole::Operator),
        "reviewer" => Some(TrustRole::Reviewer),
        "admin" => Some(TrustRole::Admin),
        _ => None,
    }
}

fn constant_time_eq_bytes(expected: &[u8], candidate: &[u8]) -> bool {
    let max_len = expected.len().max(candidate.len());
    let mut diff = (expected.len() ^ candidate.len()) as u8;

    for idx in 0..max_len {
        let left = expected.get(idx).copied().unwrap_or_default();
        let right = candidate.get(idx).copied().unwrap_or_default();
        diff |= left ^ right;
    }

    diff.ct_eq(&0).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_env_disables_auth() {
        let registry = ApiKeyRegistry {
            entries: Vec::new(),
            auth_enabled: false,
        };
        assert!(!registry.auth_enabled);
    }

    #[test]
    fn validates_known_key() {
        let registry = ApiKeyRegistry {
            entries: vec![KeyEntry {
                key_bytes: b"secret123".to_vec(),
                principal: ApiPrincipal {
                    tenant: "tenant1".to_string(),
                    role: TrustRole::Operator,
                },
            }],
            auth_enabled: true,
        };

        assert_eq!(
            registry.validate("secret123"),
            Some(ApiPrincipal {
                tenant: "tenant1".to_string(),
                role: TrustRole::Operator,
            })
        );
        assert_eq!(registry.validate("wrong"), None);
        assert_eq!(registry.validate("secret12"), None);
    }

    #[test]
    fn parses_role_aware_entries() {
        std::env::set_var(
            "PURE_REASON_TEST_ACCESS_TOKENS",
            "tenant1:reviewer:secret123",
        );
        let registry = ApiKeyRegistry::from_env_var("PURE_REASON_TEST_ACCESS_TOKENS");
        let principal = registry.validate("secret123").unwrap();
        assert_eq!(principal.tenant, "tenant1");
        assert_eq!(principal.role, TrustRole::Reviewer);
        std::env::remove_var("PURE_REASON_TEST_ACCESS_TOKENS");
    }

    #[test]
    fn loopback_bind_detection_is_strict() {
        assert!(is_loopback_bind("127.0.0.1:8080"));
        assert!(is_loopback_bind("[::1]:8080"));
        assert!(is_loopback_bind("localhost:8080"));
        assert!(!is_loopback_bind("0.0.0.0:8080"));
        assert!(!is_loopback_bind("192.168.1.10:8080"));
    }

    #[test]
    fn auth_configuration_allows_local_by_default_and_explicit_non_local_opt_in() {
        assert!(ensure_auth_configuration("api", "127.0.0.1:8080", true, false).is_ok());
        assert!(ensure_auth_configuration("api", "127.0.0.1:8080", false, false).is_ok());
        assert!(ensure_auth_configuration("api", "127.0.0.1:8080", false, true).is_ok());
        assert!(ensure_auth_configuration("api", "0.0.0.0:8080", false, false).is_err());
        assert!(ensure_auth_configuration("api", "0.0.0.0:8080", false, true).is_ok());
    }

    #[test]
    fn rejects_local_and_private_webhook_hosts() {
        assert!(is_disallowed_webhook_host("localhost"));
        assert!(is_disallowed_webhook_host("127.0.0.1"));
        assert!(is_disallowed_webhook_host("10.0.0.4"));
        assert!(!is_disallowed_webhook_host("example.com"));
    }
}
