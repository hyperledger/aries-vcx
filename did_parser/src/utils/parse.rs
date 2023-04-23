use crate::error::ParseError;
use crate::DIDRange;

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
        .ok_or(ParseError::InvalidInput(did_url.to_string()))?;

    // Empty key or value is an error
    if value_start == key_start || value_start == end {
        return Err(ParseError::InvalidInput(did_url.to_string()));
    }

    // Value ends at end of string or next separator
    let next_pos = did_url[value_start..end]
        .find(|c: char| c == ';' || c == '?' || c == '#' || c == '/' || c == '&')
        .map_or(end, |i| value_start + i);

    Ok((key_start, value_start, next_pos))
}

// TODO: Support tunnel methods
pub fn parse_did_method_id(did_url: &str) -> Result<(DIDRange, DIDRange, DIDRange), ParseError> {
    // DID = "did:" method ":" method-specific-id
    let method_start = did_url
        .find(':')
        .ok_or(ParseError::InvalidInput(did_url.to_string()))?;
    if &did_url[..method_start] != "did" {
        return Err(ParseError::InvalidInput(did_url.to_string()));
    }
    let method_end = did_url[method_start + 1..]
        .find(':')
        .map(|i| i + method_start + 1)
        .ok_or(ParseError::InvalidInput(did_url.to_string()))?;

    // TODO
    // assumed: method-specific-id = 1*idchar
    // actual : method-specific-id = *( *idchar ":" ) 1*idchar
    let id_start = method_end + 1;
    let id_end = did_url[id_start..]
        .find(|c: char| c == ';' || c == '/' || c == '?' || c == '#' || c == '&')
        .map_or(did_url.len(), |i| i + id_start);

    let did = 0..id_end;
    let method = method_start + 1..method_end;
    let id = id_start..id_end;

    // No method-specific-id is an error
    if id.is_empty() {
        return Err(ParseError::InvalidInput(did_url.to_string()));
    }

    Ok((did, method, id))
}

pub(crate) fn parse_path(did_url: &str, current_pos: usize) -> Result<DIDRange, ParseError> {
    if !did_url[current_pos..].starts_with('/') {
        return Err(ParseError::InvalidInput(did_url.to_string()));
    }
    // Path ends with query, fragment, param or end of string
    let path_end = did_url[current_pos..]
        .find(|c: char| c == '?' || c == '#' || c == ';')
        .map_or(did_url.len(), |i| i + current_pos);

    if path_end - current_pos <= 1 {
        return Err(ParseError::InvalidInput(did_url.to_string()));
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
        assert_eq!(4..11, result.1);
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
