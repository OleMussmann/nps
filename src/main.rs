use clap::Parser;
use env_logger::Builder;
use log::LevelFilter;
use std::{
    fs,
    io::{self, IsTerminal},
    path::PathBuf,
    process::ExitCode,
};

mod cache_refresh;
mod cli;
mod matches;

/// Default settings for `nps`.
///
/// They are also listed in the `-h`/`--help` commands.
const DEFAULTS: cli::Defaults = cli::Defaults {
    cache_folder: ".nix-package-search", // /home/USER/...
    cache_file: "nps.cache",             // not user settable
    experimental: false,
    experimental_cache_file: "nps.experimental.cache", // not user settable
    color_mode: clap::ColorChoice::Auto,
    columns: cli::ColumnsChoice::All,
    flip: false,
    ignore_case: true,
    multi_line: false,
    multi_line_indent: "    ", // not user settable
    print_separator: true,
    quiet: false,
    truncate: false,

    exact_color: cli::Colors::Magenta,
    direct_color: cli::Colors::Blue,
    indirect_color: cli::Colors::Green,
};

fn main() -> ExitCode {
    // Get home dir errors out of the way, since clap can't propagate errors
    // from `derive`.
    let home = home::home_dir();
    if home.is_none() || home == Some("".into()) {
        Builder::new().filter_level(LevelFilter::Trace).init();
        log::error!("Can't find home dir.");
        return ExitCode::FAILURE;
    }
    let cli = cli::Cli::parse();

    let log_level = match cli.debug {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    Builder::new().filter_level(log_level).init();

    if cli.debug > 4 {
        log::error!("Max log level is 4, e.g. -dddd");
        return ExitCode::FAILURE;
    }

    log::debug!("Log level set to: {}", log_level);

    // Set a "supports-color" override based on the variable passed in.
    let color_choice = match cli.color {
        clap::ColorChoice::Always => {
            log::debug!("clap::ColorChoice set to Always");
            termcolor::ColorChoice::Always
        }
        clap::ColorChoice::Auto => {
            log::debug!("clap::ColorChoice request Auto");
            if io::stdout().is_terminal() {
                log::debug!("Running in terminal, clap::ColorChoice set to Auto");
                termcolor::ColorChoice::Auto
            } else {
                log::warn!("Not running in terminal, clap::ColorCoice forced to Never");
                termcolor::ColorChoice::Never
            }
        }
        clap::ColorChoice::Never => {
            log::debug!("clap::ColorChoice set to Never");
            termcolor::ColorChoice::Never
        }
    };

    let cache_file = PathBuf::from(DEFAULTS.cache_file);
    let experimental_cache_file = PathBuf::from(DEFAULTS.experimental_cache_file);

    log::trace!("cache_file: {:?}", cache_file);
    log::trace!("experimental_cache_file: {:?}", experimental_cache_file);

    let file_path: PathBuf = match cli.experimental {
        true => cli.cache_folder.join(&experimental_cache_file),
        false => cli.cache_folder.join(&cache_file),
    };

    log::trace!("file_path: {:?}", file_path);

    let cache_file_exists = file_path.exists();

    log::trace!("cache_file_exists: {}", cache_file_exists);
    log::trace!("cli.refresh: {}", cli.refresh);

    // Refresh cache with new info?
    if cli.refresh || !cache_file_exists {
        log::trace!("inside if");
        match cache_refresh::refresh(cli.experimental, &file_path, cli.quiet) {
            Ok(_) => {
                if cli.refresh {
                    return ExitCode::SUCCESS;
                }
            }
            Err(err) => {
                log::error!("Can't refresh cache: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(err) => {
            log::error!("Can't open file {}: {err}", &file_path.display());
            return ExitCode::FAILURE;
        }
    };

    let raw_matches = match matches::get_matches(&cli, &content) {
        Ok(raw_matches) => raw_matches,
        Err(err) => {
            log::error!("Can't get matches: {err}");
            return ExitCode::FAILURE;
        }
    };
    if raw_matches.is_empty() {
        return ExitCode::FAILURE;
    }

    let formatted_matches =
        match matches::format_matches(&cli, DEFAULTS.multi_line_indent, raw_matches) {
            Ok(formatted_matches) => formatted_matches,
            Err(err) => {
                log::error!("Can't sort matches: {err}");
                return ExitCode::FAILURE;
            }
        };

    let colored_matches = match matches::color_matches(
        &cli.search_term,
        &cli.exact_color,
        &cli.direct_color,
        &cli.indirect_color,
        cli.ignore_case,
        formatted_matches,
        color_choice,
    ) {
        Ok(colored_matches) => colored_matches,
        Err(err) => {
            log::error!("Can't color matches: {err}");
            return ExitCode::FAILURE;
        }
    };

    if let Err(err) =
        matches::print_matches(cli.flip, cli.separate, cli.multi_line, colored_matches)
    {
        log::error!("Can't print matches: {err}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
