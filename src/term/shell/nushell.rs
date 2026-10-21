use std::process::Command;

use super::*;
use crate::term::tap;

#[test]
fn nushell_folder_uris_round_trip_unix_drive_and_unc_paths() {
    let cases = [
        ("/tmp/literal%41 space/中文\u{7}\n", false),
        ("/tmp/back\\slash", false),
        ("C:\\Users\\a b%41\\中文", true),
        ("\\\\server\\share\\a%41", true),
    ];
    let script =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/shell-integration/nu/elyterminal.nu");
    let mut command = format!(
        "source {}; [",
        nushell_literal(script.to_str().expect("UTF-8 source path"))
    );
    for (path, windows) in cases {
        command.push_str(&format!(
            "(_ely_file_uri {} {windows}) ",
            nushell_literal(path)
        ));
    }
    command.push_str("] | to json --raw");
    let output = Command::new("nu")
        .args(["--no-config-file", "-c", &command])
        .output()
        .expect("Nushell must be installed and runnable as nu");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let uris: Vec<String> = serde_json::from_slice(&output.stdout).expect("folder URI JSON");
    assert_eq!(uris.len(), cases.len());
    for ((path, windows), uri) in cases.into_iter().zip(uris) {
        let mut expected = if windows {
            path.replace('\\', "/")
        } else {
            path.into()
        };
        if !expected.starts_with('/') && !cfg!(windows) {
            expected.insert(0, '/');
        }
        let actual = tap::folder(uri.as_bytes()).expect("a local folder URI");
        assert_eq!(actual, PathBuf::from(expected), "{path:?}: {uri}");
    }
}

#[test]
fn a_failed_nushell_integration_stops_the_shell_with_its_owner_named() {
    let root = env::temp_dir().join(format!("elyterminal-nu-failure-{}", std::process::id()));
    fs::create_dir_all(root.join("nu")).expect("integration folder");
    fs::write(
        root.join("nu/elyterminal.nu"),
        "error make {msg: 'probe failure'}",
    )
    .expect("failed integration script");
    let launch = launch("nu", root.clone(), &root).expect("Nu launch");
    let (program, mut args) = launch.program.expect("Nu's program");
    args.push("--no-config-file".into());
    let output = Command::new(program)
        .args(args)
        .current_dir(&root)
        .output()
        .expect("Nu starts");
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("ELYTerminal: Nushell integration failed: probe failure"),
        "{error}"
    );
    fs::remove_dir_all(root).expect("cleanup");
}
