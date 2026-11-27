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

/// Same as `compute_diff`, for a forum item edit whose `new` side carries an `allowed_user_ids`
/// list the stored item doesn't have as a field. Both id lists must be sorted for the comparison.
pub fn compute_diff_with_allowed_users<T: Serialize, U: Serialize>(
    old: &T,
    old_allowed_user_ids: &[i32],
    new: &U,
    skip_fields: &[&str],
) -> Option<Value> {
    let mut old = serde_json::to_value(old).ok()?;
    old.as_object_mut()?
        .insert("allowed_user_ids".to_string(), json!(old_allowed_user_ids));
    compute_diff(&old, new, skip_fields)
}
