use super::*;

#[test]
fn expands_unix_session_home_only_for_home_prefixes() {
    assert_eq!(
        expand_session_home("~", Some("/root"), &['/']),
        TypedPathBuf::from_unix("/root")
    );
    assert_eq!(
        expand_session_home("~/project/src", Some("/root"), &['/']),
        TypedPathBuf::from_unix("/root/project/src")
    );
    assert_eq!(
        expand_session_home("~user/project", Some("/root"), &['/']),
        TypedPathBuf::from_unix("~user/project")
    );
    assert_eq!(
        expand_session_home("~\\project", Some("/root"), &['/']),
        TypedPathBuf::from_unix("~\\project")
    );
}

#[test]
fn leaves_paths_without_session_home_unchanged() {
    assert_eq!(
        expand_session_home("~/project", None, &['/']),
        TypedPathBuf::from_unix("~/project")
    );
    assert_eq!(
        expand_session_home("/tmp/project", Some("/root"), &['/']),
        TypedPathBuf::from_unix("/tmp/project")
    );
}

#[test]
fn expands_windows_session_home_with_windows_separators() {
    assert_eq!(
        expand_session_home(r"~\Desktop", Some(r"C:\Users\user"), &['/', '\\']),
        TypedPathBuf::from_windows(r"C:\Users\user\Desktop")
    );
}

#[test]
fn test_user_friendly_path_with_home() {
    let home = "/Users/blue";
    assert_eq!(
        user_friendly_path("/Users/blue", Some(home)),
        "~".to_string(),
    );
    assert_eq!(
        user_friendly_path("/Users/blue/warp", Some(home)),
        "~/warp".to_string(),
    );
    assert_eq!(
        user_friendly_path("/Users/admin/warp", Some(home)),
        "/Users/admin/warp".to_string(),
    );
}

#[test]
fn test_to_relative_path() {
    use std::path::Path;

    use super::to_relative_path;

    // Basic relative path conversion
    {
        assert_eq!(
            to_relative_path(
                false,
                Path::new("/Users/john/projects/app/src/main.rs"),
                Path::new("/Users/john/projects")
            ),
            Some("app/src/main.rs".to_string())
        );

        // Same directory
        assert_eq!(
            to_relative_path(
                false,
                Path::new("/Users/john/projects"),
                Path::new("/Users/john/projects")
            ),
            Some(".".to_string())
        );

        // Parent directory
        assert_eq!(
            to_relative_path(
                false,
                Path::new("/Users/john"),
                Path::new("/Users/john/projects")
            ),
            Some("..".to_string())
        );

        // Nested parent
        assert_eq!(
            to_relative_path(
                false,
                Path::new("/Users"),
                Path::new("/Users/john/projects")
            ),
            Some("../..".to_string())
        );

        // Cross-branch paths
        assert_eq!(
            to_relative_path(
                false,
                Path::new("/Users/john/documents/file.txt"),
                Path::new("/Users/john/projects")
            ),
            Some("../documents/file.txt".to_string())
        );

        // Root to subdirectory
        assert_eq!(
            to_relative_path(false, Path::new("/var/log/system.log"), Path::new("/")),
            Some("var/log/system.log".to_string())
        );

        // Handles paths that would have leading slashes correctly
        assert_eq!(
            to_relative_path(false, Path::new("/home/user/file.txt"), Path::new("/home")),
            Some("user/file.txt".to_string())
        );

        // Test with current directory references that should be cleaned
        assert_eq!(
            to_relative_path(
                false,
                Path::new("/Users/john/projects/./app/src/main.rs"),
                Path::new("/Users/john/projects")
            ),
            Some("app/src/main.rs".to_string()),
        );
    }
}

#[test]
fn test_normalize_relative_path_for_glob() {
    use std::path::Path;

    assert_eq!(
        normalize_relative_path_for_glob(Path::new("app/src/main.rs")),
        "app/src/main.rs"
    );
    assert_eq!(
        normalize_relative_path_for_glob(Path::new("./app/src/main.rs")),
        "app/src/main.rs"
    );
    assert_eq!(
        normalize_relative_path_for_glob(Path::new("../app/src/main.rs")),
        "app/src/main.rs"
    );
    assert_eq!(normalize_relative_path_for_glob(Path::new("..")), "");
    assert_eq!(normalize_relative_path_for_glob(Path::new("")), "");
}

