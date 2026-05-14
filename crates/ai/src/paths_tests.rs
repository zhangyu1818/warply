use super::*;

#[test]
fn test_host_native_absolute_path() {
    assert_eq!(
        host_native_absolute_path(
            "/home/user/file.txt",
            &None,
            &Some("/current/dir".to_string())
        ),
        "/home/user/file.txt"
    );

    assert_eq!(
        host_native_absolute_path("file.txt", &None, &Some("/current/dir".to_string())),
        "/current/dir/file.txt"
    );

    assert_eq!(
        host_native_absolute_path("~/file.txt", &None, &Some("/current/dir".to_string())),
        shellexpand::tilde("~/file.txt").into_owned()
    );

    assert_eq!(
        host_native_absolute_path("../user/file.txt", &None, &Some("/current/dir".to_string())),
        "/current/user/file.txt"
    );

    assert_eq!(
        host_native_absolute_path("./user/file.txt", &None, &Some("/current/dir".to_string())),
        "/current/dir/user/file.txt"
    );

    assert_eq!(
        host_native_absolute_path("file.txt", &None, &None),
        "file.txt"
    );

    assert_eq!(
        host_native_absolute_path("file.txt", &None, &Some("".to_string())),
        "file.txt"
    );
}

#[test]
fn test_shell_native_absolute_path() {
    let cwd = Some("/current/dir".to_string());
    assert_eq!(
        shell_native_absolute_path("/home/user/file.txt", None, cwd.as_ref()),
        "/home/user/file.txt"
    );

    assert_eq!(
        shell_native_absolute_path("file.txt", None, cwd.as_ref()),
        "/current/dir/file.txt"
    );

    assert_eq!(
        shell_native_absolute_path("~/file.txt", None, cwd.as_ref()),
        shellexpand::tilde("~/file.txt").into_owned()
    );

    assert_eq!(
        shell_native_absolute_path("../user/file.txt", None, cwd.as_ref()),
        "/current/user/file.txt"
    );

    assert_eq!(
        shell_native_absolute_path("./user/file.txt", None, cwd.as_ref()),
        "/current/dir/user/file.txt"
    );

    assert_eq!(
        shell_native_absolute_path("file.txt", None, None),
        "file.txt"
    );

    let empty_cwd = Some("".to_string());
    assert_eq!(
        shell_native_absolute_path("file.txt", None, empty_cwd.as_ref()),
        "file.txt"
    );
}
