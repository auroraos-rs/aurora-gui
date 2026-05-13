//! Q-Variant serialization helpers for Aurora OS window properties.
//!
//! Aurora OS uses Qt's QVariant format for window properties communicated
//! via Wayland generic properties.

const Q_VARIANT_BOOL_TRUE: &[u8] = &[0, 0, 0, 1, 0, 1];
const Q_VARIANT_BOOL_FALSE: &[u8] = &[0, 0, 0, 1, 0, 0];
const Q_VARIANT_U64: &[u8] = &[0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const Q_VARIANT_STRING: &[u8] = &[0, 0, 0, 10, 0, 0, 0, 0, 0];

/// Serialize a `bool` as Q-Variant.
pub fn from_bool(value: bool) -> Vec<u8> {
    if value {
        Q_VARIANT_BOOL_TRUE.to_vec()
    } else {
        Q_VARIANT_BOOL_FALSE.to_vec()
    }
}

/// Serialize a `u64` as Q-Variant.
pub fn from_u64(value: u64) -> Vec<u8> {
    let mut res = Q_VARIANT_U64.to_vec();
    for i in 0..8 {
        res[12 - i] = ((value >> (8 * i)) & 0xFF) as u8;
    }
    res
}

/// Serialize a `&str` as Q-Variant (UTF-16LE).
pub fn from_str(value: &str) -> Vec<u8> {
    let length = value.encode_utf16().count() * 2;
    let mut res = Q_VARIANT_STRING.to_vec();
    res.reserve(length);
    for i in 0..4 {
        res[8 - i] = ((length >> (8 * i)) & 0xFF) as u8;
    }
    for ch in value.encode_utf16() {
        res.push(0);
        res.push(ch as u8);
    }
    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_false() {
        assert_eq!(from_bool(false), vec![0, 0, 0, 1, 0, 0]);
    }

    #[test]
    fn bool_true() {
        assert_eq!(from_bool(true), vec![0, 0, 0, 1, 0, 1]);
    }

    #[test]
    fn u64_test() {
        assert_eq!(from_u64(7), vec![0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 7]);
    }

    #[test]
    fn string_matches_category() {
        assert_eq!(
            from_str("cover"),
            vec![
                0, 0, 0, 10, 0, 0, 0, 0, 10, 0, 99, 0, 111, 0, 118, 0, 101, 0, 114
            ]
        );
    }

    #[test]
    fn string_matches_sailfish_cover_window() {
        assert_eq!(
            from_str("__winref:9"),
            vec![
                0, 0, 0, 10, 0, 0, 0, 0, 20, 0, 95, 0, 95, 0, 119, 0, 105, 0, 110, 0, 114, 0, 101,
                0, 102, 0, 58, 0, 57
            ]
        );
    }
}
