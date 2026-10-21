use super::*;

/// What `paths` type for `quote`.
fn typed_as(quote: Quote, all: &[&str]) -> String {
    let paths: Vec<PathBuf> = all.iter().map(PathBuf::from).collect();
    typed(&paths, quote).unwrap()
}

#[test]
fn a_posix_path_escapes_what_a_shell_reads_specially() {
    let typed = |all: &[&str]| typed_as(Quote::Posix, all);
    assert_eq!(typed(&["/tmp/a.txt"]), "/tmp/a.txt ");
    assert_eq!(typed(&["/tmp/my file.png"]), "/tmp/my\\ file.png ");
    assert_eq!(typed(&["/a/it's (1)&$x"]), "/a/it\\'s\\ \\(1\\)\\&\\$x ");
    assert_eq!(
        typed(&["/a/a^b~c#d"]),
        "/a/a\\^b\\~c\\#d ",
        "zsh's extendedglob"
    );
    assert_eq!(typed(&["/a/中文 b"]), "/a/中文\\ b ");
    assert_eq!(
        typed(&["/a", "/b c"]),
        "/a /b\\ c ",
        "each followed by a space"
    );
}

#[test]
fn a_powershell_path_is_single_quoted_with_its_quotes_doubled() {
    let typed = |all: &[&str]| typed_as(Quote::PowerShell, all);
    assert_eq!(typed(&["C:\\a.txt"]), "'C:\\a.txt' ");
    assert_eq!(typed(&["C:\\tmp\\it's.txt"]), "'C:\\tmp\\it''s.txt' ");
    assert_eq!(typed(&["C:\\$HOME $(1+1);x"]), "'C:\\$HOME $(1+1);x' ");
    assert_eq!(
        typed(&["C:\\a\u{2019}b"]),
        "'C:\\a\u{2019}\u{2019}b' ",
        "a curly quote"
    );
    assert_eq!(typed(&["/a b", "/c"]), "'/a b' '/c' ");
}

#[test]
fn a_cmd_path_is_double_quoted_its_trailing_backslashes_doubled_and_one_cmd_expands_types_nothing()
{
    let typed = |all: &[&str]| typed_as(Quote::Cmd, all);
    assert_eq!(
        typed(&["C:\\My Files\\a&b^c.txt", "D:\\c"]),
        "\"C:\\My Files\\a&b^c.txt\" \"D:\\c\" "
    );
    assert_eq!(
        typed(&["C:\\", "D:\\clip.mp4"]),
        "\"C:\\\\\" \"D:\\clip.mp4\" ",
        "a root's backslash"
    );
    for name in ["C:\\%PATH%.txt", "C:\\!DROP!.txt"] {
        let refused = super::typed(&[PathBuf::from(name)], Quote::Cmd);
        assert!(refused.is_err(), "{name}: {refused:?}");
    }
}

#[test]
fn nushell_reads_dropped_paths_as_literal_arguments() {
    let paths = [
        "/tmp/plain",
        "/tmp/a space",
        "/tmp/it's",
        "/tmp/a\"quote",
        "/tmp/back\\slash",
        "/tmp/$env.HOME",
        "/tmp/(1 + 1)",
        "/tmp/*?[abc]",
        "/tmp/中文 café 🐚",
        "C:\\Windows\\",
        "/tmp/'#",
        "/tmp/'##",
        "/tmp/'###'####",
        "/tmp/r#'nested'#",
        "/tmp/ends'",
    ];
    let arguments = typed_as(Quote::Nushell, &paths);
    assert!(arguments.ends_with(' '), "each path is followed by a space");
    let command =
        format!("def capture [...args: string] {{ $args | to json --raw }}; capture {arguments}");
    let output = std::process::Command::new("nu")
        .args(["--no-config-file", "-c", &command])
        .output()
        .expect("Nushell must be installed and runnable as nu");
    assert!(
        output.status.success(),
        "Nushell rejected dropped paths: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actual: Vec<String> = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "Nushell did not return argument JSON: {error}; stdout: {}; stderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    assert_eq!(
        actual, paths,
        "Nushell must preserve every argument exactly"
    );
}

#[test]
fn a_path_with_a_control_character_types_nothing() {
    for quote in [Quote::Posix, Quote::PowerShell, Quote::Cmd, Quote::Nushell] {
        let paths = [PathBuf::from("/ok"), PathBuf::from("/tmp/a\nls")];
        let error = typed(&paths, quote).unwrap_err();
        assert!(error.to_string().contains("control character"), "{error}");
    }
}
