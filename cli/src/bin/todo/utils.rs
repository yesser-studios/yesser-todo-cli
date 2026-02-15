use yesser_todo_db::SaveData;

/// Retrieve the saved cloud server host and port if available.
///
/// # Returns
///
/// `Some((host, port))` when a cloud configuration is present and readable, `None` if no configuration exists or it cannot be read.
///
/// # Examples
///
/// ```
/// // Suppose SaveData::get_cloud_config() returns Ok(Some(("example.com".into(), "6982".into())))
/// if let Some((host, port)) = process_cloud_config() {
///     assert_eq!(host, "example.com");
///     assert_eq!(port, "6982");
/// } else {
///     panic!("expected cloud config");
/// }
/// ```
pub(crate) fn process_cloud_config() -> Option<(String, String)> {
    SaveData::get_cloud_config().unwrap_or_else(|_| None)
}
