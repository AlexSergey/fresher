//! The XML form: a minimal tokenizer that knows only what a property list uses.

use std::collections::BTreeMap;

use super::PlistError;

/// The deepest nesting of a skipped value.
const MAX_DEPTH: usize = 64;
/// The longest entity name between `&` and `;`, `#x10FFFF` included.
const MAX_ENTITY: usize = 10;

pub(super) fn read_strings(bytes: &[u8]) -> Result<BTreeMap<String, String>, PlistError> {
    let mut reader = Reader { bytes, pos: 0 };
    let mut first = reader.next_element()?;
    let in_plist = matches!(first, Token::Start(b"plist"));
    if in_plist {
        first = reader.next_element()?;
    }
    let map = match first {
        Token::Start(b"dict") => read_dict(&mut reader)?,
        Token::Empty(b"dict") => BTreeMap::new(),
        Token::Empty(b"plist") | Token::End(_) | Token::Eof => return Err(malformed("no top-level object")),
        Token::Start(_) | Token::Empty(_) => return Err(PlistError::NotADictionary),
        Token::Text(_) => return Err(malformed("text outside an element")),
    };
    if in_plist && !matches!(reader.next_element()?, Token::End(b"plist")) {
        return Err(malformed("the plist element is not closed after its object"));
    }
    if !matches!(reader.next_element()?, Token::Eof) {
        return Err(malformed("content after the top-level object"));
    }
    Ok(map)
}

