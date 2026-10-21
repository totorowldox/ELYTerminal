use super::*;

fn start() -> Run {
    let config = r#"
$env.config.show_banner = false
$env.config.shell_integration.osc2 = false
$env.config.shell_integration.osc133 = false
$env.config.shell_integration.osc633 = true
$env.config.hooks.pre_prompt = [{||
    $env.ELY_HOOK_COUNT = (($env.ELY_HOOK_COUNT? | default 0) + 1)
}]
$env.config.hooks.pre_execution = [{|| $env.ELY_EXEC_HOOK = 'kept'}]
$env.PROMPT_COMMAND = {|| $'USER_STATUS=($env.LAST_EXIT_CODE) > ' }
$env.PROMPT_COMMAND_RIGHT = ''
$env.PROMPT_INDICATOR = ''
"#;
    Run::start_with(
        &installed("nu"),
        &[
            (".config/nushell/env.nu", "$env.ELY_ENV_FILE = 'kept'"),
            (".config/nushell/config.nu", config),
        ],
        |launch, home| {
            launch.cwd.push(INNER);
            for name in ["XDG_CONFIG_HOME", "XDG_DATA_HOME", "XDG_DATA_DIRS"] {
                launch
                    .env
                    .push((name.into(), home.join(name).display().to_string()));
            }
            launch.env.push(("TERM_PROGRAM".into(), "vscode".into()));
            let (_, args) = launch.program.as_mut().expect("Nu's program");
            args.extend([
                "--config-home".into(),
                home.join(".config/nushell").display().to_string(),
                "--no-history".into(),
            ]);
        },
    )
}

#[test]
fn nushell_reports_exact_folders_and_preserves_user_configuration_and_status() {
    let mut run = start();
    let initial = run.start.join(INNER);
    run.until("initial folder and prompt", |run| {
        run.folders.contains(&initial) && run.marks.contains(&Mark::Input)
    });
    run.until("control-free initial title", |run| {
        run.titles.iter().any(|title| title == "in ner%xy")
    });
    run.ends(
        "print $'ENV=($env.ELY_ENV_FILE),HOOK=($env.ELY_HOOK_COUNT),EXEC=($env.ELY_EXEC_HOOK)'",
        &ended(0),
    );
    run.until("user configuration and hooks", |run| {
        run.shows("ENV=kept,HOOK=1,EXEC=kept")
    });
    run.send("mkdir 'literal%41中文'; cd 'literal%41中文'");
    let literal = initial.join("literal%41中文");
    run.until("literal percent and Unicode", |run| {
        run.folders.contains(&literal)
    });
    assert_eq!(run.folders[0], initial);
    run.ends("^false", &ended(1));
    run.until("user prompt's failed status", |run| {
        run.shows("USER_STATUS=1 >")
    });
    run.ends("error make {msg: 'failure'}", &ended(1));
    run.ends("print 'RECOVERED'", &ended(0));
    run.until("user prompt's successful status", |run| {
        run.shows("USER_STATUS=0 >")
    });
    run.until("folder title", |run| {
        run.titles.iter().any(|title| title == "literal%41中文")
    });
    assert!(run.titles.iter().any(|title| title == "^false"));
    assert!(run.marks.contains(&Mark::Prompt(Click::Absolute)));
    run.finish();
}
