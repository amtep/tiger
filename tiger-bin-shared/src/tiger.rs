use std::io::stdout;
use std::{mem::forget, path::PathBuf};

use anyhow::{bail, Result};
use clap::{error::ErrorKind, Args, Parser, Subcommand};
#[cfg(any(feature = "ck3", feature = "imperator", feature = "hoi4"))]
use tiger_lib::ModFile;
#[cfg(feature = "vic3")]
use tiger_lib::ModMetadata;
use tiger_lib::{
    disable_ansi_colors, emit_reports, get_version_from_launcher, set_show_loaded_mods,
    set_show_vanilla, suppress_from_json, validate_config_file, Everything,
};

use crate::gamedir::{
    find_game_directory_steam, find_paradox_directory, find_workshop_directory_steam,
};
use crate::update::update;
use crate::version::warn_versions;
use crate::GameConsts;

#[derive(Parser)]
#[clap(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(flatten)]
    validate_args: Option<ValidateArgs>,
}

#[derive(Subcommand)]
enum Commands {
    /// Update the binary. If no version is specified, the latest release is pulled from GitHub and
    /// installed over the current binary.
    Update {
        /// release version (e.g. 0.9.3)
        version: Option<String>,
    },
}

#[derive(Args)]
struct ValidateArgs {
    #[cfg(feature = "vic3")]
    /// Path to folder of mod to check.
    modpath: PathBuf,
    #[cfg(any(feature = "ck3", feature = "imperator", feature = "hoi4"))]
    /// Path to .mod file of mod to check.
    modpath: PathBuf,
    #[cfg_attr(feature = "ck3", clap(visible_alias = "ck3"))]
    #[cfg_attr(feature = "vic3", clap(visible_alias = "vic3"))]
    #[cfg_attr(feature = "imperator", clap(visible_alias = "imperator"))]
    #[clap(long)]
    /// Path to game main directory.
    game: Option<PathBuf>,
    #[cfg_attr(not(feature = "vic3"), clap(skip))]
    #[cfg_attr(feature = "vic3", clap(long))]
    /// Path to game workshop directory.
    workshop: Option<PathBuf>,
    #[cfg_attr(not(any(feature = "ck3", feature = "imperator", feature = "hoi4")), clap(skip))]
    #[cfg_attr(any(feature = "ck3", feature = "imperator", feature = "hoi4"), clap(long))]
    /// Path to paradox directory
    paradox: Option<PathBuf>,
    /// Path to custom .conf file.
    #[clap(long)]
    config: Option<PathBuf>,
    /// Show errors in the base game script code as well
    #[clap(long)]
    show_vanilla: bool,
    /// Show errors in other loaded mods as well
    #[clap(long)]
    show_mods: bool,
    /// Output the reports in JSON format
    #[clap(long)]
    json: bool,
    /// Consolidate certain repeated errors
    #[clap(long, short)]
    consolidate: bool,
    /// Warn about items that are defined but unused
    #[clap(long)]
    unused: bool,
    /// Do checks specific to the Princes of Darkness mod
    #[cfg(feature = "ck3")]
    #[clap(long)]
    pod: bool,
    /// Omit color from the output. False by default.
    /// Can also be configured in the config file.
    #[clap(long)]
    no_color: bool,
    /// Load a JSON file of reports to remove from the output.
    #[clap(long)]
    suppress: Option<PathBuf>,
}

