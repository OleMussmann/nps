use serde::Deserialize;
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
    process::{Command, Stdio},
    str,
};
use tempfile::NamedTempFile;

/// Check if flakes are enabled
fn check_flakes_enabled() -> Result<bool, Box<dyn Error>> {
    let probe_for_flakes = Command::new("nix")
        .arg("--extra-experimental-features")
        .arg("nix-command")
        .arg("config")
        .arg("show")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Can't execute `nix` command: {err}"))?;
    let find_experimental_features = Command::new("grep")
        .arg("^experimental-features")
        .stdin(Stdio::from(probe_for_flakes.stdout.unwrap()))
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Can't execute `grep` command: {err}"))?;
    let find_flakes = Command::new("grep")
        .arg("flakes")
        .stdin(Stdio::from(find_experimental_features.stdout.unwrap()))
        .stdout(Stdio::piped())
        .status()
        .map_err(|err| format!("Can't execute `grep` command: {err}"))?;

    Ok(find_flakes.success())
}

/// Check if requested `nps` features match system features
///
/// Give helpful warnings if there is a mismatch.
fn check_for_features(
    flakes_enabled: bool,
    experimental: bool,
    quiet: bool,
) -> Result<(), Box<dyn Error>> {
    if flakes_enabled && !experimental {
        let flakes_messages = [
            "Feature mismatch:",
            "> Your system seems to be based on flakes.",
            "> You may want to use `nps -e=true ...` instead to enable querying flake-based packages.",
        ];
        for flake_message in flakes_messages {
            message(flake_message, quiet)?;
            log::warn!("{}", flake_message);
        }
    }
    if !flakes_enabled && experimental {
        let channels_messages = [
            "Feature mismatch:",
            "> Your system seems to be based on channels.",
            "> You may want to use `nps -e=false ...` instead to query packages from channels.",
        ];
        for channel_message in channels_messages {
            message(channel_message, quiet)?;
            log::warn!("{}", channel_message);
        }
    }

    Ok(())
}

/// Print messages if quiet==false
fn message(message_string: &str, quiet: bool) -> Result<(), Box<dyn Error>> {
    if !quiet {
        writeln!(io::stdout(), "{}", message_string)
            .map_err(|err| format!("Can't write to stdout: {err}"))?;
    }
    Ok(())
}

/// Format to parse JSON package info into
#[derive(Debug, Deserialize)]
struct Package {
    // we are not using `pname`
    version: String,
    description: String,
}

/// Parse package info from JSON to (NAME VERSION DESCRIPTION) lines
fn parse_json_to_lines(raw_output: &str) -> Result<String, Box<dyn Error>> {
    // Load JSON package info into a HashMap
    let parsed: HashMap<String, Package> =
        serde_json::from_str(raw_output).map_err(|err| format!("Can't parse JSON: {err}"))?;

    let mut lines = vec![];
    for (name_string, package) in parsed.into_iter() {
        // `name_string` is, for example, "legacyPackages.x86_64-linux.auctex"
        // Keep everything after the second '.' to get the package "name".
        // This is different from package.pname, which contains the name
        // of the executable, which can be different from the package name.
        let name_vec: Vec<&str> = name_string.splitn(3, '.').collect();
        let name = name_vec.get(2).ok_or("Can't get package name from JSON.")?;
        lines.push(format!(
            "{} {} {}",
            name, package.version, package.description
        ));
    }
    lines.sort();
    Ok(lines.join("\n"))
}

