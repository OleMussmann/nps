use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{ArgAction, Parser, ValueEnum};

use std::{path::PathBuf, str};

/// Find SEARCH_TERM in available nix packages and sort results by relevance.
///
/// List up to three columns, the latter two being optional:
/// PACKAGE_NAME  <PACKAGE_VERSION>  <PACKAGE_DESCRIPTION>
///
/// Matches are sorted by type. Show 'indirect' matches first, then 'direct' matches, and finally 'exact' matches.
///
///   indirect  fooSEARCH_TERMbar (SEARCH_TERM appears in any column)
///   direct    SEARCH_TERMbar (PACKAGE_NAME starts with SEARCH_TERM)
///   exact     SEARCH_TERM (PACKAGE_NAME is exactly SEARCH_TERM)
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    verbatim_doc_comment,
    styles = styles(),
    after_long_help = option_help_text(ENV_VAR_OPTIONS)
)]
pub struct Cli {
    // default_value_t: value if flag (or env var) not present
    // default_missing_value: value if flag is present, but has no value
    //                        needs `.num_args(0..N)` and `.require_equals(true)`
    // require_equals: force `--option=val` syntax
    // env: read env var if flag not present
    // takes_values: accept values from command line
    // hide: hides the option from `-h`, those parameters are set via env vars
    /// Highlight search matches in color
    #[arg(
        short,
        long = "color",
        require_equals = true,
        visible_alias = "colour",
        default_value_t = crate::DEFAULTS.color_mode,
        default_missing_value = "clap::ColorChoice::Auto",
        num_args = 0..=1,
        env = "NIX_PACKAGE_SEARCH_COLOR_MODE"
        )]
    pub color: clap::ColorChoice,

    /// Choose columns to show
    #[arg(
        short = 'C',
        long = "columns",
        require_equals = true,
        default_value_t = crate::DEFAULTS.columns,
        default_missing_value = "ColumnsChoice::All",
        value_enum,
        num_args = 0..=1,
        env = "NIX_PACKAGE_SEARCH_COLUMNS"
    )]
    pub columns: ColumnsChoice,

    /// Turn debugging information on
    ///
    /// Use up to four times for increased verbosity
    #[arg(
        short,
        long,
        action = ArgAction::Count
    )]
    pub debug: u8,

    /// Use experimental flakes
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = crate::DEFAULTS.experimental,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_EXPERIMENTAL"
    )]
    pub experimental: bool,

    /// Flip the order of matches and sorting
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = crate::DEFAULTS.flip,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_FLIP"
    )]
    pub flip: bool,

    /// Ignore case
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = crate::DEFAULTS.ignore_case,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_IGNORE_CASE"
    )]
    pub ignore_case: bool,

    /// Multi line
    ///
    /// Print search matches on two lines, followed by a newline:
    /// > PACKAGE_NAME  PACKAGE_VERSION
    /// >     PACKAGE_DESCRIPTION
    /// >
    #[arg(
        short,
        long,
        verbatim_doc_comment,
        require_equals = true,
        default_value_t = crate::DEFAULTS.multi_line,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_MULTI_LINE"
    )]
    pub multi_line: bool,

    /// Suppress non-debug messages
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = crate::DEFAULTS.quiet,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_QUIET"
    )]
    pub quiet: bool,

    /// Refresh package cache and exit
    #[arg(short, long)]
    pub refresh: bool,

    /// Separate match types with a newline
    ///
    /// Only applicable when --multi-line=false
    #[arg(
        short,
        long,
        require_equals = true,
        default_value_t = crate::DEFAULTS.print_separator,
        default_missing_value = "true",
        num_args = 0..=1,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_PRINT_SEPARATOR"
    )]
    pub separate: bool,

    /// Search for any SEARCH_TERM in package names, description, or versions
    #[arg(
        required_unless_present_any = ["refresh"]
    )]
    pub search_term: Option<String>,

    // hidden vars, to be set via env vars
    /// Cache lives here
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value = home::home_dir()
            .unwrap()  // We previously made sure this works.
            .join(crate::DEFAULTS.cache_folder)
            .display()
            .to_string(),
        value_parser = clap::value_parser!(PathBuf),
        env = "NIX_PACKAGE_SEARCH_CACHE_FOLDER_ABSOLUTE_PATH"
    )]
    pub cache_folder: PathBuf,

    /// Color of EXACT matches, match SEARCH_TERM
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value_t = crate::DEFAULTS.exact_color,
        value_enum,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_EXACT_COLOR"
    )]
    pub exact_color: Colors,

    /// Color of DIRECT matches, match SEARCH_TERMbar
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value_t = crate::DEFAULTS.direct_color,
        value_enum,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_DIRECT_COLOR"
    )]
    pub direct_color: Colors,

    /// Color of DIRECT matches, match fooSEARCH_TERMbar (or match other columns)
    #[arg(
        long,
        require_equals = true,
        hide = true,
        default_value_t = crate::DEFAULTS.indirect_color,
        value_enum,
        action = ArgAction::Set,
        env = "NIX_PACKAGE_SEARCH_INDIRECT_COLOR"
    )]
    pub indirect_color: Colors,
}

