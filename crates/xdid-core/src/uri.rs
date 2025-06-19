#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Segment {
    /// segment
    Base,
    /// segment-nz
    Nz,
    // segment-nz-nc
    NzNc,
}

/// Whether the string conforms to a given [Segment], following [RFC 3986](https://www.rfc-editor.org/rfc/rfc3986#section-3.3).
pub fn is_segment(value: &str, segment: Segment) -> bool {
    if (segment == Segment::Nz || segment == Segment::NzNc) && value.is_empty() {
        return false;
    }

    let mut processing_pct_encoded = false;
    let mut pct_encoded_char = 0usize;

    for c in value.chars() {
        // pct-encoded = "%" HEXDIG HEXDIG
        if processing_pct_encoded {
            if pct_encoded_char == 2 {
                pct_encoded_char = 0;
                processing_pct_encoded = false;
            } else {
                pct_encoded_char += 1;

                if c.is_ascii_hexdigit() {
                    continue;
                } else {
                    return false;
                }
            }
        }

        if c == '%' {
            processing_pct_encoded = true;
            continue;
        }

        if is_unreserved(c) {
            continue;
        }

        if is_sub_delim(c) {
            continue;
        }

        // pchar = unreserved / pct-encoded / sub-delims / ":" / "@"
        // segment       = *pchar
        // segment-nz    = 1*pchar
        // segment-nz-nc = 1*( unreserved / pct-encoded / sub-delims / "@" )
        //            ; non-zero-length segment without any colon ":"
        if c == '@' {
            continue;
        }

        match segment {
            Segment::Base | Segment::Nz => {
                if c == ':' {
                    continue;
                }
            }
            Segment::NzNc => {}
        }

        return false;
    }

    true
}

/// unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"
fn is_unreserved(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '.' || c == '_' || c == '~'
}

/// sub-delims = "!" / "$" / "&" / "'" / "(" / ")" / "*" / "+" / "," / ";" / "="
fn is_sub_delim(c: char) -> bool {
    c == '!'
        || c == '$'
        || c == '&'
        || c == '\''
        || c == '('
        || c == ')'
        || c == '*'
        || c == '+'
        || c == ','
        || c == ';'
        || c == '='
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_length() {
        assert!(is_segment("", Segment::Base));
        assert!(!is_segment("", Segment::Nz));
        assert!(!is_segment("", Segment::NzNc));
    }

    #[test]
    fn test_segment_alphanumeric() {
        assert!(is_segment(
            "abcdefghijklmnopqrstuvwxyz0123456789",
            Segment::Base
        ));
    }

    #[test]
    fn test_segment_symbols() {
        assert!(is_segment("!$&'()*+,;=@", Segment::Base));
        assert!(is_segment("!$&'()*+,;=@", Segment::Nz));
        assert!(is_segment("!$&'()*+,;=@", Segment::NzNc));
    }

    #[test]
    fn test_segment_colon() {
        assert!(is_segment(":", Segment::Base));
        assert!(is_segment(":", Segment::Nz));
        assert!(!is_segment(":", Segment::NzNc));
    }

    #[test]
    fn test_segment_pct_encode() {
        assert!(is_segment("%30%f9a", Segment::Base));
        assert!(!is_segment("%3%f9a", Segment::Base));
        assert!(!is_segment("%%f9a", Segment::Base));
    }
}
