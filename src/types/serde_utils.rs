//! Internal ser/de utilities.
// #![macro_use]

pub(crate) mod default_on_null {
    /// Serializes an empty `Vec` to `None` instead of `[]`.
    ///
    /// The Plaid API will return an `Error` if `[]` is passed as an input to a
    /// request instead of `null`.
    pub fn serialize<S, T>(value: &[T], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: serde::Serialize,
    {
        if value.is_empty() {
            serializer.serialize_none()
        } else {
            serializer.serialize_some(value)
        }
    }

    // taken from: https://github.com/jonasbb/serde_with
    /// Deserialize `T` and return the [`Default`] value if original value is `null`
    #[allow(dead_code)]
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::de::Deserializer<'de>,
        T: serde::Deserialize<'de> + Default,
    {
        use serde::Deserialize;
        Ok(Option::deserialize(deserializer)?.unwrap_or_default())
    }

    #[cfg(test)]
    mod tests {
        use serde_json::json;

        #[derive(serde::Serialize, serde::Deserialize, Debug)]
        struct TestValue {
            #[serde(default, with = "super")]
            test_field: Vec<String>,
        }

        #[test]
        fn can_serde_non_empty() {
            let val = TestValue {
                test_field: vec!["test_id".to_string()],
            };
            let json_val = serde_json::to_string(&val).unwrap();
            serde_json::from_str::<TestValue>(&json_val).unwrap();
        }

        #[test]
        fn can_serde_empty() {
            let original_val = TestValue { test_field: vec![] };
            let json_val = serde_json::to_value(&original_val).unwrap();
            assert_eq!(json_val, json!({ "test_field": null }));
            let actual_val: TestValue = serde_json::from_value(json_val).unwrap();
            assert_eq!(actual_val.test_field, Vec::<String>::new());
        }
    }
}

// TODO: is there a crate or something that will support this?
// HACK: https://github.com/serde-rs/serde/issues/1560
macro_rules! named_unit_variant {
    ($variant:ident) => {
        pub(crate) mod $variant {
            #[allow(dead_code)]
            pub fn serialize<S>(serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                serializer.serialize_str(stringify!($variant))
            }

            pub fn deserialize<'de, D>(deserializer: D) -> Result<(), D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                struct UnitVisitor;

                impl<'de> serde::de::Visitor<'de> for UnitVisitor {
                    type Value = ();

                    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                        f.write_str(concat!("\"", stringify!($variant), "\""))
                    }

                    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                        if value == stringify!($variant) {
                            Ok(())
                        } else {
                            Err(E::invalid_value(serde::de::Unexpected::Str(value), &self))
                        }
                    }
                }

                deserializer.deserialize_str(UnitVisitor)
            }
        }
    };
}

pub(crate) mod strings {
    named_unit_variant!(home);
    named_unit_variant!(work);
    named_unit_variant!(office);
    named_unit_variant!(mobile);
    named_unit_variant!(mobile1);
}