/// Help text for using environment variables for configuration.
///
/// Contains template items that still need to be replaced.
static ENV_VAR_OPTIONS: &str = "
CONFIGURATION

`nps` can be configured with environment variables. You can set these in
the configuration file of your shell, e.g. .bashrc/.zshrc

NIX_PACKAGE_SEARCH_EXPERIMENTAL
  Use the experimental 'nix search' command.
  It pulls information from the nix flake registries instead of nix channels.
  This is useful if no channels are in use, or channels are not updated
  regularly.
    [default: {DEFAULT_EXPERIMENTAL}]
    [possible values: true, false]

NIX_PACKAGE_SEARCH_FLIP
  Flip the order of matches? By default most relevant matches appear below,
  which is easier to read with long output. Flipping shows most relevant
  matches on top.
    [default: {DEFAULT_FLIP}]
    [possible values: true, false]

NIX_PACKAGE_SEARCH_CACHE_FOLDER_ABSOLUTE_PATH
  Absolute path of the cache folder
    [default: {DEFAULT_CACHE_FOLDER}]
    [possible values: path]

NIX_PACKAGE_SEARCH_COLUMNS
  Choose columns to show: PACKAGE_NAME plus any of PACKAGE_VERSION or
  PACKAGE_DESCRIPTION
    [default: {DEFAULT_COLUMNS}]
    [possible values: all, none, version, description]

NIX_PACKAGE_SEARCH_EXACT_COLOR
  Color of EXACT matches, match SEARCH_TERM in PACKAGE_NAME
    [default: {DEFAULT_EXACT_COLOR}]
    [possible values: black, blue, green, red, cyan, magenta, yellow, white]

NIX_PACKAGE_SEARCH_DIRECT_COLOR
  Color of DIRECT matches, match SEARCH_TERMbar in PACKAGE_NAME
    [default: {DEFAULT_DIRECT_COLOR}]
    [possible values: black, blue, green, red, cyan, magenta, yellow, white]

NIX_PACKAGE_SEARCH_INDIRECT_COLOR
  Color of INDIRECT matches, match fooSEARCH_TERMbar in any column
    [default: {DEFAULT_INDIRECT_COLOR}]
    [possible values: black, blue, green, red, cyan, magenta, yellow, white]

NIX_PACKAGE_SEARCH_COLOR_MODE
  Show search matches in color
  auto: Only show color if stdout is in terminal, suppress if e.g. piped
    [default: {DEFAULT_COLOR_MODE}]
    [possible values: always, never, auto]

NIX_PACKAGE_SEARCH_PRINT_SEPARATOR
  Separate matches with a newline?
    [default: {DEFAULT_PRINT_SEPARATOR}]
    [possible values: true, false]

NIX_PACKAGE_SEARCH_QUIET
  Suppress non-debug messages?
    [default: {DEFAULT_QUIET}]
    [possible values: true, false]

NIX_PACKAGE_SEARCH_IGNORE_CASE
  Search ignore capitalization for the search?
    [default: {DEFAULT_IGNORE_CASE}]
    [possible values: true, false]

