//! Version-one THP value format. Objects require an explicit future contract.

use std::collections::HashSet;

use thp_hir::Type;
use thp_runtime::{RuntimeErrorKind, Value};

const MAGIC: &[u8] = b"THPS\x01";
const MAX_BYTES: usize = 16 * 1024 * 1024;
const MAX_DEPTH: usize = 64;
const MAX_NODES: usize = 1_000_000;

pub(super) fn serialize(value: &Value) -> Result<Vec<u8>, String> {
    let mut output = MAGIC.to_vec();
    let mut nodes = 0;
    encode(value, &mut output, 0, &mut nodes)?;
    Ok(output)
}

fn append(output: &mut Vec<u8>, bytes: &[u8]) -> Result<(), String> {
    if output
        .len()
        .checked_add(bytes.len())
        .is_none_or(|len| len > MAX_BYTES)
    {
        return Err("serialized value exceeds 16 MiB".to_owned());
    }
    output
        .try_reserve(bytes.len())
        .map_err(|_| "serialization allocation failed")?;
    output.extend_from_slice(bytes);
    Ok(())
}

fn encode(
    value: &Value,
    output: &mut Vec<u8>,
    depth: usize,
    nodes: &mut usize,
) -> Result<(), String> {
    *nodes += 1;
    if *nodes > MAX_NODES {
        return Err("serialized value exceeds one million values".to_owned());
    }
    if depth > MAX_DEPTH {
        return Err("serialized value exceeds 64 nesting levels".to_owned());
    }
    if value.is_null() {
        append(output, &[0])
    } else if let Some(boolean) = value.as_bool() {
        append(output, &[if boolean { 2 } else { 1 }])
    } else if let Some(integer) = value.as_int() {
        append(output, &[3])?;
        append(output, &integer.to_le_bytes())
    } else if let Some(float) = value.as_float() {
        append(output, &[4])?;
        append(output, &float.to_bits().to_le_bytes())
    } else if let Some(bytes) = value.as_bytes() {
        append(output, &[5])?;
        append(
            output,
            &u32::try_from(bytes.len())
                .map_err(|_| "string is too long")?
                .to_le_bytes(),
        )?;
        append(output, bytes)
    } else if let Some(values) = value.vector_values() {
        append(output, &[6])?;
        append(
            output,
            &u32::try_from(values.len())
                .map_err(|_| "vector is too long")?
                .to_le_bytes(),
        )?;
        for value in values {
            encode(value, output, depth + 1, nodes)?;
        }
        Ok(())
    } else if let Some(entries) = value.map_entries() {
        append(output, &[7])?;
        append(
            output,
            &u32::try_from(entries.len())
                .map_err(|_| "map is too long")?
                .to_le_bytes(),
        )?;
        for (key, value) in entries {
            if key.as_int().is_none() && key.as_bytes().is_none() {
                return Err("map keys must be int or string".to_owned());
            }
            encode(key, output, depth + 1, nodes)?;
            encode(value, output, depth + 1, nodes)?;
        }
        Ok(())
    } else {
        Err(format!("{} is not serializable", value.type_name()))
    }
}

#[derive(Debug)]
pub(super) enum DecodeError {
    Malformed(&'static str),
    Runtime(RuntimeErrorKind),
}

impl From<&'static str> for DecodeError {
    fn from(message: &'static str) -> Self {
        Self::Malformed(message)
    }
}

pub(super) fn unserialize(bytes: &[u8]) -> Result<Value, DecodeError> {
    if bytes.len() > MAX_BYTES || !bytes.starts_with(MAGIC) {
        return Err(DecodeError::Malformed(
            "invalid THP serialization header or size",
        ));
    }
    let mut decoder = Decoder {
        bytes,
        offset: MAGIC.len(),
        nodes: 0,
    };
    let value = decoder.decode(0)?;
    if decoder.offset != bytes.len() {
        return Err(DecodeError::Malformed("trailing serialized bytes"));
    }
    Ok(value)
}

struct Decoder<'a> {
    bytes: &'a [u8],
    offset: usize,
    nodes: usize,
}

impl Decoder<'_> {
    fn take(&mut self, count: usize) -> Result<&[u8], DecodeError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or("serialized length overflow")?;
        let bytes = self
            .bytes
            .get(self.offset..end)
            .ok_or("truncated serialized value")?;
        self.offset = end;
        Ok(bytes)
    }

    fn u32(&mut self) -> Result<usize, DecodeError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()) as usize)
    }

    fn decode(&mut self, depth: usize) -> Result<Value, DecodeError> {
        self.nodes += 1;
        if self.nodes > MAX_NODES {
            return Err(DecodeError::Malformed(
                "serialized value exceeds one million values",
            ));
        }
        if depth > MAX_DEPTH {
            return Err(DecodeError::Malformed(
                "serialized value exceeds 64 nesting levels",
            ));
        }
        let tag = self.take(1)?[0];
        match tag {
            0 => Ok(Value::NULL),
            1 => Ok(Value::bool(false)),
            2 => Ok(Value::bool(true)),
            3 => Ok(Value::integer(i64::from_le_bytes(
                self.take(8)?.try_into().unwrap(),
            ))),
            4 => Ok(Value::float(f64::from_bits(u64::from_le_bytes(
                self.take(8)?.try_into().unwrap(),
            )))),
            5 => {
                let count = self.u32()?;
                Value::try_bytes(self.take(count)?.to_vec()).map_err(DecodeError::Runtime)
            }
            6 => {
                let count = self.u32()?;
                if count > self.bytes.len() - self.offset {
                    return Err(DecodeError::Malformed("truncated serialized vector"));
                }
                let mut values = Vec::new();
                for _ in 0..count {
                    values.push(self.decode(depth + 1)?);
                }
                Value::try_vector(Type::Mixed, values).map_err(DecodeError::Runtime)
            }
            7 => {
                let count = self.u32()?;
                if count > (self.bytes.len() - self.offset) / 2 {
                    return Err(DecodeError::Malformed("truncated serialized map"));
                }
                let mut entries = Vec::new();
                let mut keys = HashSet::new();
                for _ in 0..count {
                    let start = self.offset;
                    let key = self.decode(depth + 1)?;
                    if key.as_int().is_none() && key.as_bytes().is_none() {
                        return Err(DecodeError::Malformed("invalid serialized map key"));
                    }
                    if !keys.insert(self.bytes[start..self.offset].to_vec()) {
                        return Err(DecodeError::Malformed("duplicate serialized map key"));
                    }
                    entries.push((key, self.decode(depth + 1)?));
                }
                Value::try_map(
                    Type::Union(vec![Type::Int, Type::String]),
                    Type::Mixed,
                    entries,
                )
                .map_err(DecodeError::Runtime)
            }
            _ => Err(DecodeError::Malformed("unknown serialized value tag")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{serialize, unserialize};
    use thp_runtime::Value;

    #[test]
    fn version_one_integer_bytes_and_malformed_documents() {
        let bytes = serialize(&Value::integer(-2)).unwrap();
        assert_eq!(
            bytes,
            [b"THPS\x01\x03".as_slice(), &(-2_i64).to_le_bytes()].concat()
        );
        assert_eq!(unserialize(&bytes).unwrap().as_int(), Some(-2));
        assert!(unserialize(b"THPS\x01\x03").is_err());
        assert!(unserialize(b"THPS\x01\x08").is_err());
        assert!(unserialize(b"THPS\x01\x00\x00").is_err());
    }
}
