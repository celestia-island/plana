#[cfg(target_os = "linux")]
pub struct LandlockRules {
    allowed_paths: Vec<String>,
    read_only_paths: Vec<String>,
}

#[cfg(target_os = "linux")]
impl LandlockRules {
    pub fn new() -> Self {
        Self {
            allowed_paths: Vec::new(),
            read_only_paths: Vec::new(),
        }
    }

    pub fn allow_read_write(mut self, path: &str) -> Self {
        self.allowed_paths.push(path.to_string());
        self
    }

    pub fn allow_read_only(mut self, path: &str) -> Self {
        self.read_only_paths.push(path.to_string());
        self
    }

    pub fn workspace_sandbox(workspace_root: &str) -> Self {
        Self::new()
            .allow_read_write(workspace_root)
            .allow_read_write("/tmp")
            .allow_read_write("/dev/null")
            .allow_read_write("/dev/zero")
            .allow_read_write("/dev/urandom")
            .allow_read_write("/dev/random")
            .allow_read_only("/usr")
            .allow_read_only("/lib")
            .allow_read_only("/lib64")
            .allow_read_only("/etc/alternatives")
            .allow_read_only("/etc/ssl")
            .allow_read_only("/etc/resolv.conf")
            .allow_read_only("/etc/hosts")
            .allow_read_only("/etc/nsswitch.conf")
            .allow_read_only("/etc/passwd")
            .allow_read_only("/etc/group")
            .allow_read_only("/proc/self")
    }

    pub fn allowed_paths(&self) -> &[String] {
        &self.allowed_paths
    }

    pub fn read_only_paths(&self) -> &[String] {
        &self.read_only_paths
    }
}

#[cfg(target_os = "linux")]
impl Default for LandlockRules {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(target_os = "linux"))]
pub struct LandlockRules;

#[cfg(not(target_os = "linux"))]
impl LandlockRules {
    pub fn new() -> Self {
        Self
    }

    pub fn workspace_sandbox(_workspace_root: &str) -> Self {
        Self
    }
}

#[cfg(not(target_os = "linux"))]
impl Default for LandlockRules {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_sandbox_allows_workspace_rw() {
        let rules = LandlockRules::workspace_sandbox("/workspace");
        #[cfg(target_os = "linux")]
        {
            assert!(rules.allowed_paths().contains(&"/workspace".to_string()));
            assert!(rules.allowed_paths().contains(&"/tmp".to_string()));
            assert!(rules.read_only_paths().contains(&"/usr".to_string()));
            assert!(rules.read_only_paths().contains(&"/etc/passwd".to_string()));
        }
    }

    #[test]
    fn default_rules_empty() {
        let rules = LandlockRules::new();
        #[cfg(target_os = "linux")]
        {
            assert!(rules.allowed_paths().is_empty());
            assert!(rules.read_only_paths().is_empty());
        }
    }
}
