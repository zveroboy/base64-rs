#![allow(dead_code)]
use std::collections::HashMap;

type Base64SliceType = [u8; 64];
const BASE64_CHARS: Base64SliceType = get_base64_chars();
const BASE64_URL_CHARS: Base64SliceType = get_base64url_chars();

const fn get_base_chars() -> Base64SliceType {
    let mut result = [0u8; 64];
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

            // 0bxxxxxxxx -> 0b00xxxxxx
            //   ^^^^^^          ^^^^^^
            let a = byte_1 >> 2;
            let a = BASE64_CHARS[a as usize];

            // 0bxxxxxxxx -> 0b00xx0000 + 0byyyyyyyy -> 0b0000yyyy = 0b00xxyyyy
            //         ^^        ^^         ^^^^              ^^^^
            let b = ((byte_1 & ((1 << 2) - 1)) << 4) + (byte_2.cloned().unwrap_or_default() >> 4);
            let b = BASE64_CHARS[b as usize];

            // 0byyyyyyyy -> 0b00yyyy00 + 0bzzzzzzzz -> 0b000000zz = 0b00yyyyzz
            //       ^^^^        ^^^^       ^^                  ^^
            let c = byte_2.map(|byte| {
                ((byte & ((1 << 4) - 1)) << 2) + (byte_3.cloned().unwrap_or_default() >> 6)
            });

            let mut result = vec![a, b];

            let pad_fallback = Some(b'=').filter(|_| padding);

            let c = c.map(|i| base_chars[i as usize]).or(pad_fallback);

            if let Some(c) = c {
                result.push(c);
            }

            // 0bzzzzzzzz -> 0b00zzzzzz
            //     ^^^^^^        ^^^^^^
            let d = byte_3.map(|b| b & ((1 << 6) - 1));

            let d = d.map(|i| base_chars[i as usize]).or(pad_fallback);

            if let Some(d) = d {
                result.push(d);
            }

            result
        })
        .map(char::from)
        .collect::<String>()
}

fn base64_encode(input: &[u8], padding: bool) -> String {
    base_encode(&BASE64_CHARS, input, padding)
}

fn base64url_encode(input: &[u8], padding: bool) -> String {
    base_encode(&BASE64_URL_CHARS, input, padding)
}

fn base_decode<T: AsRef<str>>(base_chars: &Base64SliceType, input: T) -> String {
    // TODO: extract and make const
    let hash_map = base_chars
        .iter()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect::<HashMap<_, _>>();

    let codes = input
        .as_ref()
        .as_bytes()
        .iter()
        .filter_map(|c| hash_map.get(&c))
        .map(|b| *b as u8)
        .collect::<Vec<_>>();

    codes
        .chunks(4)
        .flat_map(|chunk| {
            let a = chunk.get(0).unwrap();
            let b = chunk.get(1).expect("Broken encoding");
            let c = chunk.get(2);
            let d = chunk.get(3);

            // 0b_00aaaaaa -> 0b_aaaaaa00 + 0b_00bbbbbb -> 0b_000000bb = 0b_aaaaaabb
            //      ^^^^^^       ^^^^^^          ^^                 ^^
            let byte_1 = (a << 2) + (b >> 4);

            let mut result = vec![byte_1];

            if let Some(c) = c {
                // 0b_00bbbbbb -> 0b_bbbb0000 + 0b_00cccccc -> 0b_0000cccc = 0b_bbbbcccc
                //        ^^^^       ^^^^            ^^^^             ^^^^
                let byte_2 = (b << 4) + (c >> 2);
                result.push(byte_2);
            }

            if let Some((c, d)) = c.zip(d) {
                // 0b_00cccccc -> 0b_cc000000 + 0b_00dddddd = 0b_ccdddddd
                //          ^^       ^^              ^^^^^^
                let byte_3 = (c << 6) + d;
                result.push(byte_3);
            }

            result
        })
        .map(char::from)
        .collect::<String>()
}

fn base64_decode<T: AsRef<str>>(input: T) -> String {
    base_decode(&BASE64_CHARS, input)
}

fn base64url_decode<T: AsRef<str>>(input: T) -> String {
    base_decode(&BASE64_URL_CHARS, input)
}

#[cfg(test)]
mod tests {
    use super::*;

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
