use kirino::rbac::permission::Permission;

pub fn parse_permission(s: &str) -> Option<Permission> {
    Permission::from_path(s)
}

pub fn validate_grant_permissions(grants: &[String]) -> Result<(), Vec<String>> {
    let invalid: Vec<String> = grants
        .iter()
        .filter(|g| Permission::from_path(g).is_none() && Permission::expand_domain(g).is_empty())
        .cloned()
        .collect();
    if invalid.is_empty() {
        Ok(())
    } else {
        Err(invalid)
    }
}

pub fn list_all_permission_names() -> Vec<String> {
    Permission::all().iter().map(|p| p.name().to_string()).collect()
}

pub fn list_all_domain_names() -> Vec<&'static str> {
    Permission::all_domains()
}

pub fn expand_domain(domain: &str) -> Vec<String> {
    Permission::expand_domain(domain)
        .iter()
        .map(|p| p.name().to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_leaf() {
        assert!(parse_permission("agent.read").is_some());
        assert!(parse_permission("workspace.manage").is_some());
        assert!(parse_permission("provider.use").is_some());
    }

    #[test]
    fn parse_invalid() {
        assert!(parse_permission("nonexistent.action").is_none());
        assert!(parse_permission("").is_none());
        assert!(parse_permission("agent.invalid").is_none());
    }

    #[test]
    fn validate_grants_ok() {
        let grants = vec!["agent.read".into(), "workspace.manage".into()];
        assert!(validate_grant_permissions(&grants).is_ok());
    }

    #[test]
    fn validate_grants_rejects_invalid() {
        let grants = vec!["agent.read".into(), "invalid.permission".into()];
        assert!(validate_grant_permissions(&grants).is_err());
    }

    #[test]
    fn validate_grants_allows_domain_wildcards() {
        let grants = vec!["agent".into()];
        assert!(validate_grant_permissions(&grants).is_ok());
    }

    #[test]
    fn all_permissions_count() {
        let names = list_all_permission_names();
        assert!(names.len() > 20, "expected 30 leaf permissions");
    }

    #[test]
    fn all_domains_count() {
        let domains = list_all_domain_names();
        assert_eq!(domains.len(), 14);
    }
}
