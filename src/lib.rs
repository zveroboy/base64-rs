#![allow(dead_code)]
const CHARS_LEN: usize = 0o100;
type Base64SliceType = [u8; CHARS_LEN];
const BASE64_CHARS: Base64SliceType = get_base64_chars();
const BASE64_URL_CHARS: Base64SliceType = get_base64url_chars();

const fn get_base_chars() -> Base64SliceType {
    let mut result = [0u8; CHARS_LEN];
    let mut i = 0u8;
    const UPPERS: (u8, u8) = (b'A', b'Z');
    let uppers_len = UPPERS.1 - UPPERS.0 + 1;
    while i < uppers_len {
        result[i as usize] = i + UPPERS.0;
        i += 1;
    }

    let mut i = 0u8;
    const LOWERS: (u8, u8) = (b'a', b'z');
    let lowers_len = LOWERS.1 - LOWERS.0 + 1;
    while i < lowers_len {
        result[(uppers_len + i) as usize] = i + LOWERS.0;
        i += 1;
    }

    let mut i = 0u8;
    const DIGITS: (u8, u8) = (b'0', b'9');
    let digits_length = 10;
    while i < digits_length {
        result[(uppers_len + lowers_len + i) as usize] = i + DIGITS.0;
        i += 1;
    }

    result
}

const fn get_base64_chars() -> Base64SliceType {
    let mut result = get_base_chars();

    result[result.len() - 2] = b'+';
    result[result.len() - 1] = b'/';

    result
}

const fn get_base64url_chars() -> Base64SliceType {
    let mut result = get_base_chars();

    result[result.len() - 2] = b'-';
    result[result.len() - 1] = b'_';

    result
}

fn base_encode(base_chars: &Base64SliceType, input: &[u8], padding: bool) -> String {
    input
        .chunks(3)
        .flat_map(|chunk| {
            let byte_1 = chunk.get(0).unwrap();
            let byte_2 = chunk.get(1);
            let byte_3 = chunk.get(2);

            // 0b_xxxxxxxx -> 0b_00xxxxxx
            //    ^^^^^^           ^^^^^^
            let a = byte_1 >> 2;
            let a = BASE64_CHARS[a as usize];

            // 0b_xxxxxxxx -> 0b_00xx0000 + 0b_yyyyyyyy -> 0b_0000yyyy = 0b_00xxyyyy
            //          ^^         ^^          ^^^^               ^^^^
            let b = ((byte_1 & ((1 << 2) - 1)) << 4) + (byte_2.cloned().unwrap_or_default() >> 4);
            let b = BASE64_CHARS[b as usize];

            let mut result = [a, b, b'=', b'='];

            // 0b_yyyyyyyy -> 0b_00yyyy00 + 0b_zzzzzzzz -> 0b_000000zz = 0b_00yyyyzz
            //        ^^^^         ^^^^        ^^                   ^^
            let c = byte_2.map(|byte| {
                ((byte & ((1 << 4) - 1)) << 2) + (byte_3.cloned().unwrap_or_default() >> 6)
            });
            let c = c.map(|i| base_chars[i as usize]);

            if let Some(c) = c {
                result[2] = c;
            }

            // 0b_zzzzzzzz -> 0b_00zzzzzz
            //      ^^^^^^         ^^^^^^
            let d = byte_3.map(|b| b & ((1 << 6) - 1));
            let d = d.map(|i| base_chars[i as usize]);

            if let Some(d) = d {
                result[3] = d;
            }

            result
        })
        .filter(|b| padding || *b != b'=')
        .map(char::from)
        .collect::<String>()
}

pub fn base64_encode(input: &[u8], padding: bool) -> String {
    base_encode(&BASE64_CHARS, input, padding)
}

pub fn base64url_encode(input: &[u8], padding: bool) -> String {
    base_encode(&BASE64_URL_CHARS, input, padding)
}

fn get_position(b: u8) -> Option<u8> {
    const UPPERS_LEN: u8 = b'Z' - b'A' + 1;
    const LOWERS_LEN: u8 = b'z' - b'a' + 1;

    let ch = match b {
        b'A'..=b'Z' => b - b'A',
        b'a'..=b'z' => b - b'a' + UPPERS_LEN,
        b'0'..=b'9' => b - b'0' + UPPERS_LEN + LOWERS_LEN,
        b'-' | b'+' => (CHARS_LEN - 2) as u8,
        b'_' | b'/' => (CHARS_LEN - 1) as u8,
        _ => {
            return None;
        }
    };

    Some(ch)
}

pub fn base64_decode<T: AsRef<str>>(input: T) -> String {
    input
        .as_ref()
        .as_bytes()
        .chunks(4)
        .flat_map(|chunk| {
            let a = chunk.get(0).copied().and_then(get_position).unwrap();
            let b = chunk
                .get(1)
                .copied()
                .and_then(get_position)
                .expect("Broken encoding");
            let c = chunk.get(2).copied().and_then(get_position);
            let d = chunk.get(3).copied().and_then(get_position);

            // 0b_00aaaaaa -> 0b_aaaaaa00 + 0b_00bbbbbb -> 0b_000000bb = 0b_aaaaaabb
            //      ^^^^^^       ^^^^^^          ^^                 ^^
            let byte_1 = (a << 2) + (b >> 4);

            let mut result = [Some(byte_1), None, None];

            if let Some(c) = c {
                // 0b_00bbbbbb -> 0b_bbbb0000 + 0b_00cccccc -> 0b_0000cccc = 0b_bbbbcccc
                //        ^^^^       ^^^^            ^^^^             ^^^^
                let byte_2 = (b << 4) + (c >> 2);
                result[1] = Some(byte_2);
            }

            if let Some((c, d)) = c.zip(d) {
                // 0b_00cccccc -> 0b_cc000000 + 0b_00dddddd = 0b_ccdddddd
                //          ^^       ^^              ^^^^^^
                let byte_3 = (c << 6) + d;
                result[2] = Some(byte_3);
            }

            result
        })
        .filter_map(|opt| opt)
        .map(char::from)
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use crate::base64_decode;
    use crate::base64_encode;

    #[test]
    fn it_works() {
        let test_1 = "Man is distinguished";
        let test_1_encoded = base64_encode(test_1.as_bytes(), true);

        assert_eq!(test_1_encoded, "TWFuIGlzIGRpc3Rpbmd1aXNoZWQ=");

        let test_2_encoded = base64_encode(test_1.as_bytes(), false);

        assert_eq!(test_2_encoded, "TWFuIGlzIGRpc3Rpbmd1aXNoZWQ");

        assert_eq!(base64_decode(test_1_encoded), test_1);

        assert_eq!(base64_decode("bGlnaHQgd29yay4="), "light work.");

        assert_eq!(base64_decode("bGlnaHQgd29yay4"), "light work.");
    }
}