#[test]
fn test_posix_escape() {
    let shell_family = ShellFamily::Posix;
    assert_eq!(
        shell_family.escape("~/test_dir/library% 1$2"),
        "\\~/test_dir/library%\\ 1\\$2"
    );
    assert_eq!(shell_family.escape("あい"), "あい");
    assert_eq!(shell_family.escape("abc \n \t"), "abc\\ \\\n\\ \\\t");
    assert_eq!(shell_family.escape(""), "''");
    assert_eq!(
        shell_family.escape("foo '\"' bar"),
        "foo\\ \\'\\\"\\'\\ bar"
    );
}

#[test]
fn test_powershell_escape() {
    let shell_family = ShellFamily::PowerShell;
    assert_eq!(
        shell_family.escape("~/test_dir/library% 1$2"),
        "~/test_dir/library%` 1`$2"
    );
    assert_eq!(shell_family.escape("あい"), "あい");
    assert_eq!(shell_family.escape("abc \n \t"), "abc` `\n` `\t");
    assert_eq!(shell_family.escape(""), "''");
    assert_eq!(shell_family.escape("foo '\"' bar"), "foo` `'`\"`'` bar");
}

#[test]
fn test_posix_unescape() {
    let shell_family = ShellFamily::Posix;
    // Escaped spaces
    assert_eq!(shell_family.unescape("my\\ file.txt"), "my file.txt");
    // Multiple escaped characters
    assert_eq!(
        shell_family.unescape("path/to/my\\ file\\ \\(1\\).txt"),
        "path/to/my file (1).txt"
    );
    // No escaping needed — returns borrowed
    assert!(matches!(
        shell_family.unescape("simple.txt"),
        std::borrow::Cow::Borrowed(_)
    ));
    // Trailing backslash kept as-is
    assert_eq!(shell_family.unescape("trailing\\"), "trailing\\");
    // Roundtrip: unescape(escape(x)) == x
    let original = "hello world $HOME 'quotes'";
    assert_eq!(
        shell_family.unescape(&shell_family.escape(original)),
        original
    );
}

#[test]
fn test_powershell_unescape() {
    let shell_family = ShellFamily::PowerShell;
    // Escaped spaces
    assert_eq!(shell_family.unescape("my` file.txt"), "my file.txt");
    // Multiple escaped characters
    assert_eq!(shell_family.unescape("path` `$var"), "path $var");
    // No escaping needed — returns borrowed
    assert!(matches!(
        shell_family.unescape("simple.txt"),
        std::borrow::Cow::Borrowed(_)
    ));
    // Roundtrip: unescape(escape(x)) == x
    let original = "hello world $HOME";
    assert_eq!(
        shell_family.unescape(&shell_family.escape(original)),
        original
    );
}

