//! ZIP 302 TVLV (Type-Version-Length-Value) encoder/decoder.
//!
//! Implements the structured memo container from str4d's ZIP 302 draft (PR #638).
//! Self-contained, zero external dependencies.

const MARKER: u8 = 0xF7;

/// A single part in a TVLV-encoded memo.
#[derive(Debug, Clone, PartialEq)]
pub struct TvlvPart {
    pub part_type: u16,
    pub version: u8,
    pub value: Vec<u8>,
}

/// TVLV decode errors.
#[derive(Debug, Clone, PartialEq)]
pub enum TvlvError {
    MissingMarker,
    Truncated,
    InvalidCompactSize,
    DuplicatePartType(u16),
    NonZeroPadding,
}

impl core::fmt::Display for TvlvError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            TvlvError::MissingMarker => write!(f, "missing 0xF7 marker"),
            TvlvError::Truncated => write!(f, "unexpected end of data"),
            TvlvError::InvalidCompactSize => write!(f, "invalid compactSize encoding"),
            TvlvError::DuplicatePartType(t) => write!(f, "duplicate part type {}", t),
            TvlvError::NonZeroPadding => write!(f, "non-zero padding after end marker"),
        }
    }
}

/// Encode parts into a TVLV memo with 0xF7 prefix.
///
/// Each part is (type, version, value). An end marker (type 0) and zero padding
/// are appended automatically.
pub fn encode(parts: &[(u16, u8, &[u8])]) -> Vec<u8> {
    let mut out = vec![MARKER];
    for &(ptype, version, value) in parts {
        out.extend(encode_compact_size(ptype as u64));
        out.extend(encode_compact_size(version as u64));
        out.extend(encode_compact_size(value.len() as u64));
        out.extend_from_slice(value);
    }
    // end marker
    out.push(0x00);
    out
}

/// Decode TVLV memo bytes (including the 0xF7 prefix) into parts.
pub fn decode(data: &[u8]) -> Result<Vec<TvlvPart>, TvlvError> {
    if data.is_empty() || data[0] != MARKER {
        return Err(TvlvError::MissingMarker);
    }

    let mut pos = 1;
    let mut parts = Vec::new();
    let mut seen = [false; 65536];

    loop {
        if pos >= data.len() {
            return Err(TvlvError::Truncated);
        }

        let (ptype_raw, sz) = decode_compact_size(&data[pos..])?;
        pos += sz;

        if ptype_raw == 0 {
            // end marker - rest must be zero padding
            for &b in &data[pos..] {
                if b != 0 {
                    return Err(TvlvError::NonZeroPadding);
                }
            }
            break;
        }

        let ptype = ptype_raw as u16;
        if seen[ptype as usize] {
            return Err(TvlvError::DuplicatePartType(ptype));
        }
        seen[ptype as usize] = true;

        if pos >= data.len() {
            return Err(TvlvError::Truncated);
        }
        let (version_raw, sz) = decode_compact_size(&data[pos..])?;
        pos += sz;

        if pos >= data.len() {
            return Err(TvlvError::Truncated);
        }
        let (length, sz) = decode_compact_size(&data[pos..])?;
        pos += sz;

        let length = length as usize;
        if pos + length > data.len() {
            return Err(TvlvError::Truncated);
        }

        parts.push(TvlvPart {
            part_type: ptype,
            version: version_raw as u8,
            value: data[pos..pos + length].to_vec(),
        });
        pos += length;
    }

    Ok(parts)
}

fn encode_compact_size(n: u64) -> Vec<u8> {
    if n <= 252 {
        vec![n as u8]
    } else if n <= 0xFFFF {
        let mut buf = vec![0xFD];
        buf.extend_from_slice(&(n as u16).to_le_bytes());
        buf
    } else if n <= 0xFFFF_FFFF {
        let mut buf = vec![0xFE];
        buf.extend_from_slice(&(n as u32).to_le_bytes());
        buf
    } else {
        let mut buf = vec![0xFF];
        buf.extend_from_slice(&n.to_le_bytes());
        buf
    }
}

