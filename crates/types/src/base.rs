macro_rules! non_empty_text {
    ($name:ident) => {
        non_empty_text!($name, {});
    };
    ($name:ident, {$(#[$doc:meta])*}) => {
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
            // SeaORM
            sea_orm::DeriveValueType,
        )]
        pub struct $name(String);

        impl $name {
            pub fn try_new(value: &str) -> Result<Self, $crate::error::TypeError> {
                if value.is_empty() {
                    return Err($crate::error::ValidationError {
                        value: value.to_string(),
                        message: format!("Empty value is not allowed for {}", stringify!($name)),
                    })?;
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
pub(crate) use non_empty_text;

macro_rules! text {
    ($name:ident) => {
        text!($name, {});
    };
    ($name:ident, {$(#[$doc:meta])*}) => {
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
            // SeaORM
            sea_orm::DeriveValueType,
        )]
        pub struct $name(String);

        impl $name {
            pub fn new(value: &str) -> Self {
                Self(value.to_string())
            }

            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl sea_orm::sea_query::Nullable for $name {
            fn null() -> sea_orm::Value {
                sea_orm::Value::String(None)
            }
        }
    };
}
pub(crate) use text;

macro_rules! digits {
    // Entry points with optional doc comments
    ($name:ident, $allow_empty:tt, $digits:expr) => {
        digits!($name, $allow_empty, $digits, {});
    };
    ($name:ident, $allow_empty:tt, $digits:expr, {$(#[$doc:meta])*}) => {
        digits!(@common $name, $digits, {$(#[$doc])*});
        digits!(@impl $name, $allow_empty, $digits);
        digits!(@nullable $name, $allow_empty);
    };

    // Common derivations and implementations
    (@common $name:ident, $digits:expr, {$(#[$doc:meta])*}) => {
        $(#[$doc])*
        #[derive(
            // std
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
            derive_more::Into,
            // serde
            serde::Serialize,
            serde::Deserialize,
            // diesel
            diesel_derive_newtype::DieselNewType,
            // SeaORM
            sea_orm::DeriveValueType,
        )]
        pub struct $name(String);

        impl TryFrom<&str> for $name {
            type Error = $crate::error::TypeError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::try_new(value)
            }
        }

        #[cfg(test)]
        impl<T> fake::Dummy<T> for $name {
            fn dummy_with_rng<R: fake::Rng + ?Sized>(_config: &T, _rng: &mut R) -> Self {
                let format = format!("^{}", "#".repeat($digits));
                fake::faker::number::raw::NumberWithFormat(fake::locales::EN, &format)
                    .fake::<String>()
                    .as_str()
                    .try_into()
                    .expect("Failed to create mock")
            }
        }

        impl sea_orm::TryFromU64 for $name {
            fn try_from_u64(value: u64) -> Result<Self, sea_orm::DbErr> {
                Self::try_new(&value.to_string()).map_err(|e| {
                    tracing::error!(?e, concat!("Failed to convert u64 to ", stringify!($name)));
                    sea_orm::DbErr::ConvertFromU64(stringify!($name))
                })
            }
        }
    };

    // Implementation with conditional empty check
    (@impl $name:ident, $allow_empty:tt, $digits:expr) => {
        impl $name {
            pub fn try_new(value: &str) -> Result<Self, $crate::error::TypeError> {
                if value.is_empty() {
                    return if $allow_empty {
                        Ok(Self(value.to_string()))
                    } else {
                        Err($crate::error::ValidationError {
                            value: value.to_string(),
                            message: format!("Empty value is not allowed for {}", stringify!($name)),
                        })?
                    };
                }

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
    };

    // Nullable implementation only for allow_empty = true
    (@nullable $name:ident, true) => {
        impl sea_orm::sea_query::Nullable for $name {
            fn null() -> sea_orm::Value {
                sea_orm::Value::String(None)
            }
        }
    };
    (@nullable $name:ident, false) => {};
}
pub(crate) use digits;
