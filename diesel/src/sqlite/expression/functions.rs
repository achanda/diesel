//! SQLite specific functions
use crate::expression::functions::define_sql_function;
use crate::sql_types::*;
use crate::sqlite::expression::expression_methods::BinaryOrNullableBinary;
use crate::sqlite::expression::expression_methods::MaybeNullableValue;
use crate::sqlite::expression::expression_methods::TextOrNullableText;
use crate::sqlite::expression::expression_methods::TextOrNullableTextOrBinaryOrNullableBinary;

#[cfg(feature = "sqlite")]
define_sql_function! {
    /// Verifies that its argument is a valid JSON string or JSONB blob and returns a minified
    /// version of that JSON string with all unnecessary whitespace removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// # include!("../../doctest_setup.rs");
    /// #
    /// # fn main() {
    /// #     #[cfg(feature = "serde_json")]
    /// #     run_test().unwrap();
    /// # }
    /// #
    /// # #[cfg(feature = "serde_json")]
    /// # fn run_test() -> QueryResult<()> {
    /// #     use diesel::dsl::json;
    /// #     use serde_json::{json, Value};
    /// #     use diesel::sql_types::{Text, Nullable};
    /// #     let connection = &mut establish_connection();
    ///
    /// let result = diesel::select(json::<Text, _>(r#"{"a": "b", "c": 1}"#))
    ///     .get_result::<Value>(connection)?;
    ///
    /// assert_eq!(json!({"a":"b","c":1}), result);
    ///
    /// let result = diesel::select(json::<Text, _>(r#"{ "this" : "is", "a": [ "test" ] }"#))
    ///     .get_result::<Value>(connection)?;
    ///
    /// assert_eq!(json!({"a":["test"],"this":"is"}), result);
    ///
    /// let result = diesel::select(json::<Nullable<Text>, _>(None::<&str>))
    ///     .get_result::<Option<Value>>(connection)?;
    ///
    /// assert!(result.is_none());
    ///
    /// #     Ok(())
    /// # }
    /// ```
    fn json<E: TextOrNullableText + MaybeNullableValue<Json>>(e: E) -> E::Out;
}

#[cfg(feature = "sqlite")]
define_sql_function! {
    /// The jsonb(X) function returns the binary JSONB representation of the JSON provided as argument X.
    ///
    /// # Example
    ///
    /// ```rust
    /// # include!("../../doctest_setup.rs");
    /// #
    /// # fn main() {
    /// #     #[cfg(feature = "serde_json")]
    /// #     run_test().unwrap();
    /// # }
    /// #
    /// # #[cfg(feature = "serde_json")]
    /// # fn run_test() -> QueryResult<()> {
    /// #     use diesel::dsl::{sql, jsonb};
    /// #     use serde_json::{json, Value};
    /// #     use diesel::sql_types::{Text, Binary, Nullable};
    /// #     let connection = &mut establish_connection();
    ///
    /// let version = diesel::select(sql::<Text>("sqlite_version();"))
    ///         .get_result::<String>(connection)?;
    ///
    /// // Querying SQLite version should not fail.
    /// let version_components: Vec<&str> = version.split('.').collect();
    /// let major: u32 = version_components[0].parse().unwrap();
    /// let minor: u32 = version_components[1].parse().unwrap();
    /// let patch: u32 = version_components[2].parse().unwrap();
    ///
    /// if major > 3 || (major == 3 && minor >= 45) {
    ///     /* Valid sqlite version, do nothing */
    /// } else {
    ///     println!("SQLite version is too old, skipping the test.");
    ///     return Ok(());
    /// }
    ///
    /// let result = diesel::select(jsonb::<Binary, _>(br#"{"a": "b", "c": 1}"#))
    ///     .get_result::<Value>(connection)?;
    ///
    /// assert_eq!(json!({"a": "b", "c": 1}), result);
    ///
    /// let result = diesel::select(jsonb::<Binary, _>(br#"{"this":"is","a":["test"]}"#))
    ///     .get_result::<Value>(connection)?;
    ///
    /// assert_eq!(json!({"this":"is","a":["test"]}), result);
    ///
    /// let result = diesel::select(jsonb::<Nullable<Binary>, _>(None::<Vec<u8>>))
    ///     .get_result::<Option<Value>>(connection)?;
    ///
    /// assert!(result.is_none());
    ///
    /// #     Ok(())
    /// # }
    /// ```
    fn jsonb<E: BinaryOrNullableBinary + MaybeNullableValue<Jsonb>>(e: E) -> E::Out;
}

