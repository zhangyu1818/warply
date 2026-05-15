use dirs::home_dir;

use super::*;

#[test]
fn test_data_dir_path() {
    let home_dir = home_dir().expect("Should be able to compute home directory");
    assert_eq!(data_dir(), home_dir.join(".warply"));
}

#[test]
fn test_config_local_dir_path() {
    let home_dir = home_dir().expect("Should be able to compute home directory");
    assert_eq!(config_local_dir(), home_dir.join(".warply"));
}

#[test]
fn test_warp_home_config_dir_path() {
    let home_dir = home_dir().expect("Should be able to compute home directory");
    let expected_dir_name = match ChannelState::data_profile() {
        Some(data_profile) => format!(".warply-{data_profile}"),
        None => ".warply".to_string(),
    };

    assert_eq!(
        warp_home_config_dir(),
        Some(home_dir.join(expected_dir_name))
    );
}

#[test]
fn test_cache_dir_path() {
    let home_dir = home_dir().expect("Should be able to compute home directory");
    assert_eq!(
        cache_dir(),
        home_dir.join("Library/Application Support/dev.zhangyu1818.warply")
    );
}

#[test]
fn test_state_dir_path() {
    let home_dir = home_dir().expect("Should be able to compute home directory");
    assert_eq!(
        state_dir(),
        home_dir.join("Library/Application Support/dev.zhangyu1818.warply")
    );
}

#[test]
fn test_project_path_for_warp_app_id() {
    let project_dirs = project_dirs_for_app_id(AppId::new("dev", "zhangyu1818", "warply"), None)
        .expect("should be able to compute project dirs");
    assert_eq!(project_dirs.project_path(), "dev.zhangyu1818.warply");
}

#[test]
fn test_project_path_for_warp_dev_app_id() {
    let project_dirs =
        project_dirs_for_app_id(AppId::new("dev", "zhangyu1818", "warply-dev"), None)
            .expect("should be able to compute project dirs");
    assert_eq!(project_dirs.project_path(), "dev.zhangyu1818.warply-dev");
}

#[test]
fn test_project_path_for_oss_app_id() {
    let project_dirs = project_dirs_for_app_id(AppId::new("dev", "zhangyu1818", "warply"), None)
        .expect("should be able to compute project dirs");
    assert_eq!(project_dirs.project_path(), "dev.zhangyu1818.warply");
}
