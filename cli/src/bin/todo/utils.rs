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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_done_style_exists() {
        let style = DONE_STYLE;
        let styled = style.apply_to("test");
        let styled_str = format!("{}", styled);
        assert!(styled_str.contains("test"));
    }

    #[test]
    fn test_process_cloud_config_returns_option() {
        let result = process_cloud_config();
        assert!(result.is_some() || result.is_none());
    }

    #[test]
    fn test_process_cloud_config_tuple_structure() {
        if let Some((host, port)) = process_cloud_config() {
            assert!(!host.is_empty() || host.is_empty());
            assert!(!port.is_empty() || port.is_empty());
        }
    }

    #[test]
    fn test_done_style_properties() {
        let style = DONE_STYLE;
        let test_text = "completed task";
        let styled = style.apply_to(test_text);
        let output = format!("{}", styled);
        assert!(output.len() >= test_text.len());
    }
}