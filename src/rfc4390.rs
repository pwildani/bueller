/// RFC 3490: Handle unicode-encoded names.
///
/// Given a dotted name in a string, `encode_dotted_name` transforms it to the bytes to pass to the
/// query constructor.
///
/// `vec_ref` handles the conversion to slice references that the query constructor wants.  The
/// query constructor does not take the more natural `Vec<Vec<u8>>` to represent the name because
/// passing each segment as a slice allows it to copy the bytes directly out of their source
/// location if they are already encoded and do not need to go through this library.
///
/// # Examples
///
/// ```
/// # use bueller::rfc4390::*;
///   let name = "github.com";
///   let encoded_name = encode_dotted_name(&name).unwrap();
///   let query_name = vec_ref(&encoded_name);
/// ```
///
use std::vec::Vec;

fn encode_segment(segment: &str) -> Option<Vec<u8>> {
    use std::ascii::AsciiExt;
    extern crate url;
    use url::idna::punycode;

    if segment.is_ascii() {
        return Some(segment.into());
    } else {
        if let Some(encoded) = punycode::encode(&segment.chars().collect::<Vec<char>>()[..]) {
            let mut result = Vec::with_capacity(encoded.len() + 4);
            // RFC 3490 Section 5: ACE prefix.
            result.extend("xn--".bytes());
            result.extend(encoded.bytes());
            return Some(result);
        }
        return None;
    }
}

/// Encodes Unicode "foo.bar" into the ASCII bytes that DNS handles over the wire.
///
/// Not zero-copy.
///
/// # Failures
/// Returns None if name cannot be punycoded. (e.g. if any segment encodes to more than 63 bytes)
pub fn encode_dotted_name(name: &str) -> Option<Vec<Vec<u8>>> {
    let mut result = Vec::with_capacity(7);
    for s in name.split('.').map(encode_segment) {
        if let Some(segment) = s {
            result.push(segment);
        } else {
            return None;
        }
    }
    return Some(result);
}

pub fn vec_ref<'a>(segments: &'a Vec<Vec<u8>>) -> Vec<&'a [u8]> {
    let mut nref = Vec::with_capacity(segments.len());
    for i in 0..segments.len() {
        nref.push(&segments[i][..]);
    }
    return nref;
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn encode() {
        let name = "123.a-b.ű.déf.";
        let segments = encode_dotted_name(name).unwrap();
        assert_eq!(vec!["123".bytes().collect::<Vec<u8>>(),
                        "a-b".bytes().collect::<Vec<u8>>(),
                        "xn--5ga".bytes().collect::<Vec<u8>>(),
                        "xn--df-bja".bytes().collect::<Vec<u8>>(),
                        "".bytes().collect::<Vec<u8>>()],
                   segments);
    }
}
