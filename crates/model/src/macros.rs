#[macro_export]
macro_rules! string {
    // Pattern for structs with additional nutype attributes
        ($name:ident => { $($nutype_attrs:tt)+ }) => {
        #[nutype::nutype(
            $(
                $nutype_attrs
            )+
            derive(
                Clone, Eq, PartialEq, Ord, PartialOrd,
                Debug, Display, Serialize, Deserialize, Deref,
                TryFrom, FromStr, Into, Hash
            ),
        )]
         pub struct $name(String);

        $crate::assert_impl_commons_without_default!($name);
    };
    // Pattern for structs without additional nutype attributes
    ($name:ident) => {
        #[nutype::nutype(
            derive(
                Clone, Eq, PartialEq, Ord, PartialOrd,
                Debug, Display, Serialize, Deserialize, Deref,
                TryFrom, FromStr, Into, Hash
            ),
        )]
        pub struct $name(String);

        $crate::assert_impl_commons_without_default!($name);
    };
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    string!(Inner);

    #[derive(Serialize, Deserialize)]
    struct SomeStruct {
        field: Inner,
    }

    #[test]
    fn string_macro_should_serialize_as_expected() {
        let s = Inner::new("value".to_string());
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#""value""#);
    }

    #[test]
    fn string_macro_should_deserialize_as_expected() {
        let json = r#""value""#;
        let s: Inner = serde_json::from_str(json).unwrap();
        assert_eq!(s, Inner::new("value".to_string()));
    }

    #[test]
    fn string_macro_within_struct_should_serialize_as_expected() {
        let s = SomeStruct {
            field: Inner::new("value".to_string()),
        };
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, r#"{"field":"value"}"#);
    }

    #[test]
    fn string_macro_within_struct_should_deserialize_as_expected() {
        let json = r#"{"field":"value"}"#;
        let s: SomeStruct = serde_json::from_str(json).unwrap();
        assert_eq!(s.field, Inner::new("value".to_string()));
    }
}
