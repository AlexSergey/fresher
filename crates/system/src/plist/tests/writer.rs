//! A tiny test-only writer of binary property lists, so the tests state their bytes explicitly.

/// One object of a binary property list; containers hold the indexes of other objects.
#[derive(Clone, Debug)]
pub(super) enum Object {
    Ascii(String),
    Utf16(String),
    Int(u64),
    Bool(bool),
    Array(Vec<usize>),
    Dict(Vec<(usize, usize)>),
}

#[derive(Debug, Default)]
pub(super) struct Writer {
    objects: Vec<Object>,
}

impl Writer {
    pub(super) fn add(&mut self, object: Object) -> usize {
        self.objects.push(object);
        self.objects.len() - 1
    }

    pub(super) fn ascii(&mut self, text: &str) -> usize {
        self.add(Object::Ascii(text.to_owned()))
    }

    pub(super) fn utf16(&mut self, text: &str) -> usize {
        self.add(Object::Utf16(text.to_owned()))
    }

    /// Encodes the objects with `top` as the top-level object.
    pub(super) fn finish(&self, top: usize) -> Vec<u8> {
        let ref_size: u8 = if self.objects.len() <= 0x100 { 1 } else { 2 };
        let mut bytes = b"bplist00".to_vec();
        let mut offsets = Vec::new();
        for object in &self.objects {
            offsets.push(bytes.len() as u64);
            encode(object, ref_size, &mut bytes);
        }
        let table = bytes.len() as u64;
        let offset_size = minimal_size(table);
        for offset in offsets {
            push_be(&mut bytes, offset, offset_size);
        }
        bytes.extend_from_slice(&[0; 6]);
        bytes.push(offset_size);
        bytes.push(ref_size);
        push_be(&mut bytes, self.objects.len() as u64, 8);
        push_be(&mut bytes, top as u64, 8);
        push_be(&mut bytes, table, 8);
        bytes
    }
}

/// A binary property list whose top-level object is a dictionary of ASCII strings.
pub(crate) fn dict_of_strings(entries: &[(&str, &str)]) -> Vec<u8> {
    let mut writer = Writer::default();
    let pairs = entries.iter().map(|(key, value)| (writer.ascii(key), writer.ascii(value))).collect();
    let top = writer.add(Object::Dict(pairs));
    writer.finish(top)
}

fn encode(object: &Object, ref_size: u8, bytes: &mut Vec<u8>) {
    match object {
        Object::Ascii(text) => {
            push_marker(bytes, 0x5, text.len());
            bytes.extend_from_slice(text.as_bytes());
        }
        Object::Utf16(text) => {
            let units: Vec<u16> = text.encode_utf16().collect();
            push_marker(bytes, 0x6, units.len());
            for unit in units {
                bytes.extend_from_slice(&unit.to_be_bytes());
            }
        }
        Object::Int(value) => {
            bytes.push(0x13);
            push_be(bytes, *value, 8);
        }
        Object::Bool(value) => bytes.push(if *value { 0x09 } else { 0x08 }),
        Object::Array(items) => {
            push_marker(bytes, 0xA, items.len());
            for &item in items {
                push_be(bytes, item as u64, ref_size);
            }
        }
        Object::Dict(pairs) => {
            push_marker(bytes, 0xD, pairs.len());
            for &(key, _) in pairs {
                push_be(bytes, key as u64, ref_size);
            }
            for &(_, value) in pairs {
                push_be(bytes, value as u64, ref_size);
            }
        }
    }
}

/// A marker with its length: in the low nibble below 15, else as a following integer object.
fn push_marker(bytes: &mut Vec<u8>, kind: u8, len: usize) {
    if len < 15 {
        bytes.push((kind << 4) | u8::try_from(len).unwrap());
        return;
    }
    bytes.push((kind << 4) | 0xF);
    let size = minimal_size(len as u64);
    bytes.push(0x10 | u8::try_from(size.trailing_zeros()).unwrap());
    push_be(bytes, len as u64, size);
}

/// The fewest bytes, a power of two, that hold `value`.
fn minimal_size(value: u64) -> u8 {
    match value {
        0..=0xFF => 1,
        0x100..=0xFFFF => 2,
        0x1_0000..=0xFFFF_FFFF => 4,
        _ => 8,
    }
}

fn push_be(bytes: &mut Vec<u8>, value: u64, size: u8) {
    bytes.extend_from_slice(&value.to_be_bytes()[8 - usize::from(size)..]);
}
