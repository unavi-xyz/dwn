use xdid_core::did::Did;

pub fn parse_url(did: &Did) -> String {
    // web-did = "did:web:" domain-name
    // web-did = "did:web:" domain-name * (":" path)
    let mut split = did.method_id.0.split(':');

    let domain = split.next().unwrap().replace("%3A", ":");

    // Don't use HTTPS for localhost to make testing easier.
    let mut url = if domain.starts_with("localhost:") {
        "http://".to_string()
    } else {
        "https://".to_string()
    };

    url.push_str(&domain);

    let mut has_path = false;

    for path in split {
        has_path = true;

        url.push('/');
        url.push_str(path);
    }

    if !has_path {
        url.push_str("/.well-known");
    }

    url.push_str("/did.json");

    url
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use xdid_core::did::Did;

    use super::*;

    #[test]
    fn test_parse_no_path() {
        let did = Did::from_str("did:web:w3c-ccg.github.io").unwrap();
        let url = parse_url(&did);
        assert_eq!(url, "https://w3c-ccg.github.io/.well-known/did.json");
    }

    #[test]
    fn test_parse_path() {
        let did = Did::from_str("did:web:w3c-ccg.github.io:user:alice").unwrap();
        let url = parse_url(&did);
        assert_eq!(url, "https://w3c-ccg.github.io/user/alice/did.json");
    }

    #[test]
    fn test_parse_port() {
        let did = Did::from_str("did:web:example.com%3A3000:user:alice").unwrap();
        let url = parse_url(&did);
        assert_eq!(url, "https://example.com:3000/user/alice/did.json");
    }

    #[test]
    fn test_parse_localhost_http() {
        let did = Did::from_str("did:web:localhost%3A3000").unwrap();
        let url = parse_url(&did);
        assert_eq!(url, "http://localhost:3000/.well-known/did.json");
    }
}
