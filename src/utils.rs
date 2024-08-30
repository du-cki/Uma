#[macro_export]
macro_rules! mapping {
    ($($key:expr => $value:expr),*) => {
        std::collections::HashMap::from([$(($key.to_string(), $value.to_string()),)*])
    };
}
