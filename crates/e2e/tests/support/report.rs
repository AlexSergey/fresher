//! What a JSON report leaves out or puts in order before a golden file compares it. Pulled in with `#[path]` only by
//! the homes that use it.

use serde_json::Value;

/// Removes every field named `key`, at any depth.
pub(crate) fn strip(value: &mut Value, key: &str) {
    match value {
        Value::Object(fields) => {
            fields.remove(key);
            fields.values_mut().for_each(|field| strip(field, key));
        }
        Value::Array(items) => items.iter_mut().for_each(|item| strip(item, key)),
        _ => {}
    }
}

/// Sorts every list of objects that all name a place (`path` or `original`), by it, at any depth.
pub(crate) fn sort_by_path(value: &mut Value) {
    fn place(item: &Value) -> Option<String> {
        item["path"].as_str().or_else(|| item["original"].as_str()).map(str::to_owned)
    }
    match value {
        Value::Object(fields) => fields.values_mut().for_each(sort_by_path),
        Value::Array(items) => {
            items.iter_mut().for_each(sort_by_path);
            if items.iter().all(|item| place(item).is_some()) {
                items.sort_by_key(|item| place(item).unwrap_or_default());
            }
        }
        _ => {}
    }
}

/// `text` with the fake Trash's order number dropped from the names it gives in the folder `trash`, written with its
/// trailing `/` (in `trash/`, `trash/3-Name` is `trash/Name`).
pub(crate) fn trash_unnumbered(text: &str, trash: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find(trash) {
        let (before, after) = rest.split_at(at + trash.len());
        out.push_str(before);
        let digits = after.bytes().take_while(u8::is_ascii_digit).count();
        let named = after.get(digits..).and_then(|tail| tail.strip_prefix('-'));
        rest = named.filter(|_| digits > 0).unwrap_or(after);
    }
    out.push_str(rest);
    out
}
