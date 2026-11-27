use std::path::PathBuf;

use platform_dirs::AppDirs;
use yesser_todo_errors::db_error::DatabaseError;

pub(crate) fn get_app_dirs() -> Result<AppDirs, DatabaseError> {
    AppDirs::new(Some("todo"), true).ok_or(DatabaseError::UserDirsError)
}

pub(crate) fn get_data_dir(dirs: &AppDirs) -> PathBuf {
    dirs.data_dir.clone()
}

pub(crate) fn get_config_dir(dirs: &AppDirs) -> PathBuf {
    dirs.config_dir.clone()
}

#[cfg(test)]
mod tests {
    use platform_dirs::AppDirs;

    use crate::env::{get_app_dirs, get_config_dir, get_data_dir};

    #[test]
    fn get_app_dirs_gets_app_dirs() {
        let expected = AppDirs::new(Some("todo"), true).unwrap();

        let actual = get_app_dirs().unwrap();

        assert_eq!(expected.data_dir, actual.data_dir);
        assert_eq!(expected.config_dir, actual.config_dir);
        assert_eq!(expected.cache_dir, actual.cache_dir);
        assert_eq!(expected.state_dir, actual.state_dir);
    }

    #[test]
    fn get_data_dir_gets_data_dir() {
        let dirs = AppDirs::new(Some("todo-test"), true).unwrap();
        let expected = dirs.data_dir.clone();

        let actual = get_data_dir(&dirs);

        assert_eq!(expected, actual);
    }

    #[test]
    fn get_config_dir_gets_config_dir() {
        let dirs = AppDirs::new(Some("todo-test"), true).unwrap();
        let expected = dirs.config_dir.clone();

        let actual = get_config_dir(&dirs);

        assert_eq!(expected, actual);
    }
}
