#[macro_export]
macro_rules! function_id {
    () => {{
        fn f() {}
        fn type_name_of_val<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let mut name = type_name_of_val(f).strip_suffix("::f").unwrap_or("");
        while let Some(rest) = name.strip_suffix("::{{closure}}") {
            name = rest;
        }
        name.replace("::", "-")
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn function_id_should_return_expected() {
        let id = function_id!();
        assert_eq!(id, "utils-id-tests-function_id_should_return_expected");
    }
}
