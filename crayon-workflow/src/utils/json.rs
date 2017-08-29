use serde_json::Value;

pub fn load<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut next = value;
    for i in path {
        if let Some(leaf) = next.get(i) {
            next = leaf;
        } else {
            return None;
        }
    }
    Some(next)

}

pub fn load_as_str<'a>(value: &'a Value, path: &[&str]) -> Option<&'a str> {
    load(&value, path).and_then(|v| v.as_str())
}

pub fn load_as_f32(value: &Value, path: &[&str]) -> Option<f32> {
    load(&value, path)
        .and_then(|v| v.as_f64())
        .and_then(|v| Some(v as f32))
}

pub fn load_as_u16(value: &Value, path: &[&str]) -> Option<u16> {
    load(&value, path)
        .and_then(|v| v.as_u64())
        .and_then(|v| Some(v as u16))
}