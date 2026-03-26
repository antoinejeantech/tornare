use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NullableField<T> {
    Missing,
    Null,
    Value(T),
}

impl<T> Default for NullableField<T> {
    fn default() -> Self {
        Self::Missing
    }
}

impl<'de, T> Deserialize<'de> for NullableField<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(match Option::<T>::deserialize(deserializer)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::NullableField;

    #[derive(Deserialize)]
    struct Payload {
        #[serde(default)]
        field: NullableField<u32>,
    }

    #[test]
    fn nullable_field_distinguishes_missing() {
        let payload: Payload = serde_json::from_str("{}").expect("payload should deserialize");
        assert_eq!(payload.field, NullableField::Missing);
    }

    #[test]
    fn nullable_field_distinguishes_null() {
        let payload: Payload =
            serde_json::from_str(r#"{"field":null}"#).expect("payload should deserialize");
        assert_eq!(payload.field, NullableField::Null);
    }

    #[test]
    fn nullable_field_distinguishes_value() {
        let payload: Payload =
            serde_json::from_str(r#"{"field":7}"#).expect("payload should deserialize");
        assert_eq!(payload.field, NullableField::Value(7));
    }
}