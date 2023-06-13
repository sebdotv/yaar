use itertools::Itertools;
use log::{debug, info};
use std::process::Command;
use std::{iter, str};

pub fn run_command(program: &str, args: Vec<String>, dry_run: bool) {
    debug!(
        "command: {}",
        iter::once(program.to_string())
            .chain(args.clone())
            .collect_vec()
            .join(" ")
    );

    if dry_run {
        info!("Dry run requested, not executing command");
        return;
    }

    let cmd_output = Command::new(program)
        .args(args)
        .output()
        .expect("failed to execute process");

    debug!(
        "stdout: [{}]",
        str::from_utf8(cmd_output.stdout.as_slice()).expect("illegal UTF-8 in stdout")
    );

    debug!(
        "stderr: [{}]",
        str::from_utf8(cmd_output.stderr.as_slice()).expect("illegal UTF-8 in stderr")
    );

    debug!("{}", cmd_output.status);

    assert!(cmd_output.status.success());
    assert_eq!(cmd_output.stderr.len(), 0);
}
