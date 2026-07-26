use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use ts_rs::TS;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS, JsonSchema)]
#[ts(export, export_to = "region.ts")]
#[serde(rename_all = "snake_case")]
pub enum RegionPolicy {
    China {
        icp_number: Option<String>,
        #[serde(default = "default_true")]
        data_residency_required: bool,
        #[serde(default = "default_true")]
        payment_domestic_only: bool,
        #[serde(default = "default_true")]
        pipi_consent_required: bool,
    },
    EuropeanUnion {
        #[serde(default = "default_true")]
        gdpr_consent_required: bool,
        #[serde(default = "default_true")]
        right_to_deletion: bool,
        #[serde(default = "default_true")]
        right_to_export: bool,
        dpo_contact: Option<String>,
        #[serde(default = "default_true")]
        cookie_consent_required: bool,
    },
    Russia {
        #[serde(default = "default_true")]
        data_localization_required: bool,
        #[serde(default)]
        payment_mir_supported: bool,
    },
    FreeMarket,
}

fn default_true() -> bool {
    true
}

impl Default for RegionPolicy {
    fn default() -> Self {
        RegionPolicy::FreeMarket
    }
}

impl RegionPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            RegionPolicy::China { .. } => "china",
            RegionPolicy::EuropeanUnion { .. } => "european_union",
            RegionPolicy::Russia { .. } => "russia",
            RegionPolicy::FreeMarket => "free_market",
        }
    }

    pub fn all() -> Vec<RegionPolicy> {
        vec![
            RegionPolicy::China {
                icp_number: None,
                data_residency_required: true,
                payment_domestic_only: true,
                pipi_consent_required: true,
            },
            RegionPolicy::EuropeanUnion {
                gdpr_consent_required: true,
                right_to_deletion: true,
                right_to_export: true,
                dpo_contact: None,
                cookie_consent_required: true,
            },
            RegionPolicy::Russia {
                data_localization_required: true,
                payment_mir_supported: false,
            },
            RegionPolicy::FreeMarket,
        ]
    }
}

impl fmt::Display for RegionPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for RegionPolicy {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "china" => Ok(RegionPolicy::China {
                icp_number: None,
                data_residency_required: true,
                payment_domestic_only: true,
                pipi_consent_required: true,
            }),
            "european_union" => Ok(RegionPolicy::EuropeanUnion {
                gdpr_consent_required: true,
                right_to_deletion: true,
                right_to_export: true,
                dpo_contact: None,
                cookie_consent_required: true,
            }),
            "russia" => Ok(RegionPolicy::Russia {
                data_localization_required: true,
                payment_mir_supported: false,
            }),
            "free_market" => Ok(RegionPolicy::FreeMarket),
            _ => Err(format!("unknown region policy: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn china_serde_round_trip() {
        let policy = RegionPolicy::China {
            icp_number: Some("ICP-12345".into()),
            data_residency_required: true,
            payment_domestic_only: true,
            pipi_consent_required: true,
        };
        let json = serde_json::to_string(&policy).unwrap();
        let back: RegionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, policy);
    }

