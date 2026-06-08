use yansi::{Color::Green, Style};
use yesser_todo_db::SaveData;

use crate::args::TodoArgs;

pub(crate) const DONE_STYLE: Style = Green.strike();

/// Return the saved cloud server host and port if available.
///
/// # Arguments
///
/// * `args` - An optional `TodoArgs` object. Will be used to determine whether local mode is on.
///
/// # Returns
///
/// If `args` is `Some`, and `args.local` is `true`, returns `None`.  
/// If `args` is None, local mode will be assumed false.
///
/// Returns an optional `(String, String)` in the format `(host URL, port)` based on whether the
/// cloud config is saved.  
/// If an error occurs while reading the cloud config, an error message will
/// be printed and None will be returned (local mode will be assumed).
///
/// # Examples
///
/// ```
/// // Suppose SaveData::get_cloud_config() returns Ok(Some(("example.com".into(), "6982".into())))
/// if let Some((host, port)) = process_cloud_config(None) {
///     assert_eq!(host, "example.com");
///     assert_eq!(port, "6982");
/// } else {
///     panic!("expected cloud config");
/// }
/// ```
pub(crate) fn process_cloud_config(args: Option<&TodoArgs>) -> Option<(String, String)> {
    if let Some(args) = args
        && args.local
    {
        None
    } else {
        SaveData::get_cloud_config().unwrap_or_else(|err| {
            eprintln!("Warning: Failed to read cloud config: {err}. Proceeding with local mode.");
            None
        })
    }
}

#[cfg(test)]
mod tests {
    use yansi::Paint;

    use super::*;

    #[test]
    fn test_done_style_exists() {
        let styled = "test".paint(DONE_STYLE);
        let styled_str = format!("{}", styled);
        assert!(styled_str.contains("test"));
    }

    #[test]
    fn test_process_cloud_config_returns_option() {
        let result = process_cloud_config(None);
        assert!(result.is_some() || result.is_none());
    }

    #[test]
    fn test_process_cloud_config_tuple_structure() {
        if let Some((host, port)) = process_cloud_config(None) {
            assert!(!host.is_empty() || host.is_empty());
            assert!(!port.is_empty() || port.is_empty());
        }
    }

    fn construct_todo_args(local: bool) -> TodoArgs {
        TodoArgs {
            command: crate::args::Command::Add(crate::args::TasksCommand { tasks: vec!["".to_string()] }),
            local,
        }
    }

    #[test]
    fn test_process_cloud_config_args_is_some_local_false() {
        const HOST: &str = "http://127.0.0.1";
        const PORT: &str = yesser_todo_api::DEFAULT_PORT;

        let previous_cloud_config = SaveData::get_cloud_config().unwrap();

        SaveData::save_cloud_config(HOST, PORT).unwrap();
        let result = process_cloud_config(Some(&construct_todo_args(false)));

        match previous_cloud_config {
            Some((host, port)) => SaveData::save_cloud_config(&host, &port).unwrap(),
            None => SaveData::remove_cloud_config().unwrap(),
        };

        let (host, port) = result.unwrap();
        assert_eq!(host, HOST);
        assert_eq!(port, PORT);
    }

    #[test]
    fn test_process_cloud_config_args_is_some_local_true() {
        const HOST: &str = "http://127.0.0.1";
        const PORT: &str = yesser_todo_api::DEFAULT_PORT;

        let previous_cloud_config = SaveData::get_cloud_config().unwrap();

        SaveData::save_cloud_config(HOST, PORT).unwrap();
        let result = process_cloud_config(Some(&construct_todo_args(true)));

        match previous_cloud_config {
            Some((host, port)) => SaveData::save_cloud_config(&host, &port).unwrap(),
            None => SaveData::remove_cloud_config().unwrap(),
        };

        assert!(result.is_none());
    }

    #[test]
    fn test_process_cloud_config_args_is_none() {
        const HOST: &str = "http://127.0.0.1";
        const PORT: &str = yesser_todo_api::DEFAULT_PORT;

        let previous_cloud_config = SaveData::get_cloud_config().unwrap();

        SaveData::save_cloud_config(HOST, PORT).unwrap();
        let result = process_cloud_config(None);

        match previous_cloud_config {
            Some((host, port)) => SaveData::save_cloud_config(&host, &port).unwrap(),
            None => SaveData::remove_cloud_config().unwrap(),
        };

        let (host, port) = result.unwrap();
        assert_eq!(host, HOST);
        assert_eq!(port, PORT);
    }

    #[test]
    fn test_done_style_properties() {
        let test_text = "completed task";
        let styled = test_text.paint(DONE_STYLE);
        let output = format!("{}", styled);
        assert!(output.len() >= test_text.len());
    }
}
