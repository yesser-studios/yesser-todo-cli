use std::{env, path::PathBuf, str::FromStr};

use platform_dirs::AppDirs;
use yesser_todo_errors::db_error::DatabaseError;

pub(crate) fn get_app_dirs() -> Result<AppDirs, DatabaseError> {
    AppDirs::new(Some("todo"), true).ok_or(DatabaseError::UserDirsError)
}

pub(crate) fn get_data_dir(dirs: &AppDirs) -> PathBuf {
    if let Ok(data_dir) = env::var("TODO_DATA_DIR")
        && let Ok(path) = PathBuf::from_str(data_dir.as_str())
    {
        path
    } else {
        dirs.data_dir.clone()
    }
}

pub(crate) fn get_config_dir(dirs: &AppDirs) -> PathBuf {
    if let Ok(config_dir) = env::var("TODO_CONFIG_DIR")
        && let Ok(path) = PathBuf::from_str(config_dir.as_str())
    {
        path
    } else {
        dirs.config_dir.clone()
    }
}

#[cfg(test)]
mod tests {
    use std::{env, path::PathBuf, str::FromStr};

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

        unsafe {
            env::remove_var("TODO_DATA_DIR");
        }
        let actual = get_data_dir(&dirs);

        assert_eq!(expected, actual);
    }
    #[test]
    fn get_data_dir_gets_envvar() {
        let dirs = AppDirs::new(Some("todo-test"), true).unwrap();
        let expected = PathBuf::from_str("/todo/data").unwrap();

        unsafe {
            env::set_var("TODO_DATA_DIR", expected.to_str().unwrap());
        }
        let actual = get_data_dir(&dirs);

        assert_eq!(expected, actual)
    }

    #[test]
    fn get_config_dir_gets_config_dir() {
        let dirs = AppDirs::new(Some("todo-test"), true).unwrap();
        let expected = dirs.config_dir.clone();

        unsafe {
            env::remove_var("TODO_CONFIG_DIR");
        }
        let actual = get_config_dir(&dirs);

        assert_eq!(expected, actual);
    }
    #[test]
    fn get_config_dir_gets_envvar() {
        let dirs = AppDirs::new(Some("todo-test"), true).unwrap();
        let expected = PathBuf::from_str("/todo/config").unwrap();

        unsafe {
            env::set_var("TODO_CONFIG_DIR", expected.to_str().unwrap());
        }
        let actual = get_config_dir(&dirs);

        assert_eq!(expected, actual)
    }
}
