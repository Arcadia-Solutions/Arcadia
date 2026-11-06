use serde::Serialize;
use serde_json::{Value, json};

/// Computes the diff between two serializable structs.
/// Returns None if there are no changes, otherwise returns a JSON object with the changes.
/// Fields in `skip_fields` will be excluded from the comparison.
pub fn compute_diff<T: Serialize, U: Serialize>(
    old: &T,
    new: &U,
    skip_fields: &[&str],
) -> Option<Value> {
    let old = serde_json::to_value(old).ok()?;
    let new = serde_json::to_value(new).ok()?;
    let (old_obj, new_obj) = (old.as_object()?, new.as_object()?);

    let edits: serde_json::Map<_, _> = new_obj
        .iter()
        .filter(|(k, _)| !skip_fields.contains(&k.as_str()))
        .filter(|(k, new_v)| old_obj.get(*k) != Some(*new_v))
        .map(|(k, new_v)| (k.clone(), json!({"old": old_obj.get(k), "new": new_v})))
        .collect();

    (!edits.is_empty()).then(|| Value::Object(edits))
}