NIX_PACKAGE_SEARCH_MULTI_LINE
  Print search matches on multiple lines?
  > PACKAGE_NAME  PACKAGE_VERSION
  >     PACKAGE_DESCRIPTION
  >
    [default: {DEFAULT_MULTI_LINE}]
    [possible values: true, false]
";

/// Supply Styles for colored help output.
fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Red.on_default() | Effects::BOLD)
        .usage(AnsiColor::Red.on_default() | Effects::BOLD)
        .literal(AnsiColor::Blue.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Green.on_default())
}

/// Replace template items in long help text with default settings.
fn option_help_text(help_text: &str) -> String {
    help_text
        .replace(
            "{DEFAULT_EXPERIMENTAL}",
            &crate::DEFAULTS.experimental.to_string(),
        )
        .replace(
            "{DEFAULT_CACHE_FOLDER}",
            &home::home_dir()
                .unwrap() // We previously made sure this works.
                .join(crate::DEFAULTS.cache_folder)
                .display()
                .to_string(),
        )
        .replace("{DEFAULT_CACHE_FILE}", crate::DEFAULTS.cache_file)
        .replace(
            "{DEFAULT_EXPERIMENTAL_CACHE_FILE}",
            crate::DEFAULTS.experimental_cache_file,
        )
        .replace(
            "{DEFAULT_COLOR_MODE}",
            &crate::DEFAULTS.color_mode.to_string().to_lowercase(),
        )
        .replace(
            "{DEFAULT_COLUMNS}",
            &format!("{:?}", crate::DEFAULTS.columns).to_lowercase(),
        )
        .replace("{DEFAULT_FLIP}", &crate::DEFAULTS.flip.to_string())
        .replace(
            "{DEFAULT_IGNORE_CASE}",
            &crate::DEFAULTS.ignore_case.to_string(),
        )
        .replace(
            "{DEFAULT_MULTI_LINE}",
            &crate::DEFAULTS.multi_line.to_string(),
        )
        .replace(
            "{DEFAULT_PRINT_SEPARATOR}",
            &crate::DEFAULTS.print_separator.to_string(),
        )
        .replace("{DEFAULT_QUIET}", &crate::DEFAULTS.quiet.to_string())
        .replace(
            "{DEFAULT_EXACT_COLOR}",
            &format!("{:?}", crate::DEFAULTS.exact_color).to_lowercase(),
        )
        .replace(
            "{DEFAULT_DIRECT_COLOR}",
            &format!("{:?}", crate::DEFAULTS.direct_color).to_lowercase(),
        )
        .replace(
            "{DEFAULT_INDIRECT_COLOR}",
            &format!("{:?}", crate::DEFAULTS.indirect_color).to_lowercase(),
        )
}

/// Column name options
#[derive(Clone, Debug, ValueEnum)]
pub enum ColumnsChoice {
    /// Show all columns
    All,
    /// Show only PACKAGE_NAME
    None,
    /// Also show PACKAGE_VERSION
    Version,
    /// Also show PACKAGE_DESCRIPTION
    Description,
}

/// Allowed values for coloring output.
#[derive(Debug, Clone, ValueEnum)]
pub enum Colors {
    Black,
    Blue,
    Green,
    Red,
    Cyan,
    Magenta,
    Yellow,
    White,
}

/// Defines possible default settings.
pub struct Defaults<'a> {
    pub cache_folder: &'a str,
    pub cache_file: &'a str,
    pub experimental: bool,
    pub experimental_cache_file: &'a str,
    pub color_mode: clap::ColorChoice,
    pub columns: ColumnsChoice,
    pub flip: bool,
    pub ignore_case: bool,
    pub multi_line: bool,
    pub multi_line_indent: &'a str,
    pub print_separator: bool,
    pub quiet: bool,

    pub exact_color: Colors,
    pub direct_color: Colors,
    pub indirect_color: Colors,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_help_text() {
        let replaced = option_help_text(ENV_VAR_OPTIONS);

        // Make sure we replace all possible placeholders with values
        assert!(!replaced.contains("DEFAULT"));
    }
}