/// Run the main tiger application.
///
/// It provides a number of command line arguments, as well as self-updating capability with the `update` subcommand.
#[allow(clippy::missing_panics_doc)] // it thinks we can panic on cli.validate_args.unwrap()
pub fn run(
    game_consts: &GameConsts,
    current_version: &'static str,
    bin_name: &'static str,
) -> Result<()> {
    use clap::{CommandFactory, FromArgMatches};

    let &GameConsts { name, name_short, version, app_id, signature_file, paradox_dir } =
        game_consts;

    let matches = Cli::command().version(current_version).name(bin_name).get_matches();
    let cli = Cli::from_arg_matches(&matches).map_err(|err| err.exit()).unwrap();

    #[allow(clippy::single_match_else)]
    match cli.command {
        Some(Commands::Update { version: target_version }) => {
            update(current_version, target_version.as_deref())?;
            Ok(())
        }
        None => {
            let mut args = cli.validate_args.unwrap();

            if args.json && args.consolidate {
                Cli::command()
                    .error(
                        ErrorKind::ArgumentConflict,
                        "Can't use report consolidation with JSON output.",
                    )
                    .exit();
            }

            #[cfg(windows)]
            if !args.no_color {
                let _ = ansiterm::enable_ansi_support()
                    .map_err(|_| eprintln!("Failed to enable ANSI support for Windows10 users. Continuing probably without colored output."));
            }

            if args.game.is_none() {
                args.game = find_game_directory_steam(app_id).ok();
            }
            if args.workshop.is_none() {
                args.workshop = find_workshop_directory_steam(app_id).ok();
            }
            if args.paradox.is_none() {
                args.paradox = find_paradox_directory(&PathBuf::from(paradox_dir));
            }
            if let Some(ref mut game) = args.game {
                eprintln!("Using {name_short} directory: {}", game.display());
                let mut sig = game.clone();
                sig.push(signature_file);
                if !sig.is_file() {
                    eprintln!("That does not look like a {name_short} directory.");
                    game.push("..");
                    eprintln!("Trying: {}", game.display());
                    sig.clone_from(game);
                    sig.push(signature_file);
                    if sig.is_file() {
                        eprintln!("Ok.");
                    } else {
                        bail!("Cannot find {name_short} directory. Please supply it as the --game option.");
                    }
                }
            } else {
                bail!("Cannot find {name_short} directory. Please supply it as the --game option.");
            }

            if let Some(ref game_dir) = args.game {
                if let Ok(launcher_game_version) = get_version_from_launcher(game_dir) {
                    if warn_versions(name, version, &launcher_game_version).is_err() {
                        eprintln!("Tiger was made for {name} version {version}.");
                        eprintln!(
                            "Comparing this to the game's version {launcher_game_version} failed."
                        );
                        eprintln!(
                            "If you are using a newer version of {name}, it may be inaccurate."
                        );
                    }
                } else {
                    eprintln!("Tiger was made for {name} version {version}.");
                    eprintln!("If you are using a newer version of {name}, it may be inaccurate.");
                }
            }

            args.config = validate_config_file(args.config);

            if let Some(suppress) = args.suppress {
                eprintln!("Suppressing reports from: {}", suppress.display());
                suppress_from_json(&suppress)?;
            }

            if args.show_vanilla {
                eprintln!("Showing warnings for base game files too. There will be many false positives in those.");
            }

            if args.show_mods {
                eprintln!("Showing warnings for other loaded mods too.");
            }

            if args.unused {
                eprintln!(
                    "Showing warnings for unused localization. There will be many false positives."
                );
            }

            #[cfg(feature = "ck3")]
            if args.pod {
                eprintln!("Doing special checks for the Princes of Darkness mod.");
            }

            if args.no_color {
                // Disable colors both here and after reading the config, because reading the modfile and config may emit errors.
                disable_ansi_colors();
            }

            let mut everything;

            #[cfg(any(feature = "ck3", feature = "imperator", feature = "hoi4"))]
            {
                if args.modpath.is_dir() {
                    args.modpath.push("descriptor.mod");
                }

                let modfile = ModFile::read(&args.modpath)?;
                let modpath = modfile.modpath();
                if !modpath.exists() {
                    eprintln!("Looking for mod in {}", modpath.display());
                    bail!("Cannot find mod directory. Please make sure the .mod file is correct.");
                }
                eprintln!("Using mod directory: {}", modpath.display());

                everything = Everything::new(
                    args.config.as_deref(),
                    args.game.as_deref(),
                    args.workshop.as_deref(),
                    args.paradox.as_deref(),
                    &modpath,
                    modfile.replace_paths(),
                )?;
            }
            #[cfg(feature = "vic3")]
            {
                let metadata = ModMetadata::read(&args.modpath)?;
                eprintln!("Using mod directory: {}", metadata.modpath().display());

                everything = Everything::new(
                    args.config.as_deref(),
                    args.game.as_deref(),
                    args.workshop.as_deref(),
                    args.paradox.as_deref(),
                    &args.modpath,
                    metadata.replace_paths(),
                )?;
            }

            // Print a blank line between the preamble and the first report:
            eprintln!();

            everything.load_output_settings(true);
            everything.load_config_filtering_rules();

            let mut output = stdout();

            let mut any_printed = false;
            if !args.json {
                any_printed |= emit_reports(&mut output, false, args.consolidate, false);
            }

            // We must apply the --no-color flag AFTER loading and applying the config,
            // because we want it to override the config.
            if args.no_color {
                disable_ansi_colors();
            }
            // Same logic applies to showing vanilla and other mods
            if args.show_vanilla {
                set_show_vanilla(true);
            }
            if args.show_mods {
                set_show_loaded_mods(true);
            }

            everything.load_all();
            everything.validate_all();
            everything.check_rivers();

            #[cfg(feature = "ck3")]
            if args.pod {
                everything.check_pod();
            }

            if args.unused {
                everything.check_unused();
            }

            any_printed |= emit_reports(&mut output, args.json, args.consolidate, true);

            if !args.json && !any_printed {
                eprintln!("No problems found.");
            }

            // Properly dropping `everything` takes a noticeable amount of time, and we're exiting anyway.
            forget(everything);
            Ok(())
        }
    }
}