fn read_dict(reader: &mut Reader<'_>) -> Result<BTreeMap<String, String>, PlistError> {
    let mut map = BTreeMap::new();
    loop {
        let key = match reader.next_element()? {
            Token::End(b"dict") => return Ok(map),
            Token::Start(b"key") => reader.read_text(b"key")?,
            Token::Empty(b"key") => String::new(),
            Token::Eof => return Err(malformed("an unclosed dictionary")),
            _ => return Err(malformed("a dictionary entry without a key")),
        };
        match reader.next_element()? {
            Token::Start(b"string") => {
                let value = reader.read_text(b"string")?;
                map.insert(key, value);
            }
            Token::Empty(b"string") => {
                map.insert(key, String::new());
            }
            Token::Empty(_) => {}
            Token::Start(name) => reader.skip_element(name)?,
            _ => return Err(malformed("a key without a value")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token<'a> {
    Start(&'a [u8]),
    End(&'a [u8]),
    Empty(&'a [u8]),
    Text(Text<'a>),
    Eof,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Text<'a> {
    /// Character data whose entities are still encoded.
    Raw(&'a [u8]),
    /// The content of a CDATA section, taken as it is.
    Cdata(&'a [u8]),
}

#[derive(Debug)]
struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    /// The next token that is not whitespace-only text.
    fn next_element(&mut self) -> Result<Token<'a>, PlistError> {
        loop {
            match self.next_token()? {
                Token::Text(Text::Raw(text) | Text::Cdata(text)) if text.iter().all(u8::is_ascii_whitespace) => {}
                token => return Ok(token),
            }
        }
    }

    /// The decoded text of an element whose start tag was just read, up to its end tag.
    fn read_text(&mut self, name: &[u8]) -> Result<String, PlistError> {
        let mut text = Vec::new();
        loop {
            match self.next_token()? {
                Token::Text(Text::Raw(raw)) => decode_entities(raw, &mut text)?,
                Token::Text(Text::Cdata(cdata)) => text.extend_from_slice(cdata),
                Token::End(end) if end == name => break,
                Token::End(_) => return Err(malformed("a mismatched end tag")),
                Token::Start(_) | Token::Empty(_) => return Err(malformed("an element inside text")),
                Token::Eof => return Err(malformed("an unclosed element")),
            }
        }
        String::from_utf8(text).map_err(|_| malformed("text is not UTF-8"))
    }

    /// Skips an element whose start tag was just read, with everything inside it.
    fn skip_element(&mut self, name: &'a [u8]) -> Result<(), PlistError> {
        let mut open = vec![name];
        loop {
            match self.next_token()? {
                Token::Start(inner) => {
                    open.push(inner);
                    if open.len() > MAX_DEPTH {
                        return Err(malformed("nesting is too deep"));
                    }
                }
                Token::End(end) => {
                    if open.pop() != Some(end) {
                        return Err(malformed("a mismatched end tag"));
                    }
                    if open.is_empty() {
                        return Ok(());
                    }
                }
                Token::Empty(_) | Token::Text(_) => {}
                Token::Eof => return Err(malformed("an unclosed element")),
            }
        }
    }

    /// The next token; declarations, processing instructions and comments are skipped.
    fn next_token(&mut self) -> Result<Token<'a>, PlistError> {
        loop {
            let rest = self.bytes.get(self.pos..).unwrap_or_default();
            if rest.is_empty() {
                return Ok(Token::Eof);
            }
            if !rest.starts_with(b"<") {
                let len = rest.iter().position(|&byte| byte == b'<').unwrap_or(rest.len());
                return Ok(Token::Text(Text::Raw(self.advance(len)?)));
            }
            if rest.starts_with(b"<!--") {
                self.skip_past(4, b"-->", "an unclosed comment")?;
            } else if rest.starts_with(b"<![CDATA[") {
                let cdata = self.skip_past(9, b"]]>", "an unclosed CDATA section")?;
                return Ok(Token::Text(Text::Cdata(cdata)));
            } else if rest.starts_with(b"<?") {
                self.skip_past(2, b"?>", "an unclosed processing instruction")?;
            } else if rest.starts_with(b"<!") {
                self.skip_declaration()?;
            } else {
                return self.read_tag();
            }
        }
    }

    /// Skips `skip` bytes and then up to and past `end`; returns what was between.
    fn skip_past(&mut self, skip: usize, end: &[u8], error: &'static str) -> Result<&'a [u8], PlistError> {
        self.advance(skip)?;
        let rest = self.bytes.get(self.pos..).unwrap_or_default();
        let len = rest.windows(end.len()).position(|window| window == end).ok_or(malformed(error))?;
        let inner = self.advance(len)?;
        self.advance(end.len())?;
        Ok(inner)
    }

    /// Skips a `<!DOCTYPE ...>` declaration, an internal subset in brackets included.
    fn skip_declaration(&mut self) -> Result<(), PlistError> {
        let rest = self.bytes.get(self.pos..).unwrap_or_default();
        let mut brackets = 0_usize;
        for (index, &byte) in rest.iter().enumerate() {
            match byte {
                b'[' => brackets = brackets.saturating_add(1),
                b']' => brackets = brackets.saturating_sub(1),
                b'>' if brackets == 0 => {
                    self.advance(index.saturating_add(1))?;
                    return Ok(());
                }
                _ => {}
            }
        }
        Err(malformed("an unclosed declaration"))
    }

    /// Reads a start, end or empty-element tag.
    fn read_tag(&mut self) -> Result<Token<'a>, PlistError> {
        let rest = self.bytes.get(self.pos..).unwrap_or_default();
        let mut quote = None;
        let mut close = None;
        for (index, &byte) in rest.iter().enumerate().skip(1) {
            match (quote, byte) {
                (None, b'"' | b'\'') => quote = Some(byte),
                (Some(open), _) if open == byte => quote = None,
                (None, b'>') => {
                    close = Some(index);
                    break;
                }
                (None, b'<') => return Err(malformed("an unclosed tag")),
                _ => {}
            }
        }
        let close = close.ok_or(malformed("an unclosed tag"))?;
        let tag = self.advance(close.saturating_add(1))?;
        let inner = tag.get(1..close).unwrap_or_default();
        if let Some(name) = inner.strip_prefix(b"/") {
            return Ok(Token::End(tag_name(name.trim_ascii_end())?));
        }
        match inner.strip_suffix(b"/") {
            Some(body) => Ok(Token::Empty(tag_name(body)?)),
            None => Ok(Token::Start(tag_name(inner)?)),
        }
    }

    fn advance(&mut self, len: usize) -> Result<&'a [u8], PlistError> {
        let end = self.pos.checked_add(len).ok_or(malformed("an offset overflows"))?;
        let taken = self.bytes.get(self.pos..end).ok_or(malformed("unexpected end of input"))?;
        self.pos = end;
        Ok(taken)
    }
}

/// The name at the start of a tag body; attributes after it are ignored.
fn tag_name(body: &[u8]) -> Result<&[u8], PlistError> {
    let len = body.iter().position(u8::is_ascii_whitespace).unwrap_or(body.len());
    match body.get(..len) {
        Some(name) if !name.is_empty() => Ok(name),
        _ => Err(malformed("a tag without a name")),
    }
}

/// Appends `raw` to `out` with the predefined and numeric entities decoded.
fn decode_entities(raw: &[u8], out: &mut Vec<u8>) -> Result<(), PlistError> {
    let mut rest = raw;
    while let Some(amp) = rest.iter().position(|&byte| byte == b'&') {
        out.extend_from_slice(rest.get(..amp).unwrap_or_default());
        let after = rest.get(amp.saturating_add(1)..).unwrap_or_default();
        let semi = after
            .iter()
            .take(MAX_ENTITY.saturating_add(1))
            .position(|&byte| byte == b';')
            .ok_or(malformed("an unterminated entity"))?;
        let name = after.get(..semi).unwrap_or_default();
        let decoded = decode_entity(name)?;
        out.extend_from_slice(decoded.encode_utf8(&mut [0; 4]).as_bytes());
        rest = after.get(semi.saturating_add(1)..).unwrap_or_default();
    }
    out.extend_from_slice(rest);
    Ok(())
}

fn decode_entity(name: &[u8]) -> Result<char, PlistError> {
    let (digits, radix) = match name {
        b"amp" => return Ok('&'),
        b"lt" => return Ok('<'),
        b"gt" => return Ok('>'),
        b"quot" => return Ok('"'),
        b"apos" => return Ok('\''),
        [b'#', b'x' | b'X', hex @ ..] if !hex.is_empty() && hex.iter().all(u8::is_ascii_hexdigit) => (hex, 16),
        [b'#', decimal @ ..] if !decimal.is_empty() && decimal.iter().all(u8::is_ascii_digit) => (decimal, 10),
        _ => return Err(malformed("an unknown entity")),
    };
    let code = std::str::from_utf8(digits)
        .ok()
        .and_then(|digits| u32::from_str_radix(digits, radix).ok())
        .filter(|&code| code != 0)
        .and_then(char::from_u32);
    code.ok_or(malformed("an invalid character reference"))
}

fn malformed(reason: &'static str) -> PlistError {
    PlistError::Malformed(reason)
}
