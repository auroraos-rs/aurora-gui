const Q_VARIANT_VOID: &[u8] = &[0, 0, 0, 0, 1];
const Q_VARIANT_U32: &[u8] = &[0, 0, 0, 3, 0, 0, 0, 0, 0];
const Q_VARIANT_U64: &[u8] = &[0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const Q_VARIANT_STRING: &[u8] = &[0, 0, 0, 10, 0, 0, 0, 0, 0];
const Q_VARIANT_BOOL_TRUE: &[u8] = &[0, 0, 0, 1, 0, 1];
const Q_VARIANT_BOOL_FALSE: &[u8] = &[0, 0, 0, 1, 0, 0];

pub fn from_void() -> Vec<u8> {
    Q_VARIANT_VOID.to_vec()
}

pub fn from_bool(value: bool) -> Vec<u8> {
    if value {
        Q_VARIANT_BOOL_TRUE.to_vec()
    } else {
        Q_VARIANT_BOOL_FALSE.to_vec()
    }
}

pub fn from_u32(value: u32) -> Vec<u8> {
    let mut res = Q_VARIANT_U32.to_vec();
    for i in 0..4 {
        res[8 - i] = ((value >> (8 * i)) & 0xFF) as u8;
    }
    res
}

pub fn from_u64(value: u64) -> Vec<u8> {
    let mut res = Q_VARIANT_U64.to_vec();
    for i in 0..8 {
        res[12 - i] = ((value >> (8 * i)) & 0xFF) as u8;
    }
    res
}

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

pub fn from_string(value: String) -> Vec<u8> {
    from_str(value.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qvariant_string() {
        assert_eq!(
            from_str("cover"),
            vec![
                0, 0, 0, 10, 0, 0, 0, 0, 10, 0, 99, 0, 111, 0, 118, 0, 101, 0, 114
            ]
        );
        assert_eq!(
            from_str("__winref:7"),
            vec![
                0, 0, 0, 10, 0, 0, 0, 0, 20, 0, 95, 0, 95, 0, 119, 0, 105, 0, 110, 0, 114, 0, 101,
                0, 102, 0, 58, 0, 55,
            ]
        )
    }

    #[test]
    fn test_qvariant_u64() {
        assert_eq!(from_u64(256), vec![0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 1, 0]);
        assert_eq!(from_u64(5), vec![0, 0, 0, 5, 0, 0, 0, 0, 0, 0, 0, 0, 5]);
    }
}
