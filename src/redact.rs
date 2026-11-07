use std::collections::BTreeMap;

pub(crate) fn sanitize(value: &str) -> String {
    value.chars().filter(|c| !c.is_control()).collect()
}
// Go strings.ToLower uses simple scalar mappings. Full-string lowercasing expands
// İ and applies contextual final-sigma rules, which can miss credential names.
// Use official ICU simple mappings for both built-in and custom flag matching.
fn sensitive_name(name: &str) -> String {
    let mapper = icu_casemap::CaseMapper::new();
    name.trim_start_matches('-')
        .chars()
        .map(|c| mapper.simple_lowercase(c))
        .collect()
}
fn sensitive_flag(name: &str, extra: &[&str]) -> bool {
    let normalized = sensitive_name(name);
    if !normalized.is_empty() && extra.iter().any(|flag| normalized == sensitive_name(flag)) {
        return true;
    }
    name.starts_with("--")
        && [
            "password",
            "passphrase",
            "passwd",
            "secret",
            "token",
            "private-key",
            "wallet",
        ]
        .iter()
        .any(|marker| normalized.contains(marker))
}

/// Sanitize a copy and redact known credential flags and URL credentials.
/// Custom flags extend defaults. Sensitive flags consume the next value even if
/// it resembles a flag. Short clusters and unmarked positional secrets are not parsed.
pub fn redact_args(args: &[&str], sensitive_flags: &[&str]) -> Vec<String> {
    let mut redact_next = false;
    args.iter()
        .map(|original| {
            let arg = sanitize(original);
            if redact_next {
                redact_next = false;
                return "[REDACTED]".into();
            }
            if arg.starts_with('-') {
                let (name, value) = arg
                    .split_once('=')
                    .map_or((arg.as_str(), None), |(n, v)| (n, Some(v)));
                if sensitive_flag(name, sensitive_flags) {
                    if value.is_some() {
                        return format!("{name}=[REDACTED]");
                    }
                    redact_next = true;
                    return name.into();
                }
                if let Some(value) = value {
                    return format!("{name}={}", redact_url(value, sensitive_flags));
                }
            }
            redact_url(&arg, sensitive_flags)
        })
        .collect()
}

/// Quoted display text, never a shell command. Does not mutate caller arguments.
pub fn command_line(command: &str, args: &[&str], sensitive_flags: &[&str]) -> String {
    std::iter::once(quote(&sanitize(command)))
        .chain(redact_args(args, sensitive_flags).iter().map(|s| quote(s)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote(value: &str) -> String {
    if !value.is_empty() && !value.chars().any(char::is_whitespace) && !value.contains('"') {
        return value.into();
    }
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            // Match Go display quoting for nonprintable Unicode characters;
            // official category data owns the Unicode-version boundary.
            c if !is_print(c) => {
                if u32::from(c) <= 0xffff {
                    out.push_str(&format!("\\u{:04x}", u32::from(c)));
                } else {
                    out.push_str(&format!("\\U{:08x}", u32::from(c)));
                }
            }
            _ => out.push(c),
        }
    }
    out.push('"');
    out
}

fn redact_url(value: &str, extra: &[&str]) -> String {
    if !value.contains("://") {
        return value.into();
    }
    redact_url_checked(value, extra).unwrap_or_else(|| "[REDACTED-URL]".into())
}
fn redact_url_checked(value: &str, extra: &[&str]) -> Option<String> {
    // Validate with the official maintained parser, but retain raw path, host,
    // scheme case and escapes: WHATWG normalization would change display values.
    url::Url::parse(value).ok()?;
    valid_percent_escapes(value)?;
    let (scheme, remainder) = value.split_once("://")?;
    let (body, fragment) = remainder
        .split_once('#')
        .map_or((remainder, None), |(b, f)| (b, Some(f)));
    let (body, query) = body
        .split_once('?')
        .map_or((body, None), |(b, q)| (b, Some(q)));
    let authority_end = body.find('/').unwrap_or(body.len());
    let (authority, path) = body.split_at(authority_end);
    let authority = match authority.rsplit_once('@') {
        Some((_, host)) => format!("%5BREDACTED%5D@{host}"),
        None => authority.into(),
    };
    let scheme = scheme.to_ascii_lowercase();
    let path = escape_component(path, false)?;
    let mut out = format!("{scheme}://{authority}{path}");
    if let Some(query) = query {
        let mut pairs: BTreeMap<Vec<u8>, Vec<Vec<u8>>> = BTreeMap::new();
        for pair in query.split('&').filter(|pair| !pair.is_empty()) {
            // Go ParseQuery rejects unescaped semicolons instead of guessing
            // whether the following text is a second credential parameter.
            if pair.contains(';') {
                return None;
            }
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            pairs
                .entry(query_decode(key)?)
                .or_default()
                .push(query_decode(value)?);
        }
        let mut encoded = Vec::new();
        for (key, values) in pairs {
            let sensitive = sensitive_flag(&format!("--{}", String::from_utf8_lossy(&key)), extra);
            let values = if sensitive {
                vec![b"[REDACTED]".to_vec()]
            } else {
                values
            };
            for value in values {
                encoded.push(format!("{}={}", query_encode(&key), query_encode(&value)));
            }
        }
        if !encoded.is_empty() || query.is_empty() {
            out.push('?');
            out.push_str(&encoded.join("&"));
        }
    }
    if let Some(fragment) = fragment.filter(|f| !f.is_empty()) {
        out.push('#');
        out.push_str(&escape_component(fragment, true)?);
    }
    Some(out)
}
fn valid_percent_escapes(value: &str) -> Option<()> {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            if !bytes.get(i + 1)?.is_ascii_hexdigit() || !bytes.get(i + 2)?.is_ascii_hexdigit() {
                return None;
            }
            i += 3;
        } else {
            i += 1;
        }
    }
    Some(())
}
fn query_decode(value: &str) -> Option<Vec<u8>> {
    let mut bytes = value.bytes();
    let mut out = Vec::new();
    while let Some(byte) = bytes.next() {
        out.push(match byte {
            b'+' => b' ',
            b'%' => {
                (char::from(bytes.next()?).to_digit(16)? * 16
                    + char::from(bytes.next()?).to_digit(16)?) as u8
            }
            _ => byte,
        });
    }
    Some(out)
}
fn query_encode(value: &[u8]) -> String {
    let mut out = String::new();
    for byte in value {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(char::from(*byte))
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn escape_component(raw: &str, fragment: bool) -> Option<String> {
    // Preserve an already valid escaped representation. This avoids altering
    // encoded slashes and the caller's escape case when displaying a URL.
    let plain = |b: u8| {
        b.is_ascii_alphanumeric()
            || b"-_.~$&+,/:;=@".contains(&b)
            || (fragment && b"?!()*".contains(&b))
    };
    if raw.bytes().all(|b| plain(b) || b"%!\x27()*[]".contains(&b)) {
        return Some(raw.into());
    }
    let mut bytes = raw.bytes();
    let mut out = String::new();
    while let Some(byte) = bytes.next() {
        let byte = if byte == b'%' {
            (char::from(bytes.next()?).to_digit(16)? * 16
                + char::from(bytes.next()?).to_digit(16)?) as u8
        } else {
            byte
        };
        if plain(byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    Some(out)
}

fn is_print(c: char) -> bool {
    use unicode_general_category::{GeneralCategory::*, get_general_category};
    c == ' '
        || !matches!(
            get_general_category(c),
            Control
                | Format
                | Surrogate
                | PrivateUse
                | Unassigned
                | LineSeparator
                | ParagraphSeparator
                | SpaceSeparator
        )
}
