use crate::{ApiError, Result};

/// Normalizes user input into an instance base URL.
///
/// Trims whitespace, defaults to `https://`, and strips trailing slashes and
/// an accidentally pasted `/api/v1` or `/docs` suffix.
pub fn normalize_base_url(input: &str) -> Result<url::Url> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(ApiError::InvalidUrl("enter the address of your YAAMS instance".into()));
    }
    let with_scheme = if trimmed.contains("://") {
        trimmed.to_string()
    } else {
        format!("https://{trimmed}")
    };
    let mut url = url::Url::parse(&with_scheme).map_err(|e| ApiError::InvalidUrl(e.to_string()))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(ApiError::InvalidUrl("only http and https are supported".into()));
    }
    if url.host_str().is_none_or(str::is_empty) {
        return Err(ApiError::InvalidUrl("the address has no host".into()));
    }
    let mut path = url.path().trim_end_matches('/').to_string();
    for suffix in ["/api/v1", "/api", "/docs"] {
        if let Some(stripped) = path.strip_suffix(suffix) {
            path = stripped.to_string();
            break;
        }
    }
    url.set_path(&format!("{path}/"));
    url.set_query(None);
    url.set_fragment(None);
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::normalize_base_url;

    fn norm(s: &str) -> String {
        normalize_base_url(s).unwrap().to_string()
    }

    #[test]
    fn adds_scheme_and_trailing_slash() {
        assert_eq!(norm("yaams.example"), "https://yaams.example/");
        assert_eq!(norm("  https://yaams.example  "), "https://yaams.example/");
    }

    #[test]
    fn strips_api_suffixes() {
        assert_eq!(norm("https://yaams.example/api/v1/"), "https://yaams.example/");
        assert_eq!(norm("https://yaams.example/docs"), "https://yaams.example/");
        assert_eq!(norm("http://host:8080/va/api/v1"), "http://host:8080/va/");
    }

    #[test]
    fn rejects_bad_input() {
        assert!(normalize_base_url("").is_err());
        assert!(normalize_base_url("ftp://x").is_err());
    }
}
