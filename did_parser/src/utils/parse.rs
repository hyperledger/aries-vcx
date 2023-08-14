use crate::error::ParseError;
use crate::DidRange;

pub(crate) fn parse_key_value(
    did_url: &str,
    start: usize,
    end: usize,
) -> Result<(usize, usize, usize), ParseError> {
    // Skip separator
    let key_start = start + 1;

    // Value starts after equal sign
    // No equal sign is an error
    let value_start = did_url[key_start..end]
        .find('=')
        .map(|i| key_start + i + 1)
        .ok_or(ParseError::InvalidInput(
            "No value found when parsing key value pair",
        ))?;

    // Empty key or value is an error
    if value_start == key_start || value_start == end {
        return Err(ParseError::InvalidInput("Empty key or value"));
    }

    // Value ends at end of string or next separator
    let next_pos = did_url[value_start..end]
        .find(|c: char| c == ';' || c == '?' || c == '#' || c == '/' || c == '&')
        .map_or(end, |i| value_start + i);

    Ok((key_start, value_start, next_pos))
}

fn find_method_start_and_end(did_url: &str) -> Result<(usize, usize), ParseError> {
    // DID = "did:" method ":" method-specific-id
    let method_start = did_url
        .find(':')
        .ok_or(ParseError::InvalidInput("Failed to find method start"))?;
    if &did_url[..method_start] != "did" {
        return Err(ParseError::InvalidInput("Invalid scheme"));
    }
    let method_end = did_url[method_start + 1..]
        .find(':')
        .map(|i| i + method_start + 1)
        .ok_or(ParseError::InvalidInput("Failed to find method end"))?;

    Ok((method_start, method_end))
}

fn find_id_start_and_end(did_url: &str, method_end: usize) -> Result<(usize, usize), ParseError> {
    // method-specific-id = *( *idchar ":" ) 1*idchar
    let id_start = method_end + 1;
    let id_end = did_url[id_start..]
        .find(|c: char| c == ';' || c == '/' || c == '?' || c == '#' || c == '&')
        .map_or(did_url.len(), |i| i + id_start);

    Ok((id_start, id_end))
}

fn validate_did_url(did_url: &str, id_range: DidRange) -> Result<(), ParseError> {
    // No method-specific-id is an error
    if id_range.is_empty() {
        return Err(ParseError::InvalidInput("Empty method-specific-id"));
    }

    // idchar = ALPHA / DIGIT / "." / "-" / "_" / pct-encoded
    if !did_url[id_range]
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || ".-_%:".contains(c))
    {
        return Err(ParseError::InvalidInput(
            "Invalid characters in method-specific-id",
        ));
    }

    Ok(())
}

fn parse_qualified(did_url: &str) -> Result<(DidRange, Option<DidRange>, DidRange), ParseError> {
    let (method_start, method_end) = find_method_start_and_end(did_url)?;
    let (id_start, id_end) = find_id_start_and_end(did_url, method_end)?;

    let did_range = 0..id_end;
    let method_range = method_start + 1..method_end;
    let id_range = id_start..id_end;

    validate_did_url(did_url, id_range.clone())?;

    Ok((did_range, Some(method_range), id_range))
}

// TODO: Remove as soon as migration to qualified DIDs is complete
fn parse_unqualified(did_url: &str) -> Result<(DidRange, Option<DidRange>, DidRange), ParseError> {
    if did_url.contains(':') {
        return Err(ParseError::InvalidInput(
            "Unqualified did cannot contain ':'",
        ));
    }

    shared_vcx::validation::did::validate_did(&did_url)
        .map_err(|_| ParseError::InvalidInput("Unqualified DID failed validation"))?;

    let id_range = 0..did_url.len();

    validate_did_url(did_url, id_range.clone())?;

    Ok((id_range.clone(), None, id_range))
}

// TODO: Support tunnel methods
pub fn parse_did_method_id(
    did_url: &str,
) -> Result<(DidRange, Option<DidRange>, DidRange), ParseError> {
    if !did_url.starts_with("did:") {
        parse_unqualified(did_url)
    } else {
        parse_qualified(did_url)
    }
}

pub(crate) fn parse_path(did_url: &str, current_pos: usize) -> Result<DidRange, ParseError> {
    if !did_url[current_pos..].starts_with('/') {
        return Err(ParseError::InvalidInput("Path must start with '/'"));
    }
    // Path ends with query, fragment, param or end of string
    let path_end = did_url[current_pos..]
        .find(|c: char| c == '?' || c == '#' || c == ';')
        .map_or(did_url.len(), |i| i + current_pos);

    if path_end - current_pos <= 1 {
        return Err(ParseError::InvalidInput("Empty path"));
    }

    Ok(current_pos..path_end)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_key_value() {
        let did_url = "?key1=value1&key2=value2";
        let start = 0;
        let end = did_url.len();

        let result = parse_key_value(did_url, start, end).unwrap();
        assert_eq!((1, 6, 12), result);

        let start = 12;
        let result = parse_key_value(did_url, start, end).unwrap();
        assert_eq!((13, 18, did_url.len()), result);
    }

    #[test]
    fn test_parse_did_method_id() {
        let valid_did = "did:example:123456789abcdefghi";
        let result = parse_did_method_id(valid_did).unwrap();

        assert_eq!(0..valid_did.len(), result.0);
        assert_eq!(Some(4..11), result.1);
        assert_eq!(12..valid_did.len(), result.2);

        let invalid_did = "did-example:123456789abcdefghi";
        let result = parse_did_method_id(invalid_did);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_path() {
        let did_url =
            "did:example:123456789abcdefghi/path1/path2?param1=value1&param2=value2#fragment";
        let path_start = did_url.find('/').unwrap();

        let result = parse_path(did_url, path_start).unwrap();
        assert_eq!(path_start..(did_url.find('?').unwrap()), result);

        let no_path_did_url = "did:example:123456789abcdefghi?param1=value1&param2=value2#fragment";

        let result = parse_path(no_path_did_url, 0);
        assert!(result.is_err());
    }
}