    #[test]
    fn european_union_serde_round_trip() {
        let policy = RegionPolicy::EuropeanUnion {
            gdpr_consent_required: true,
            right_to_deletion: true,
            right_to_export: false,
            dpo_contact: Some("dpo@example.com".into()),
            cookie_consent_required: true,
        };
        let json = serde_json::to_string(&policy).unwrap();
        let back: RegionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, policy);
    }

    #[test]
    fn russia_serde_round_trip() {
        let policy = RegionPolicy::Russia {
            data_localization_required: true,
            payment_mir_supported: true,
        };
        let json = serde_json::to_string(&policy).unwrap();
        let back: RegionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, policy);
    }

    #[test]
    fn free_market_serde_round_trip() {
        let policy = RegionPolicy::FreeMarket;
        let json = serde_json::to_string(&policy).unwrap();
        let back: RegionPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(back, policy);
    }

    #[test]
    fn default_is_free_market() {
        assert_eq!(RegionPolicy::default(), RegionPolicy::FreeMarket);
    }

    #[test]
    fn display_from_str_roundtrip() {
        for policy in RegionPolicy::all() {
            let s = policy.to_string();
            let back: RegionPolicy = s.parse().unwrap();
            assert_eq!(back, policy, "roundtrip failed for {s}");
        }
    }

    #[test]
    fn from_str_rejects_unknown() {
        assert!(RegionPolicy::from_str("unknown").is_err());
        assert!(RegionPolicy::from_str("").is_err());
    }

    #[test]
    fn all_returns_four_unique_variants() {
        let all = RegionPolicy::all();
        assert_eq!(
            all.len(),
            4,
            "RegionPolicy::all() must return exactly 4 variants"
        );
        let mut seen = std::collections::HashSet::new();
        for p in &all {
            let s = p.as_str();
            assert!(seen.insert(s), "duplicate region policy: {s}");
        }
    }

    #[test]
    fn china_default_flags() {
        let policy = RegionPolicy::China {
            icp_number: None,
            data_residency_required: true,
            payment_domestic_only: true,
            pipi_consent_required: true,
        };
        match policy {
            RegionPolicy::China {
                data_residency_required,
                payment_domestic_only,
                pipi_consent_required,
                icp_number,
            } => {
                assert!(data_residency_required);
                assert!(payment_domestic_only);
                assert!(pipi_consent_required);
                assert!(icp_number.is_none());
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn eu_default_flags() {
        let policy = RegionPolicy::EuropeanUnion {
            gdpr_consent_required: true,
            right_to_deletion: true,
            right_to_export: true,
            dpo_contact: None,
            cookie_consent_required: true,
        };
        match policy {
            RegionPolicy::EuropeanUnion {
                gdpr_consent_required,
                right_to_deletion,
                right_to_export,
                cookie_consent_required,
                dpo_contact,
            } => {
                assert!(gdpr_consent_required);
                assert!(right_to_deletion);
                assert!(right_to_export);
                assert!(cookie_consent_required);
                assert!(dpo_contact.is_none());
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn russia_default_flags() {
        let policy = RegionPolicy::Russia {
            data_localization_required: true,
            payment_mir_supported: false,
        };
        match policy {
            RegionPolicy::Russia {
                data_localization_required,
                payment_mir_supported,
            } => {
                assert!(data_localization_required);
                assert!(!payment_mir_supported);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn china_deser_fills_defaults() {
        let json = r#"{"china": {"icp_number": null}}"#;
        let policy: RegionPolicy = serde_json::from_str(json).unwrap();
        match policy {
            RegionPolicy::China {
                data_residency_required,
                payment_domestic_only,
                pipi_consent_required,
                ..
            } => {
                assert!(data_residency_required);
                assert!(payment_domestic_only);
                assert!(pipi_consent_required);
            }
            _ => panic!("expected China"),
        }
    }

    #[test]
    fn eu_deser_fills_defaults() {
        let json = r#"{"european_union": {}}"#;
        let policy: RegionPolicy = serde_json::from_str(json).unwrap();
        match policy {
            RegionPolicy::EuropeanUnion {
                gdpr_consent_required,
                right_to_deletion,
                right_to_export,
                cookie_consent_required,
                ..
            } => {
                assert!(gdpr_consent_required);
                assert!(right_to_deletion);
                assert!(right_to_export);
                assert!(cookie_consent_required);
            }
            _ => panic!("expected EuropeanUnion"),
        }
    }

    #[test]
    fn russia_deser_fills_defaults() {
        let json = r#"{"russia": {}}"#;
        let policy: RegionPolicy = serde_json::from_str(json).unwrap();
        match policy {
            RegionPolicy::Russia {
                data_localization_required,
                payment_mir_supported,
            } => {
                assert!(data_localization_required);
                assert!(!payment_mir_supported);
            }
            _ => panic!("expected Russia"),
        }
    }

    #[test]
    fn as_str_all_variants() {
        assert_eq!(
            RegionPolicy::China {
                icp_number: None,
                data_residency_required: true,
                payment_domestic_only: true,
                pipi_consent_required: true
            }
            .as_str(),
            "china"
        );
        assert_eq!(
            RegionPolicy::EuropeanUnion {
                gdpr_consent_required: true,
                right_to_deletion: true,
                right_to_export: true,
                dpo_contact: None,
                cookie_consent_required: true
            }
            .as_str(),
            "european_union"
        );
        assert_eq!(
            RegionPolicy::Russia {
                data_localization_required: true,
                payment_mir_supported: false
            }
            .as_str(),
            "russia"
        );
        assert_eq!(RegionPolicy::FreeMarket.as_str(), "free_market");
    }

    #[test]
    fn free_market_serializes_as_snake_case_string() {
        let json = serde_json::to_string(&RegionPolicy::FreeMarket).unwrap();
        assert_eq!(json, r#""free_market""#);
    }
}
