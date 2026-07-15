use std::sync::LazyLock;

use super::defaults::RuntimeTuningConfig;

pub static CONFIG: LazyLock<RuntimeTuningConfig> = LazyLock::new(RuntimeTuningConfig::from_env);
