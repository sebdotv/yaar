mod command;
mod config;
mod devices;

use clap::Parser;
use log::{debug, info, trace, warn};
use std::collections::{HashMap, HashSet};
use std::str;
use xrandr::{XHandle, XrandrError};

#[derive(clap::Parser, Debug)]
struct Args {
    /// Dry-run (do not make any changes to the system)
    #[arg(short, long)]
    dry_run: bool,

    /// Verbose output (repeat for very verbose output)
    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,
}

fn main() {
    let args = Args::parse();
    init_logger(args.verbose);

    let cfg = config::load_config();
    debug!("loaded config with {} profiles", cfg.profiles.len());

    let outputs = get_xrandr_outputs().expect("failed to get xrandr outputs");
    debug!("found {} outputs", outputs.len());
    if outputs.is_empty() {
        warn!("No outputs found");
        return;
    }
    let outputs_by_edid = index_outputs_by_id(outputs);
    if outputs_by_edid.is_empty() {
        warn!("No EDID found");
        return;
    }
    let current_edids: HashSet<String> = outputs_by_edid.keys().cloned().collect();

    if let Some((profile_name, profile)) =
        find_matching_profile(cfg.profiles.iter().collect(), current_edids)
    {
        debug!(
            "applying profile {} with setup {:?}",
            profile_name, profile.setup
        );
        let cmd_args = compute_cmd_args(outputs_by_edid, profile);
        command::run_command("xrandr", cmd_args, args.dry_run);
        if !args.dry_run {
            info!("Successfully applied profile {}", profile_name);
        }
    } else {
        warn!("No matching profile found");
    }
}

fn find_matching_profile<'a>(
    candidate_profiles: Vec<(&'a String, &'a config::Profile)>,
    current_edids: HashSet<String>,
) -> Option<(&'a String, &'a config::Profile)> {
    let matching_profile_entries: Vec<(&String, &config::Profile)> = candidate_profiles
        .into_iter()
        .filter(|(profile_name, profile)| {
            trace!("trying profile {}", profile_name);
            let profile_edids: HashSet<String> = profile.outputs.values().cloned().collect();
            profile_edids == current_edids
        })
        .collect();

    match matching_profile_entries.as_slice() {
        [] => None,
        &[(profile_name, profile)] => Some((profile_name, profile)),
        _ => panic!("expected exactly 1 primary"),
    }
}

fn compute_cmd_args(
    outputs_by_edid: HashMap<String, xrandr::Output>,
    profile: &config::Profile,
) -> Vec<String> {
    let primary_output_key = get_primary_output_key(profile);
    debug!("primary_output_key: {}", primary_output_key);

    let get_output_by_key = |output_key: &String| {
        let output_edid = profile
            .outputs
            .get(output_key)
            .expect("unknown output in config");
        let output = outputs_by_edid
            .get(output_edid)
            .expect("unexpected error: EDID not found");
        output
    };

    let all_args: Vec<&str> = profile
        .setup
        .iter()
        .flat_map(|(output_key, output_mode)| {
            let output = get_output_by_key(output_key);
            let mut args1 = vec!["--output", output.name.as_str()];
            let args2 = match output_mode {
                config::OutputMode::Off => vec!["--off"],
                config::OutputMode::Primary => vec!["--auto", "--primary"],
                config::OutputMode::Secondary => vec![
                    "--auto",
                    "--right-of",
                    get_output_by_key(primary_output_key).name.as_str(),
                ],
            };
            args1.extend(args2);
            args1
        })
        .collect();

    all_args.into_iter().map(String::from).collect()
}

fn get_primary_output_key(profile: &config::Profile) -> &String {
    let primary_output_keys: Vec<&String> = profile
        .setup
        .iter()
        .filter_map(|(output_key, output_mode)| match output_mode {
            config::OutputMode::Primary => Some(output_key),
            _ => None,
        })
        .collect();
    let primary_output_key = match primary_output_keys.as_slice() {
        &[x] => x,
        _ => panic!("expected exactly 1 primary"),
    };
    primary_output_key
}

fn index_outputs_by_id(outputs: Vec<xrandr::Output>) -> HashMap<String, xrandr::Output> {
    let mut outputs_by_edid = HashMap::new();
    for output in outputs {
        if let Some(edid) = output.edid() {
            let prev = outputs_by_edid.insert(hex::encode(edid), output);
            assert!(prev.is_none())
        }
    }
    outputs_by_edid
}

fn get_xrandr_outputs() -> Result<Vec<xrandr::Output>, XrandrError> {
    let mut x_handle = XHandle::open()?;
    let outputs = x_handle.all_outputs()?;
    Ok(outputs)
}

fn init_logger(verbose_level: u8) {
    let level = match verbose_level {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        x if x >= 2 => log::LevelFilter::Trace,
        _ => panic!(),
    };
    env_logger::Builder::new().filter_level(level).init();
}