#[test]
fn test_clean_path() {
    assert_eq!(
        CleanPathResult::with_line_and_column_number("Cargo.toml:10:5"),
        CleanPathResult {
            path: "Cargo.toml".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 10,
                column_num: Some(5)
            }),
        }
    );

    assert_eq!(
        CleanPathResult::with_line_and_column_number("Cargo.toml:30:5abc"),
        CleanPathResult {
            path: "Cargo.toml:30:5abc".into(),
            line_and_column_num: None
        }
    );

    assert_eq!(
        CleanPathResult::with_line_and_column_number("Cargo.toml[30,5]"),
        CleanPathResult {
            path: "Cargo.toml".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 30,
                column_num: Some(5)
            })
        }
    );

    assert_eq!(
        CleanPathResult::with_line_and_column_number("Cargo.toml(3,1)"),
        CleanPathResult {
            path: "Cargo.toml".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 3,
                column_num: Some(1)
            })
        }
    );

    assert_eq!(
        CleanPathResult::with_line_and_column_number("Cargo.toml\", line 100, in"),
        CleanPathResult {
            path: "Cargo.toml".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 100,
                column_num: None,
            })
        }
    );

    assert_eq!(
        CleanPathResult::with_line_and_column_number("Cargo.toml\", line 5, column 20"),
        CleanPathResult {
            path: "Cargo.toml".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 5,
                column_num: Some(20),
            })
        }
    );

    assert_eq!(
        CleanPathResult::with_line_and_column_number("Cargo.toml#L100"),
        CleanPathResult {
            path: "Cargo.toml".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 100,
                column_num: None
            })
        }
    );

    assert_eq!(
        CleanPathResult::with_line_and_column_number("Cargo.toml#L100:4"),
        CleanPathResult {
            path: "Cargo.toml".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 100,
                column_num: Some(4)
            })
        }
    );

    // Line range format :start-end (should link to start line, ignore end line)
    assert_eq!(
        CleanPathResult::with_line_and_column_number("Cargo.toml:10-50"),
        CleanPathResult {
            path: "Cargo.toml".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 10,
                column_num: None
            })
        }
    );

    assert_eq!(
        CleanPathResult::with_line_and_column_number("/path/to/file.rs:1-1000"),
        CleanPathResult {
            path: "/path/to/file.rs".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 1,
                column_num: None
            })
        }
    );

    assert_eq!(
        CleanPathResult::with_line_and_column_number("src/main.rs:100-100"),
        CleanPathResult {
            path: "src/main.rs".into(),
            line_and_column_num: Some(LineAndColumnArg {
                line_num: 100,
                column_num: None
            })
        }
    );
}

/// The single predicate every WSL UNC host check in this module goes through, so its casing and
/// host-spelling table is asserted here rather than repeated at each call site.
#[test]
fn test_is_wsl_unc_host() {
    for host in [
        "wsl$",
        "WSL$",
        "Wsl$",
        "wsl.localhost",
        "WSL.localhost",
        "Wsl.Localhost",
    ] {
        assert!(
            is_wsl_unc_host(host),
            "{host} should be recognized as a WSL UNC host"
        );
    }

    for host in [
        "",
        "wsl",
        "wsl$$",
        "wsl.localhost.example.com",
        "localhost",
        "server",
    ] {
        assert!(
            !is_wsl_unc_host(host),
            "{host} should not be recognized as a WSL UNC host"
        );
    }
}

/// Covers the prefix-kind dispatch — only UNC and verbatim UNC prefixes carry a host to check.
#[test]
fn test_is_wsl_unc_prefix() {
    fn is_wsl_unc_path_prefix(path: &str) -> bool {
        let typed_path = TypedPath::from(path);
        match typed_path.components().next() {
            Some(TypedComponent::Windows(WindowsComponent::Prefix(prefix))) => {
                is_wsl_unc_prefix(&prefix)
            }
            _ => panic!("{path} should be inferred as a Windows path with a prefix"),
        }
    }

    for path in [
        "//wsl$/Ubuntu/home",
        "//WSL.localhost/Ubuntu/home",
        r"\\wsl$\Ubuntu\home",
        r"\\?\UNC\WSL.localhost\Ubuntu\home",
    ] {
        assert!(
            is_wsl_unc_path_prefix(path),
            "{path} should be recognized as a WSL UNC prefix"
        );
    }

    for path in [
        r"\\server\share\file",
        r"\\?\UNC\server\share",
        r"\\.\COM42",
        r"\\?\pictures",
        r"C:\Users",
        r"\\?\C:\Users",
    ] {
        assert!(
            !is_wsl_unc_path_prefix(path),
            "{path} should not be recognized as a WSL UNC prefix"
        );
    }
}

/// Asserts that `input` parses into the given distribution and Linux path.
fn assert_parses_wsl_unc(input: &str, distro: &str, linux_path: &str) {
    assert_eq!(
        parse_wsl_unc_path(Path::new(input)),
        Some(WslUncPath {
            distro: distro.to_string(),
            linux_path: linux_path.to_string(),
        }),
        "input: {input:?}"
    );
}

