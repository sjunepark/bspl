#[macro_export]
macro_rules! string {
    // Pattern for structs with additional nutype attributes
    ($name:ident, [$($custom_derives:ident),*] {$(#[$meta:meta])*} => { $($nutype_attrs:tt)+ }) => {
        $(#[$meta])*
        #[nutype::nutype(
            $(
                $nutype_attrs
            )+
            derive(
                Clone, Eq, PartialEq, Ord, PartialOrd,
                Debug, Display, Serialize, Deserialize, Deref,
                FromStr, Into, Hash, $($custom_derives),*
            ),
        )]
         pub struct $name(String);

        $crate::assert_impl_commons_without_default!($name);
    };
    // Pattern for structs without additional nutype attributes
    ($name:ident, [$($custom_derives:ident),*] {$(#[$meta:meta])*}) => {
        $(#[$meta])*
        #[nutype::nutype(
            derive(
                Clone, Eq, PartialEq, Ord, PartialOrd,
                Debug, Display, Serialize, Deserialize, Deref,
                FromStr, Into, Hash, $($custom_derives),*
            ),
        )]
        pub struct $name(String);

        $crate::assert_impl_commons_without_default!($name);
    };
}

#[macro_export]
macro_rules! bytes {
    // Pattern for structs with additional nutype attributes
    ($name:ident, [$($custom_derives:ident),*] {$(#[$meta:meta])*} => { $($nutype_attrs:tt)+ }) => {
        $(#[$meta])*
        #[nutype::nutype(
            $(
                $nutype_attrs
            )+
            derive(
                Clone, Eq, PartialEq, Ord, PartialOrd,
                Debug, Serialize, Deserialize, Deref,
                Into, Hash, $($custom_derives),*
            ),
        )]
        pub struct $name(Vec<u8>);

        impl $name {
            pub fn utf8_string(&self) -> Result<String, std::string::FromUtf8Error> {
                String::from_utf8(std::ops::Deref::deref(self).clone())
            }
        }

       impl std::str::FromStr for $name {
            type Err = $crate::error::FromStrError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Ok($name::try_new(s.as_bytes().to_vec())?)
            }
        }
    };
    // Pattern for structs without additional nutype attributes
    ($name:ident, [$($custom_derives:ident),*] {$(#[$meta:meta])*}) => {
        $(#[$meta])*
        #[nutype::nutype(
            derive(
                Clone, Eq, PartialEq, Ord, PartialOrd,
                Debug, Serialize, Deserialize, Deref,
                Into, Hash, $($custom_derives),*
            ),
        )]
        pub struct $name(Vec<u8>);

        impl $name {
            pub fn utf8_string(&self) -> Result<String, std::string::FromUtf8Error> {
                String::from_utf8(std::ops::Deref::deref(self).clone())
            }
        }

       impl From<&str> for $name {
            fn from(s: &str) -> Self {
                $name::new(s.as_bytes().to_vec())
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    string!(Inner, [From] {});

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