/// Fetch new package info and write to cache file
pub fn refresh(experimental: bool, file_path: &PathBuf, quiet: bool) -> Result<(), Box<dyn Error>> {
    let flakes_enabled = check_flakes_enabled()?;
    // Print helpful warnings if there is a feature mismatch
    // between the system setup and the `nps` usage.
    check_for_features(flakes_enabled, experimental, quiet)?;

    let cache_start_message = "Refreshing cache. This might take a while...";
    log::info!("{}", cache_start_message);
    message(cache_start_message, quiet)?;

    let cache_folder = file_path
        .parent()
        .ok_or("Can't get cache folder from file path")?;
    log::trace!("file_path: {:?}", file_path);

    let output = match experimental {
        true => Command::new("nix")
            .arg("--extra-experimental-features")
            .arg("nix-command flakes")
            .arg("search")
            .arg("nixpkgs")
            .arg("^")
            .arg("--json")
            .output()
            .map_err(|err| format!("`nix search` failed: {err}"))?,
        false => Command::new("nix-env")
            .arg("-qaP")
            .arg("--description")
            .output()
            .map_err(|err| format!("`nix-env` failed: {err}"))?,
    };

    log::trace!("finished cli command");

    let (stdout, stderr) = (
        str::from_utf8(&output.stdout)
            .map_err(|err| format!("Can't convert stdout to UTF8: {err}"))?,
        str::from_utf8(&output.stderr)
            .map_err(|err| format!("Can't convert stderr to UTF8: {err}"))?,
    );

    log::trace!("stdout.len(): {}", stdout.len());
    log::trace!("stderr.len(): {}", stderr.len());

    // Report warnings if stderr looks bad
    let mut first_error = true;
    for line in stderr.lines() {
        // ignore standard logging to stderr
        if !line.starts_with("evaluating") {
            if first_error {
                log::warn!("These warnings were encountered during cache refresh (START)");
                first_error = false;
            }
            log::warn!("> {}", line);
        }
    }
    if !first_error {
        log::warn!("These warnings were encountered during cache refresh (END)");
    }

    // Throw error if cache is too small
    if stdout.len() < 10_000 {
        log::warn!("Cache seems too small:");
        log::warn!("> Query returned only {} lines.", stdout.len());
        if !flakes_enabled {
            log::info!(
                "> Did you set up your channels yet? See: https://nixos.wiki/wiki/Nix_channels"
            );
            log::info!(
                "> You can also set up your system for flakes instead. See: https://nixos.wiki/wiki/Flakes"
            );
        }
        log::info!("> Run with `-dddd` flag for even more information.");
        return Err("Cache seems too small. Run with `-dd` flag for more information.".into());
    }

    let cache_content = match experimental {
        true => parse_json_to_lines(stdout).map_err(|err| format!("Can't parse JSON: {err}"))?,
        false => {
            // Replace in every line the first two series of whitespaces with single spaces
            let re = regex::RegexBuilder::new(r"^([^ ]+) +([^ ]+) +(.*)$")
                .multi_line(true)
                .build()
                .unwrap();
            re.replace_all(stdout, "$1 $2 $3").to_string()
        }
    };

    log::trace!("trying to create folder: {:?}", cache_folder);
    // Create cache folder, if not exists
    fs::create_dir_all(cache_folder).map_err(|err| format!("Can't create folder: {err}"))?;
    log::trace!("folder created");

    log::trace!("cache_folder: {:?}", cache_folder);
    log::trace!("file_path: {:?}", &file_path);

    // Atomic Writing: Write first to a tmp file, then persist (move) it to destination
    let tempfile = NamedTempFile::new_in(cache_folder)
        .map_err(|err| format!("Can't create temp file: {err}"))?;
    log::trace!("tempfile: {:?}", &tempfile);
    log::trace!("trying to write tempfile");
    write!(&tempfile, "{}", cache_content)
        .map_err(|err| format!("Can't write to temp file: {err}"))?;
    log::trace!("tempfile written");

    tempfile
        .persist(file_path)
        .map_err(|err| format!("Can't persist temp file: {err}"))?;
    log::trace!("tempfile persisted");

    let number_of_packages = cache_content.lines().count();
    let cache_file_path_string = format!("{:?}", file_path);

    let cache_end_message =
        format!("Done. Cached info of {number_of_packages} packages in {cache_file_path_string}");
    log::info!("{}", &cache_end_message);
    message(&cache_end_message, quiet)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_parse_json_to_lines() -> Result<(), Box<dyn Error>> {
        init();

        let json = "{\
            \"legacyPackages.x86_64-linux.mypackage\": {\
            \"description\":\"i describe\",\
            \"pname\":\"mypackagebinary\",\
            \"version\":\"old\"},\
            \
            \"legacyPackages.x86_64-linux.myotherpackage\": {\
            \"description\":\"i also describe\",\
            \"pname\":\"myotherpackagebinary\",\
            \"version\":\"fresh\"}\
            }";
        let desired_output = "\
            myotherpackage fresh i also describe\n\
            mypackage old i describe\
            ";
        let parsed = parse_json_to_lines(json)?;

        assert_eq!(parsed, desired_output);
        Ok(())
    }

    #[test]
    fn test_check_flakes_enabled() {
        init();

        // Create a temporary directory for a nix.conf file
        let tempdir = tempfile::TempDir::new().unwrap();
        let nix_conf_dir = &tempdir.path().join("nix");
        fs::create_dir_all(nix_conf_dir).unwrap();

        let tempfile = NamedTempFile::new_in(&tempdir).unwrap();
        // Enable experimental features: "nix-command" and "flakes"
        write!(&tempfile, "experimental-features = nix-command flakes").unwrap();
        tempfile.persist(nix_conf_dir.join("nix.conf")).unwrap();

        temp_env::with_var("XDG_CONFIG_HOME", Some(&tempdir.path()), || {
            assert!(check_flakes_enabled().unwrap())
        });

        let tempfile = NamedTempFile::new_in(&tempdir).unwrap();
        // Disable all experimental features
        write!(&tempfile, "experimental-features = ").unwrap();
        tempfile.persist(nix_conf_dir.join("nix.conf")).unwrap();

        temp_env::with_var("XDG_CONFIG_HOME", Some(&tempdir.path()), || {
            assert!(!check_flakes_enabled().unwrap())
        });
    }
}