#[test]
fn test_parse_wsl_unc_path() {
    assert_parses_wsl_unc(r"\\wsl$\Ubuntu\home\user", "Ubuntu", "/home/user");
    assert_parses_wsl_unc(r"\\wsl.localhost\Ubuntu\home", "Ubuntu", "/home");
    assert_parses_wsl_unc(r"\\?\UNC\wsl$\Ubuntu\home\user", "Ubuntu", "/home/user");
    assert_parses_wsl_unc(
        r"\\?\UNC\wsl.localhost\Ubuntu\srv\repo",
        "Ubuntu",
        "/srv/repo",
    );
    assert_parses_wsl_unc("//wsl$/Ubuntu/home/user", "Ubuntu", "/home/user");
    // The host is matched case-insensitively.
    assert_parses_wsl_unc(r"\\WSL$\Ubuntu\src", "Ubuntu", "/src");
    assert_parses_wsl_unc(r"\\Wsl.LocalHost\Ubuntu\src", "Ubuntu", "/src");
    // Trailing separators are dropped.
    assert_parses_wsl_unc(r"\\wsl$\Ubuntu\home\user\", "Ubuntu", "/home/user");
    assert_parses_wsl_unc("//wsl$/Ubuntu/home/user/", "Ubuntu", "/home/user");
    // Distribution names may contain dots and dashes.
    assert_parses_wsl_unc(
        r"\\wsl.localhost\Ubuntu-24.04\home\krag",
        "Ubuntu-24.04",
        "/home/krag",
    );
    // The distribution root maps to `/`.
    assert_parses_wsl_unc(r"\\wsl$\Ubuntu", "Ubuntu", "/");
    assert_parses_wsl_unc(r"\\wsl$\Ubuntu\", "Ubuntu", "/");
    assert_parses_wsl_unc("//wsl$/archlinux", "archlinux", "/");
}

#[test]
fn test_parse_wsl_unc_path_rejects_other_paths() {
    // Non-WSL UNC share.
    assert_eq!(parse_wsl_unc_path(Path::new(r"\\server\share\dir")), None);
    // Drive-letter path.
    assert_eq!(parse_wsl_unc_path(Path::new(r"C:\Users\foo")), None);
    // Relative paths.
    assert_eq!(parse_wsl_unc_path(Path::new(r"foo\bar")), None);
    assert_eq!(parse_wsl_unc_path(Path::new("foo/bar")), None);
    // WSL host without a distribution name.
    assert_eq!(parse_wsl_unc_path(Path::new(r"\\wsl$")), None);
    assert_eq!(parse_wsl_unc_path(Path::new(r"\\wsl$\")), None);
}

#[test]
fn test_convert_windows_path_to_wsl() {
    assert_eq!(
        convert_windows_path_to_wsl(r"C:\Users\aloke\file.txt"),
        "/mnt/c/Users/aloke/file.txt"
    );
    assert_eq!(
        convert_windows_path_to_wsl(r"D:\Pictures\Screenshots\Screenshot 2025-05-14 155816.png"),
        "/mnt/d/Pictures/Screenshots/Screenshot 2025-05-14 155816.png"
    );
    // Drive letter only
    assert_eq!(convert_windows_path_to_wsl(r"C:\"), "/mnt/c");
    assert_eq!(convert_windows_path_to_wsl("C:"), "/mnt/c");
    // Uppercase drive letter should be lowercased
    assert_eq!(convert_windows_path_to_wsl(r"E:\foo"), "/mnt/e/foo");
    // Non-drive path (e.g. UNC) gets backslashes replaced
    assert_eq!(
        convert_windows_path_to_wsl(r"\\server\share\file"),
        "//server/share/file"
    );
}

#[test]
fn test_convert_windows_path_to_msys2() {
    assert_eq!(
        convert_windows_path_to_msys2(r"C:\Users\aloke\file.txt"),
        "/c/Users/aloke/file.txt"
    );
    assert_eq!(
        convert_windows_path_to_msys2(r"D:\Pictures\Screenshots\Screenshot 2025-05-14 155816.png"),
        "/d/Pictures/Screenshots/Screenshot 2025-05-14 155816.png"
    );
    // Drive letter only
    assert_eq!(convert_windows_path_to_msys2(r"C:\"), "/c");
    assert_eq!(convert_windows_path_to_msys2("C:"), "/c");
    // Uppercase drive letter should be lowercased
    assert_eq!(convert_windows_path_to_msys2(r"E:\foo"), "/e/foo");
    // Non-drive path (e.g. UNC) gets backslashes replaced
    assert_eq!(
        convert_windows_path_to_msys2(r"\\server\share\file"),
        "//server/share/file"
    );
}

#[test]
fn test_file_uri_drive_path_to_windows() {
    assert_eq!(
        file_uri_drive_path_to_windows("/E:/CLAUDE-BASE"),
        r"E:\CLAUDE-BASE"
    );
    assert_eq!(
        file_uri_drive_path_to_windows("/C:/Users/foo/bar"),
        r"C:\Users\foo\bar"
    );
    assert_eq!(file_uri_drive_path_to_windows("/E:/"), r"E:\");
    assert_eq!(file_uri_drive_path_to_windows("/E:"), "E:");
    assert_eq!(
        file_uri_drive_path_to_windows("/Users/foo/bar"),
        "/Users/foo/bar"
    );
    assert_eq!(file_uri_drive_path_to_windows("/E/notdrive"), "/E/notdrive");
    assert_eq!(file_uri_drive_path_to_windows("/E:extra"), "/E:extra");
    assert_eq!(
        file_uri_drive_path_to_windows(r"E:\CLAUDE-BASE"),
        r"E:\CLAUDE-BASE"
    );
}

#[test]
fn test_canonicalize_git_bash_path() {
    assert_eq!(
        canonicalize_git_bash_path(
            Path::new("C:")
                .join("Program Files")
                .join("Git")
                .join("bin")
                .join("bash.exe")
        ),
        Path::new("C:")
            .join("Program Files")
            .join("Git")
            .join("usr")
            .join("bin")
            .join("bash.exe")
    );
    assert_eq!(
        canonicalize_git_bash_path(
            Path::new("C:")
                .join("Windows")
                .join("system32")
                .join("bash.exe")
        ),
        Path::new("C:")
            .join("Windows")
            .join("system32")
            .join("bash.exe")
    );
}

// ── group_roots_by_common_ancestor tests ─────────────────────────────

mod group_roots_by_common_ancestor_tests {
    use std::path::PathBuf;

    use crate::path::group_roots_by_common_ancestor;

    fn pb(s: &str) -> PathBuf {
        PathBuf::from(s)
    }

    #[test]
    fn empty_input_produces_empty_grouping() {
        let grouping = group_roots_by_common_ancestor::<PathBuf>(&[]);
        assert!(grouping.roots.is_empty());
        assert!(grouping.absorbed_by_root.is_empty());
    }

    #[test]
    fn single_path_survives_with_no_absorbed() {
        let grouping = group_roots_by_common_ancestor(&[pb("/a")]);
        assert_eq!(grouping.roots, vec![pb("/a")]);
        assert!(grouping.absorbed_by_root.is_empty());
    }

    #[test]
    fn unrelated_siblings_both_survive() {
        let grouping = group_roots_by_common_ancestor(&[pb("/a"), pb("/b")]);
        assert_eq!(grouping.roots, vec![pb("/a"), pb("/b")]);
        assert!(grouping.absorbed_by_root.is_empty());
    }

    #[test]
    fn descendant_absorbed_into_ancestor() {
        // Ancestor listed first.
        let grouping = group_roots_by_common_ancestor(&[pb("/a"), pb("/a/b")]);
        assert_eq!(grouping.roots, vec![pb("/a")]);
        assert_eq!(grouping.absorbed_by_root.len(), 1);
        assert_eq!(grouping.absorbed_by_root[&pb("/a")], vec![pb("/a/b")]);
    }

    #[test]
    fn descendant_first_still_absorbed() {
        // Descendant listed first, ancestor second; survivor is still the
        // ancestor and its input order is preserved.
        let grouping = group_roots_by_common_ancestor(&[pb("/a/b"), pb("/a")]);
        assert_eq!(grouping.roots, vec![pb("/a")]);
        assert_eq!(grouping.absorbed_by_root[&pb("/a")], vec![pb("/a/b")]);
    }

    #[test]
    fn three_deep_chain_collapses_to_root() {
        // Descendant order in input is preserved in the absorbed list.
        let grouping = group_roots_by_common_ancestor(&[pb("/a/b/c"), pb("/a/b"), pb("/a")]);
        assert_eq!(grouping.roots, vec![pb("/a")]);
        assert_eq!(
            grouping.absorbed_by_root[&pb("/a")],
            vec![pb("/a/b/c"), pb("/a/b")]
        );
    }

    #[test]
    fn mixed_groups_absorb_independently() {
        let grouping =
            group_roots_by_common_ancestor(&[pb("/a"), pb("/x"), pb("/a/b"), pb("/x/y")]);
        assert_eq!(grouping.roots, vec![pb("/a"), pb("/x")]);
        assert_eq!(grouping.absorbed_by_root[&pb("/a")], vec![pb("/a/b")]);
        assert_eq!(grouping.absorbed_by_root[&pb("/x")], vec![pb("/x/y")]);
    }

    #[test]
    fn same_prefix_different_component_name_both_survive() {
        // /foo/a is NOT an ancestor of /foo/abc (component-aware match).
        let grouping = group_roots_by_common_ancestor(&[pb("/foo/a"), pb("/foo/abc")]);
        assert_eq!(grouping.roots, vec![pb("/foo/a"), pb("/foo/abc")]);
        assert!(grouping.absorbed_by_root.is_empty());
    }

    #[test]
    fn duplicate_inputs_collapse_to_single_survivor() {
        let grouping = group_roots_by_common_ancestor(&[pb("/a"), pb("/a")]);
        assert_eq!(grouping.roots, vec![pb("/a")]);
        assert!(grouping.absorbed_by_root.is_empty());
    }

    #[test]
    fn surviving_root_order_matches_input_order() {
        // Insert a descendant between two surviving ancestors; the survivors
        // should appear in their original input order even though processing
        // sorted by component count.
        let grouping = group_roots_by_common_ancestor(&[pb("/b"), pb("/a/x"), pb("/a"), pb("/c")]);
        assert_eq!(grouping.roots, vec![pb("/b"), pb("/a"), pb("/c")]);
        assert_eq!(grouping.absorbed_by_root[&pb("/a")], vec![pb("/a/x")]);
    }

    #[test]
    fn descendant_absorbed_by_closest_ancestor_not_furthest() {
        // Both /a and /a/b are surviving ancestors of /a/b/c... wait, /a/b is
        // itself absorbed into /a. So /a/b/c should be absorbed into /a as
        // well (the only surviving ancestor).
        let grouping = group_roots_by_common_ancestor(&[pb("/a"), pb("/a/b"), pb("/a/b/c")]);
        assert_eq!(grouping.roots, vec![pb("/a")]);
        assert_eq!(
            grouping.absorbed_by_root[&pb("/a")],
            vec![pb("/a/b"), pb("/a/b/c")]
        );
    }
}
#[test]
fn test_resolve_command() {
    use std::path::Path;

    use crate::path::resolve_executable;

    assert_eq!(
        &resolve_executable("/bin/sh").unwrap(),
        Path::new("/bin/sh")
    );
    assert_eq!(
        &resolve_executable("env").unwrap(),
        Path::new("/usr/bin/env")
    );
    // This path exists in the Warp repo, so it should resolve. The `../../`
    // is because Rust unit tests run from the root of the crate (`warp_util` in
    // this case).
    assert_eq!(
        &resolve_executable("../../script/run").unwrap(),
        Path::new("../../script/run")
    );
    // `pwd` should always exist (it's also a shell builtin), but we won't
    // assume a specific location.
    assert!(resolve_executable("pwd").is_some());
    assert!(resolve_executable("unlikely-command").is_none());
    assert!(resolve_executable("nonexistent/relative/path").is_none());
    // src/main.rs does exist, but is not executable.
    assert!(resolve_executable("src/main.rs").is_none());
    // Note the trailing space.
    assert!(resolve_executable("zsh ").is_none());
}
