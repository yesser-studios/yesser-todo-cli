use console::Style;
use yesser_todo_db::SaveData;

pub(crate) const DONE_STYLE: Style = Style::new().strikethrough().green();

/// Return the saved cloud server host and port when available.
///
/// # Returns
///
/// `Some((host, port))` if a readable cloud configuration exists, `None` otherwise.
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
    SaveData::get_cloud_config().unwrap_or_else(|err| {
        eprintln!("Warning: Failed to read cloud config: {err}. Proceeding with local mode.");
        None
    })
}
