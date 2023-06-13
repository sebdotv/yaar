mod command;
mod config;

use clap::Parser;
use command::run_command;
use config::load_config;
use config::{OutputMode, Profile};
use log::{debug, info, trace, warn};
use std::collections::{HashMap, HashSet};
use std::str;
use xrandr::{XHandle, XrandrError};

#[derive(clap::Parser, Debug)]
struct Args {
    /// List available profiles with current connected devices
    #[arg(short, long, group = "action")]
    list: bool,

    /// Name of the profile to apply (by default, the only perfect match is applied)
    #[arg(short, long, group = "action")]
    profile: Option<String>,

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

    let cfg = load_config();
    debug!("loaded config with {} profiles", cfg.profiles.len());

    let outputs = get_xrandr_outputs().expect("failed to get xrandr ouputs");
    let outputs_by_edid = index_outputs_by_id(outputs);
    let current_edids: HashSet<String> = outputs_by_edid.keys().cloned().collect();

    let matching_profiles = list_matching_profiles(cfg.profiles.iter().collect(), current_edids);
    if args.list {
        info!(
            "matching profiles:\n{}",
            matching_profiles
                .iter()
                .map(|(profile_name, _, perfect_match)| format!(
                    "{}{}",
                    profile_name,
                    if *perfect_match { " [perfect]" } else { "" }
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );
    } else {
        match matching_profiles
            .iter()
            .filter(|(profile_name, _, perfect_match)| match args.profile {
                Some(ref profile_name_arg) => *profile_name == profile_name_arg,
                None => *perfect_match,
            })
            .collect::<Vec<_>>()
            .as_slice()
        {
            [] => warn!("No matching profiles found"),
            &[(profile_name, profile, perfect_match)] => {
                debug!(
                    "applying profile {} (perfect match: {}) with setup {:?}",
                    profile_name, perfect_match, profile.setup
                );
                let cmd_args = compute_cmd_args(outputs_by_edid, profile);
                run_command("xrandr", cmd_args, args.dry_run);
                info!("Successfully applied profile {}", profile_name);
            }
            _ => panic!("expected at most 1 matching profile"),
        }
    }
}

pub fn list_matching_profiles<'a>(
    candidate_profiles: Vec<(&'a String, &'a Profile)>,
    current_edids: HashSet<String>,
) -> Vec<(&'a String, &'a Profile, bool)> {
    candidate_profiles
        .into_iter()
        .filter_map(|(profile_name, profile)| {
            trace!("trying profile {}", profile_name);
            let profile_edids: HashSet<String> = profile.outputs.values().cloned().collect();
            // keeps profile only if it is a subset of current_edids; return whether it is a perfect match
            profile_edids.is_subset(&current_edids).then_some((
                profile_name,
                profile,
                profile_edids == current_edids,
            ))
        })
        .collect()
}

fn compute_cmd_args(
    outputs_by_edid: HashMap<String, xrandr::Output>,
    profile: &Profile,
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
                OutputMode::Off => vec!["--off"],
                OutputMode::Primary => vec!["--auto", "--primary"],
                OutputMode::Secondary => vec![
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

fn get_primary_output_key(profile: &Profile) -> &String {
    let primary_output_keys: Vec<&String> = profile
        .setup
        .iter()
        .filter_map(|(output_key, output_mode)| match output_mode {
            OutputMode::Primary => Some(output_key),
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