#[cfg(feature = "sqlite")]
define_sql_function! {
    /// Converts the given json value to pretty-printed, indented text
    ///
    /// /// # Example
    ///
    /// ```rust
    /// # include!("../../doctest_setup.rs");
    /// #
    /// # fn main() {
    /// #     #[cfg(feature = "serde_json")]
    /// #     run_test().unwrap();
    /// # }
    /// #
    /// # #[cfg(feature = "serde_json")]
    /// # fn run_test() -> QueryResult<()> {
    /// #     use diesel::dsl::json_pretty;
    /// #     use serde_json::{json, Value};
    /// #     use diesel::sql_types::{Text, Binary, Nullable};
    /// #     let connection = &mut establish_connection();
    ///
    /// // Querying SQLite version should not fail.
    /// let version_components: Vec<&str> = version.split('.').collect();
    /// let major: u32 = version_components[0].parse().unwrap();
    /// let minor: u32 = version_components[1].parse().unwrap();
    /// let patch: u32 = version_components[2].parse().unwrap();
    ///
    /// if major > 3 || (major == 3 && minor >= 46) {
    ///     /* Valid sqlite version, do nothing */
    /// } else {
    ///     println!("SQLite version is too old, skipping the test.");
    ///     return Ok(());
    /// }
    ///
    /// let result = diesel::select(json_pretty::<Text, _>(r#"[{"f1":1,"f2":null},2,null,3]"#))
    ///     .get_result::<String>(connection)?;
    ///
    /// assert_eq!(r#"[
    ///     {
    ///         "f1": 1,
    ///         "f2": null
    ///     },
    ///     2,
    ///     null,
    ///     3
    /// ]"#, result);
    ///
    /// let result = diesel::select(json_pretty::<Binary, _>(br#"[{"f1":1,"f2":null},2,null,3]"#))
    ///     .get_result::<String>(connection)?;
    ///
    /// assert_eq!(r#"[
    ///     {
    ///         "f1": 1,
    ///         "f2": null
    ///     },
    ///     2,
    ///     null,
    ///     3
    /// ]"#, result);
    ///
    /// let result = diesel::select(json_pretty::<Text, _>(r#"{"a": 1, "b": "cd"}"#))
    ///     .get_result::<String>(connection)?;
    ///
    /// assert_eq!(r#"{
    ///     "a": 1,
    ///     "b": "cd"
    /// }"#, result);
    ///
    /// let result = diesel::select(json_pretty::<Text, _>(r#""abc""#))
    ///     .get_result::<String>(connection)?;
    ///
    /// assert_eq!(r#""abc""#, result);
    ///
    /// let result = diesel::select(json_pretty::<Text, _>(r#"22"#))
    ///     .get_result::<String>(connection)?;
    ///
    /// assert_eq!(r#"22"#, result);
    ///
    /// let result = diesel::select(json_pretty::<Text, _>(r#"false"#))
    ///     .get_result::<String>(connection)?;
    ///
    /// assert_eq!(r#"false"#, result);
    ///
    /// let result = diesel::select(json_pretty::<Text, _>(r#"null"#))
    ///     .get_result::<String>(connection)?;
    ///
    /// assert_eq!(r#"null"#, result);
    ///
    /// let result = diesel::select(json_pretty::<Text, _>(r#"{}"#))
    ///     .get_result::<String>(connection)?;
    ///
    /// assert_eq!(r#"{}"#, result);
    ///
    /// let result = diesel::select(json_pretty::<Nullable<Text>, _>(None::<&str>))
    ///     .get_result::<Option<String>>(connection)?;
    ///
    /// assert!(result.is_none());
    ///
    /// #     Ok(())
    /// # }
    /// ```
    fn json_pretty<E: TextOrNullableTextOrBinaryOrNullableBinary + MaybeNullableValue<Text>>(e: E) -> E::Out;
}
