use devkit_poe_complete::{Command, output, scripts};

#[test]
fn scripts_register_for_poe_and_call_devkit_for_data() {
  for (script, shell) in [(scripts::POWERSHELL, "powershell"), (scripts::BASH, "bash")] {
    // The shims ask one question, `query`; the old two-question protocol is gone.
    assert!(script.contains("devkit-complete query"), "{shell}");
    assert!(
      !script.contains("poe _list_tasks") && !script.contains("poe _describe_task_args"),
      "{shell}"
    );
  }
  assert!(scripts::POWERSHELL.contains("Register-ArgumentCompleter -CommandName poe"));
  assert!(scripts::BASH.contains("complete -F _poe_complete poe"));
}

#[test]
fn script_command_selects_the_shell() {
  assert_eq!(
    output(
      &Command::Script {
        powershell: true,
        bash: false
      },
      false
    ),
    scripts::POWERSHELL
  );
  assert_eq!(
    output(
      &Command::Script {
        powershell: false,
        bash: true
      },
      false
    ),
    scripts::BASH
  );
  // PowerShell is the default on this fleet.
  assert_eq!(
    output(
      &Command::Script {
        powershell: false,
        bash: false
      },
      false
    ),
    scripts::POWERSHELL
  );
}

#[test]
fn install_requires_at_least_one_shell_flag() {
  use clap::Parser as _;
  assert!(devkit_poe_complete::Args::try_parse_from(["devkit-complete", "install"]).is_err());
  let a = devkit_poe_complete::Args::try_parse_from(["devkit-complete", "install", "--powershell", "--bash", "--dry-run"]).unwrap();
  assert!(matches!(
    a.command,
    Command::Install {
      powershell: true,
      bash: true,
      dry_run: true
    }
  ));
}
