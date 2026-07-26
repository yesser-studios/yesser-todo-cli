use yansi::{Color::Green, Style};
use yesser_todo_api::Client;
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
/// ```text
/// // Suppose get_cloud_config() returns Ok(Some(("example.com".into(), "6982".into())))
/// // Then process_cloud_config would return Some(("example.com", "6982")).
/// ```
pub(crate) fn process_cloud_config(args: Option<&TodoArgs>, data: &dyn SaveData) -> Option<(String, String)> {
    if let Some(args) = args
        && args.local
    {
        None
    } else {
        data.get_cloud_config().unwrap_or_else(|err| {
            eprintln!("Warning: Failed to read cloud config: {err}. Proceeding with local mode.");
            None
        })
    }
}

pub(crate) fn get_client(args: Option<&TodoArgs>, data: &dyn SaveData) -> Option<Client> {
    process_cloud_config(args, data).map(|(hostname, port)| Client::new(hostname, Some(port)))
}

#[cfg(test)]
mod tests {
    use yansi::Paint;
    use yesser_todo_db::JsonSaveData;

    use super::*;

    #[test]
    fn test_done_style_exists() {
        let styled = "test".paint(DONE_STYLE);
        let styled_str = format!("{}", styled);
        assert!(styled_str.contains("test"));
    }

    fn construct_todo_args(local: bool) -> TodoArgs {
        TodoArgs {
            command: crate::args::Command::Add(crate::args::TasksCommand { tasks: vec!["".to_string()] }),
            local,
        }
    }

    fn make_data_with_cloud_config(host: &str, port: &str) -> (JsonSaveData, tempfile::TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let mut data = JsonSaveData::with_dir(dir.path().to_owned());
        data.save_cloud_config(host, port).unwrap();
        (data, dir)
    }

    #[test]
    fn test_process_cloud_config_returns_option_with_data() {
        let (data, _dir) = make_data_with_cloud_config("http://127.0.0.1", "6982");
        let result = process_cloud_config(None, &data);
        assert!(result.is_some());
    }

    #[test]
    fn test_process_cloud_config_tuple_structure() {
        let (data, _dir) = make_data_with_cloud_config("http://127.0.0.1", "6982");
        if let Some((host, port)) = process_cloud_config(None, &data) {
            assert!(!host.is_empty());
            assert!(!port.is_empty());
        }
    }

    #[test]
    fn test_process_cloud_config_args_is_some_local_false() {
        const HOST: &str = "http://127.0.0.1";
        const PORT: &str = yesser_todo_api::DEFAULT_PORT;

        let (data, _dir) = make_data_with_cloud_config(HOST, PORT);
        let result = process_cloud_config(Some(&construct_todo_args(false)), &data);

        let (host, port) = result.unwrap();
        assert_eq!(host, HOST);
        assert_eq!(port, PORT);
    }

    #[test]
    fn test_process_cloud_config_args_is_some_local_true() {
        const HOST: &str = "http://127.0.0.1";
        const PORT: &str = yesser_todo_api::DEFAULT_PORT;

        let (data, _dir) = make_data_with_cloud_config(HOST, PORT);
        let result = process_cloud_config(Some(&construct_todo_args(true)), &data);

        assert!(result.is_none());
    }

    #[test]
    fn test_process_cloud_config_args_is_none() {
        const HOST: &str = "http://127.0.0.1";
        const PORT: &str = yesser_todo_api::DEFAULT_PORT;

        let (data, _dir) = make_data_with_cloud_config(HOST, PORT);
        let result = process_cloud_config(None, &data);

        let (host, port) = result.unwrap();
        assert_eq!(host, HOST);
        assert_eq!(port, PORT);
    }

    #[test]
    fn test_process_cloud_config_no_config() {
        let dir = tempfile::tempdir().unwrap();
        let data = JsonSaveData::with_dir(dir.path().to_owned());
        let result = process_cloud_config(Some(&construct_todo_args(false)), &data);
        assert!(result.is_none());
    }

    #[test]
    fn test_done_style_properties() {
        let test_text = "completed task";
        let styled = test_text.paint(DONE_STYLE);
        let output = format!("{}", styled);
        assert!(output.len() >= test_text.len());
    }
}
