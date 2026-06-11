use semver::Version;

fn normalize(value: &str) -> &str {
    value.trim().trim_start_matches('v')
}

pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (
        Version::parse(normalize(candidate)),
        Version::parse(normalize(current)),
    ) {
        (Ok(c), Ok(cur)) => c > cur,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_patch() {
        assert!(is_newer("1.0.1", "1.0.0"));
    }

    #[test]
    fn not_newer_when_equal() {
        assert!(!is_newer("1.0.0", "1.0.0"));
    }

    #[test]
    fn not_newer_when_older() {
        assert!(!is_newer("0.9.9", "1.0.0"));
    }

    #[test]
    fn handles_v_prefix() {
        assert!(is_newer("v1.2.0", "1.1.9"));
    }

    #[test]
    fn prerelease_lower_than_release() {
        assert!(!is_newer("1.0.0-rc.1", "1.0.0"));
        assert!(is_newer("1.0.0", "1.0.0-rc.1"));
    }

    #[test]
    fn invalid_is_not_newer() {
        assert!(!is_newer("not-semver", "1.0.0"));
    }
}
