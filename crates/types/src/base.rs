macro_rules! text {
    ($name:ident, $allow_empty:expr) => {
        text!($name, $allow_empty, {});
    };
    ($name:ident, $allow_empty:expr, {$(#[$doc:meta])*}) => {
        $(#[$doc])*
        #[derive(
            std::fmt::Debug,
            Clone,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            // derive_more
            derive_more::AsRef,
            derive_more::Display,
            derive_more::From,
            derive_more::Into,
            // serde
            serde::Serialize,
            serde::Deserialize,
            // diesel
            diesel_derive_newtype::DieselNewType,
        )]
        pub struct $name(String);

        impl $name {
            pub fn try_new(value: &str) -> Result<Self, $crate::error::TypeError> {
                if value.is_empty() {
                    if $allow_empty {
                        return Ok(Self(value.to_string()));
                    } else {
                        return Err($crate::error::ValidationError {
                            value: value.to_string(),
                            message: format!("Empty value is not allowed for {}", stringify!($name)),
                        })?;
                    }
                };
                Ok(Self(value.to_string()))
            }

            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $crate::error::TypeError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_new(value)
            }
        }
    };
}
pub(crate) use text;

macro_rules! digit {
    ($name:ident, $allow_empty:expr, $digits:expr) => {
        digit!($name, $allow_empty, $digits, {});
    };
    ($name:ident, $allow_empty:expr, $digits:expr, {$(#[$doc:meta])*}) => {
        $(#[$doc])*
        #[derive(
            std::fmt::Debug,
            Clone,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Hash,
            // derive_more
            derive_more::AsRef,
            derive_more::Display,
            derive_more::From,
            derive_more::Into,
            // serde
            serde::Serialize,
            serde::Deserialize,
            // diesel
            diesel_derive_newtype::DieselNewType,
        )]
        pub struct $name(String);

        impl $name {
            pub fn try_new(value: &str) -> Result<Self, $crate::error::TypeError> {
                if value.is_empty() {
                    if $allow_empty {
                        return Ok(Self(value.to_string()));
                    } else {
                        return Err($crate::error::ValidationError {
                            value: value.to_string(),
                            message: format!("Empty value is not allowed for {}", stringify!($name)),
                        })?;
                    }
                };

                if value.len() == $digits && value.chars().all(|c| c.is_ascii_digit()) {
                    Ok(Self(value.to_string()))
                } else {
                    Err($crate::error::ValidationError {
                        value: value.to_string(),
                        message: concat!(stringify!($name), " must be ", $digits, " digits").to_string(),
                    })?
                }
            }

            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl TryFrom<&str> for $name {
            type Error = $crate::error::TypeError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_new(value)
            }
        }
    };
}
pub(crate) use digit;
