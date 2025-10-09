use std::collections::HashMap;

use base64::alphabet::Alphabet;

mod err;
mod result;
use crate::err::{BaseNError, DecodeError};
use crate::result::BaseNResult;

pub fn decode(s: &[u8], alphabet: Alphabet, validate: bool) -> BaseNResult<Vec<u8>> {
    if s.is_empty() {
        return Ok(Vec::new());
    }
    let alphabet = HashMap::<u8, u8>::from_iter(
        alphabet
            .as_str()
            .as_bytes()
            .iter()
            .enumerate()
            .map(|(i, &b)| (b, i as u8)),
    );
    let paddings_count = s.iter().rev().take_while(|&&b| b == b'=').count();
    if validate {
        if s.len() % 4 != 0 {
            return Err(BaseNError::Decode(DecodeError::InvalidInputLength(s.len())));
        }

        if paddings_count > 2 {
            return Err(BaseNError::Decode(DecodeError::InvalidInputLength(s.len())));
        }
    }
    let expected_len = (s.len() / 4) * 3 - paddings_count;
    let mut decoded = Vec::<u8>::with_capacity(expected_len);

    let mut collected_bits: u8 = 0;
    let mut buf: u32 = 0;

    for (i, &c) in s.iter().enumerate() {
        let pos = match alphabet.get(&c) {
            Some(pos) => *pos as u32,
            None => {
                if c == b'=' {
                    break;
                }
                if validate {
                    return Err(BaseNError::Decode(DecodeError::InvalidSymbol(i, c)));
                }
                continue;
            }
        };

        buf = (buf << 6) | pos;
        collected_bits += 6;
        if collected_bits >= 8 {
            collected_bits -= 8;
            let byte = (buf >> collected_bits) as u8;
            decoded.push(byte);
            buf &= (1 << collected_bits) - 1;
        }
    }

    Ok(decoded)
}