fn decode_compact_size(data: &[u8]) -> Result<(u64, usize), TvlvError> {
    if data.is_empty() {
        return Err(TvlvError::Truncated);
    }

    match data[0] {
        0..=252 => Ok((data[0] as u64, 1)),
        0xFD => {
            if data.len() < 3 {
                return Err(TvlvError::Truncated);
            }
            let val = u16::from_le_bytes([data[1], data[2]]);
            if val < 253 {
                return Err(TvlvError::InvalidCompactSize);
            }
            Ok((val as u64, 3))
        }
        0xFE => {
            if data.len() < 5 {
                return Err(TvlvError::Truncated);
            }
            let val = u32::from_le_bytes([data[1], data[2], data[3], data[4]]);
            if val < 0x10000 {
                return Err(TvlvError::InvalidCompactSize);
            }
            Ok((val as u64, 5))
        }
        0xFF => {
            if data.len() < 9 {
                return Err(TvlvError::Truncated);
            }
            let val = u64::from_le_bytes([
                data[1], data[2], data[3], data[4], data[5], data[6], data[7], data[8],
            ]);
            if val < 0x1_0000_0000 {
                return Err(TvlvError::InvalidCompactSize);
            }
            Ok((val, 9))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_single_text_part() {
        let parts = [(160u16, 0u8, b"hello zcash".as_slice())];
        let encoded = encode(&parts);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].part_type, 160);
        assert_eq!(decoded[0].version, 0);
        assert_eq!(decoded[0].value, b"hello zcash");
    }

    #[test]
    fn roundtrip_multiple_parts() {
        let parts = [
            (160u16, 0u8, b"text part".as_slice()),
            (255u16, 0u8, b"\x01\x02\x03".as_slice()),
        ];
        let encoded = encode(&parts);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].part_type, 160);
        assert_eq!(decoded[1].part_type, 255);
        assert_eq!(decoded[1].value, b"\x01\x02\x03");
    }

    #[test]
    fn roundtrip_empty_memo() {
        let encoded = encode(&[]);
        let decoded = decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn roundtrip_experimental_range() {
        let parts = [(65530u16, 1u8, b"zap1 attestation data".as_slice())];
        let encoded = encode(&parts);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded[0].part_type, 65530);
        assert_eq!(decoded[0].version, 1);
    }

    #[test]
    fn reject_missing_marker() {
        assert_eq!(decode(&[0x00, 0x00]).unwrap_err(), TvlvError::MissingMarker);
    }

    #[test]
    fn reject_truncated() {
        assert_eq!(decode(&[0xF7]).unwrap_err(), TvlvError::Truncated);
    }

    #[test]
    fn reject_duplicate_part_type() {
        let mut data = vec![MARKER];
        // first part: type=1, version=0, length=1, value=0x42
        data.push(1);
        data.push(0);
        data.push(1);
        data.push(0x42);
        // duplicate: type=1 again
        data.push(1);
        data.push(0);
        data.push(1);
        data.push(0x43);
        data.push(0); // end marker
        assert_eq!(decode(&data).unwrap_err(), TvlvError::DuplicatePartType(1));
    }

    #[test]
    fn reject_nonzero_padding() {
        let mut data = vec![MARKER];
        data.push(0); // end marker
        data.push(0xFF); // non-zero padding
        assert_eq!(decode(&data).unwrap_err(), TvlvError::NonZeroPadding);
    }

    #[test]
    fn compact_size_roundtrip() {
        for n in [0, 1, 252, 253, 1000, 65535, 65536, 0xFFFF_FFFF, 0x1_0000_0000] {
            let encoded = encode_compact_size(n);
            let (decoded, _) = decode_compact_size(&encoded).unwrap();
            assert_eq!(decoded, n, "compact_size failed for {n}");
        }
    }
}
