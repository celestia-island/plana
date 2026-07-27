use anyhow::{Error, Result, bail};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, Copy, PartialEq, Eq, Hash)]
pub enum ModelTier {
    Deep,
    Normal,
    Basic,
}

impl ModelTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            ModelTier::Deep => "Deep",
            ModelTier::Normal => "Normal",
            ModelTier::Basic => "Basic",
        }
    }

    pub fn as_tier_str(&self) -> &'static str {
        match self {
            ModelTier::Deep => "deep",
            ModelTier::Normal => "normal",
            ModelTier::Basic => "basic",
        }
    }

    pub fn from_tier_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "deep" => Ok(ModelTier::Deep),
            "normal" => Ok(ModelTier::Normal),
            "basic" => Ok(ModelTier::Basic),
            _ => bail!("invalid ModelTier: {}", s),
        }
    }

    pub fn sort_order(&self) -> u8 {
        match self {
            ModelTier::Normal => 0,
            ModelTier::Deep => 1,
            ModelTier::Basic => 2,
        }
    }

    pub fn sort_order_from_tier_str(tier_str: &str) -> u8 {
        Self::from_tier_str(tier_str)
            .map(|t| t.sort_order())
            .unwrap_or(u8::MAX)
    }

    pub fn all() -> &'static [ModelTier] {
        &[ModelTier::Deep, ModelTier::Normal, ModelTier::Basic]
    }

    pub fn fallback_tiers(&self) -> &'static [ModelTier] {
        match self {
            ModelTier::Basic => &[ModelTier::Normal, ModelTier::Deep],
            ModelTier::Normal => &[ModelTier::Deep, ModelTier::Basic],
            ModelTier::Deep => &[ModelTier::Normal, ModelTier::Basic],
        }
    }
}

impl FromStr for ModelTier {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_tier_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fallback_tiers_basic_goes_up() -> anyhow::Result<()> {
        let tiers: Vec<_> = ModelTier::Basic.fallback_tiers().to_vec();
        assert_eq!(tiers, vec![ModelTier::Normal, ModelTier::Deep]);
        Ok(())
    }

    #[test]
    fn fallback_tiers_normal_up_then_down() -> anyhow::Result<()> {
        let tiers: Vec<_> = ModelTier::Normal.fallback_tiers().to_vec();
        assert_eq!(tiers, vec![ModelTier::Deep, ModelTier::Basic]);
        Ok(())
    }

    #[test]
    fn fallback_tiers_deep_goes_down() -> anyhow::Result<()> {
        let tiers: Vec<_> = ModelTier::Deep.fallback_tiers().to_vec();
        assert_eq!(tiers, vec![ModelTier::Normal, ModelTier::Basic]);
        Ok(())
    }

    #[test]
    fn fallback_tiers_never_includes_self() -> anyhow::Result<()> {
        for tier in ModelTier::all() {
            for fb in tier.fallback_tiers() {
                assert_ne!(
                    *fb, *tier,
                    "{:?} should not appear in its own fallback list",
                    tier
                );
            }
        }
        Ok(())
    }

    #[test]
    fn fallback_tiers_covers_all_other_tiers() -> anyhow::Result<()> {
        for tier in ModelTier::all() {
            let fallbacks: std::collections::HashSet<_> =
                tier.fallback_tiers().iter().copied().collect();
            let others: std::collections::HashSet<_> = ModelTier::all()
                .iter()
                .copied()
                .filter(|t| t != tier)
                .collect();
            assert_eq!(
                fallbacks, others,
                "{:?} fallbacks should cover all other tiers",
                tier
            );
        }
        Ok(())
    }

    #[test]
    fn from_tier_str_case_insensitive() -> anyhow::Result<()> {
        assert_eq!(ModelTier::from_tier_str("deep")?, ModelTier::Deep);
        assert_eq!(ModelTier::from_tier_str("Deep")?, ModelTier::Deep);
        assert_eq!(ModelTier::from_tier_str("DEEP")?, ModelTier::Deep);
        assert_eq!(ModelTier::from_tier_str("normal")?, ModelTier::Normal);
        assert_eq!(ModelTier::from_tier_str("Normal")?, ModelTier::Normal);
        assert_eq!(ModelTier::from_tier_str("basic")?, ModelTier::Basic);
        assert_eq!(ModelTier::from_tier_str("BASIC")?, ModelTier::Basic);
        Ok(())
    }

    #[test]
    fn from_tier_str_rejects_invalid() -> anyhow::Result<()> {
        assert!(ModelTier::from_tier_str("invalid").is_err());
        assert!(ModelTier::from_tier_str("").is_err());
        Ok(())
    }

    #[test]
    fn sort_order_normal_preferred() -> anyhow::Result<()> {
        assert!(ModelTier::Normal.sort_order() < ModelTier::Deep.sort_order());
        assert!(ModelTier::Normal.sort_order() < ModelTier::Basic.sort_order());
        Ok(())
    }

    #[test]
    fn sort_order_from_tier_str_matches_sort_order() -> anyhow::Result<()> {
        for tier in ModelTier::all() {
            assert_eq!(
                ModelTier::sort_order_from_tier_str(tier.as_tier_str()),
                tier.sort_order()
            );
        }
        Ok(())
    }

    #[test]
    fn sort_order_from_tier_str_unknown_returns_max() -> anyhow::Result<()> {
        assert_eq!(ModelTier::sort_order_from_tier_str("unknown"), u8::MAX);
        Ok(())
    }

    #[test]
    fn all_returns_three_variants() -> anyhow::Result<()> {
        assert_eq!(ModelTier::all().len(), 3);
        assert!(ModelTier::all().contains(&ModelTier::Deep));
        assert!(ModelTier::all().contains(&ModelTier::Normal));
        assert!(ModelTier::all().contains(&ModelTier::Basic));
        Ok(())
    }

    #[test]
    fn as_tier_str_roundtrip() -> anyhow::Result<()> {
        for tier in ModelTier::all() {
            assert_eq!(ModelTier::from_tier_str(tier.as_tier_str())?, *tier);
        }
        Ok(())
    }
}
