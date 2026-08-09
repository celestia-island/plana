use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TokenSource {
    LocalStreamQuickAmount(u64),
    LocalTokenizer(u64),
    CloudResponse(u64),
}

impl TokenSource {
    pub fn value(&self) -> u64 {
        match self {
            TokenSource::LocalStreamQuickAmount(v)
            | TokenSource::LocalTokenizer(v)
            | TokenSource::CloudResponse(v) => *v,
        }
    }

    pub fn merge(&mut self, incoming: TokenSource) {
        if incoming >= *self {
            *self = incoming;
        }
    }

    pub fn is_cloud(&self) -> bool {
        matches!(self, TokenSource::CloudResponse(_))
    }

    pub fn is_exact(&self) -> bool {
        matches!(
            self,
            TokenSource::CloudResponse(_) | TokenSource::LocalTokenizer(_)
        )
    }

    pub fn format(&self) -> String {
        let v = self.value();
        let num = format_number(v);
        match self {
            TokenSource::CloudResponse(_) => num,
            TokenSource::LocalTokenizer(_) => format!("≈{}", num),
            TokenSource::LocalStreamQuickAmount(_) => format!("~{}", num),
        }
    }

    pub fn format_with_labels(&self, label_estimated: &str, label_approx: &str) -> String {
        let v = self.value();
        let num = format_number(v);
        match self {
            TokenSource::CloudResponse(_) => num,
            TokenSource::LocalTokenizer(_) => {
                if label_approx.is_empty() {
                    format!("≈{}", num)
                } else {
                    format!("{}{}", label_approx, num)
                }
            }
            TokenSource::LocalStreamQuickAmount(_) => {
                if label_estimated.is_empty() {
                    format!("~{}", num)
                } else {
                    format!("{}{}", label_estimated, num)
                }
            }
        }
    }

    pub fn quick(len_bytes: usize) -> Self {
        TokenSource::LocalStreamQuickAmount((len_bytes as u64).div_ceil(4))
    }
}

pub fn format_number(v: u64) -> String {
    if v >= 1_000_000 {
        format!("{:.1}m", v as f64 / 1_000_000.0)
    } else if v >= 1000 {
        format!("{:.1}k", v as f64 / 1000.0)
    } else {
        v.to_string()
    }
}

impl fmt::Display for TokenSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[derive(Debug, Clone)]
pub struct GroupStats {
    pub input_tokens: Option<TokenSource>,
    pub output_tokens: Option<TokenSource>,
    pub duration_secs: Option<f64>,
    pub tool_count: usize,
    pub exchange_count: Option<u32>,
}
