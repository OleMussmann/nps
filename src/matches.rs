use grep::{
    printer::{ColorSpecs, Standard, StandardBuilder, UserColorSpec},
    regex::RegexMatcherBuilder,
    searcher::SearcherBuilder,
};

use std::{
    error::Error,
    io::{self, Write},
    str,
};
use termcolor::{Buffer, BufferWriter};

use crate::cli;

type MatchVecs = (Vec<String>, Vec<String>, Vec<String>);

/// Case converter for case-insensitive searches
fn convert_case(string: &str, ignore_case: bool) -> String {
    match ignore_case {
        true => string.to_lowercase(),
        false => string.to_string(),
    }
}

/// Find matches from cache file
pub fn get_matches(cli: &cli::Cli, content: &str) -> Result<String, Box<dyn Error>> {
    let search_term = cli
        .search_term
        .as_ref()
        .ok_or("Can't get search term as ref")?;

    // Matcher to find search term in rows
    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(cli.ignore_case)
        .build(search_term)
        .map_err(|err| format!("Can't build regex: {err}"))?;
    // Printer collects matching rows in a Vec
    let mut printer = Standard::new_no_color(vec![]);

    // Execute search and collect output
    SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(&matcher, content.as_bytes(), printer.sink(&matcher))
        .map_err(|err| format!("Can't build searcher: {err}"))?;

    // into_inner gives us back the underlying writer we provided to
    // new_no_color, which is wrapped in a termcolor::NoColor. Thus, a second
    // into_inner gives us back the actual buffer.
    let output = String::from_utf8(printer.into_inner().into_inner())
        .map_err(|err| format!("Can't parse printer string: {err}"))?;

    Ok(output)
}

/// Sort matches into match types and pad the lines to aligned columns
pub fn format_matches(
    cli: &cli::Cli,
    indent: &str,
    raw_matches: String,
) -> Result<MatchVecs, Box<dyn Error>> {
    let search_term = cli
        .search_term
        .as_ref()
        .ok_or("Can't get search term as ref")?;

    let mut name_padding = 0;
    let mut version_padding = 0;

    if !cli.multi_line {
        let mut name_lengths: Vec<usize> = vec![];
        let mut version_lengths: Vec<usize> = vec![];

        for line in raw_matches.lines() {
            let split_line: Vec<&str> = line.splitn(3, ' ').collect();

            // Try to get a split_line element: `.get()`,
            // use &"" if missing: `.unwrap_or(&"")`,
            // and append lengths `.len()` to *_lengths vectors.
            #[allow(clippy::get_first)]
            name_lengths.push(split_line.get(0).unwrap_or(&"").len());
            version_lengths.push(split_line.get(1).unwrap_or(&"").len());
        }

        // Minimum cell size will be the largest contained string
        name_padding = *name_lengths.iter().max().unwrap_or(&0);
        version_padding = *version_lengths.iter().max().unwrap_or(&0);
    }

    let mut formatted_matches_exact: Vec<String> = vec![];
    let mut formatted_matches_direct: Vec<String> = vec![];
    let mut formatted_matches_indirect: Vec<String> = vec![];

    for line in raw_matches.lines() {
        let split_line: Vec<&str> = line.splitn(3, ' ').collect();

        #[allow(clippy::get_first)] // suppress clippy warning for this block
        let name = split_line.get(0).unwrap_or(&"");
        let version = split_line.get(1).unwrap_or(&"");
        let description = split_line.get(2).unwrap_or(&"");

        let assembled_line = match (&cli.columns, cli.multi_line, description) {
            (cli::ColumnsChoice::All, true, &"") => format!("{}  {}\n", name, version),
            (cli::ColumnsChoice::All, true, _) => {
                format!("{}  {}\n", name, version) + indent + &format!("{}\n", description)
            }
            (cli::ColumnsChoice::All, false, &"") => {
                format!("{:name_padding$}  {:version_padding$}", name, version)
            }
            (cli::ColumnsChoice::All, false, _) => format!(
                "{:name_padding$}  {:version_padding$}  {}",
                name, version, description
            ),

            (cli::ColumnsChoice::Version, true, _) => format!("{}  {}\n", name, version),
            (cli::ColumnsChoice::Version, false, _) => {
                format!("{:name_padding$}  {}", name, version)
            }

            (cli::ColumnsChoice::Description, true, &"") => format!("{}\n", name),
            (cli::ColumnsChoice::Description, true, _) => {
                format!("{}\n", name) + indent + &format!("{}\n", description)
            }
            (cli::ColumnsChoice::Description, false, &"") => format!("{:name_padding$}", name),
            (cli::ColumnsChoice::Description, false, _) => {
                format!("{:name_padding$}  {}", name, description)
            }

            (cli::ColumnsChoice::None, true, _) => format!("{}\n", name),
            (cli::ColumnsChoice::None, false, _) => name.to_string(),
        };

        // Handle case-insensitive, if requested
        let converted_search_term = convert_case(search_term, cli.ignore_case);
        let converted_name = convert_case(name, cli.ignore_case);

        // Package names from channels are prepended with "nixos." or "nixpkgs."
        match cli.experimental {
            true => {
                if converted_name == converted_search_term {
                    formatted_matches_exact.push(assembled_line);
                } else if converted_name.starts_with(&converted_search_term) {
                    formatted_matches_direct.push(assembled_line);
                } else {
                    formatted_matches_indirect.push(assembled_line);
                }
            }
            false => {
                if converted_name == ("nixos.".to_owned() + &converted_search_term)
                    || converted_name == ("nixpkgs.".to_owned() + &converted_search_term)
                {
                    formatted_matches_exact.push(assembled_line);
                } else if converted_name
                    .starts_with(&("nixos.".to_owned() + &converted_search_term))
                    || converted_name.starts_with(&("nixpkgs.".to_owned() + &converted_search_term))
                {
                    formatted_matches_direct.push(assembled_line);
                } else {
                    formatted_matches_indirect.push(assembled_line);
                }
            }
        }
    }

    // Let's have the top results at the bottom by default
    if !cli.flip {
        formatted_matches_exact.reverse();
        formatted_matches_direct.reverse();
        formatted_matches_indirect.reverse();
    }

    Ok((
        formatted_matches_exact,
        formatted_matches_direct,
        formatted_matches_indirect,
    ))
}

/// Color the search term in different match types
pub fn color_matches(
    maybe_search_term: &Option<String>,
    exact_color: &cli::Colors,
    direct_color: &cli::Colors,
    indirect_color: &cli::Colors,
    ignore_case: bool,
    formatted_matches: MatchVecs,
    color_choice: termcolor::ColorChoice,
) -> Result<[Buffer; 3], Box<dyn Error>> {
    let (padded_matches_exact, padded_matches_direct, padded_matches_indirect) = formatted_matches;
    let search_term = maybe_search_term
        .as_ref()
        .ok_or("Can't get search term as ref")?;

    // Defining different colors for different match types
    let exact_color_user_spec: UserColorSpec = format!("match:fg:{:?}", exact_color).parse()?;
    let direct_color_user_spec: UserColorSpec = format!("match:fg:{:?}", direct_color).parse()?;
    let indirect_color_user_spec: UserColorSpec =
        format!("match:fg:{:?}", indirect_color).parse()?;

    // Font styles for match types
    let exact_style: UserColorSpec = "match:style:bold".parse()?;
    let direct_style: UserColorSpec = "match:style:bold".parse()?;
    let indirect_style: UserColorSpec = "match:style:bold".parse()?;

    // Combining colors and styles to ColorSpecs
    let exact_color_specs = ColorSpecs::new(&[exact_color_user_spec, exact_style]);
    let direct_color_specs = ColorSpecs::new(&[direct_color_user_spec, direct_style]);
    let indirect_color_specs = ColorSpecs::new(&[indirect_color_user_spec, indirect_style]);

    // Create buffers to write colored output into
    let bufwtr = BufferWriter::stdout(color_choice);
    let mut exact_buffer = bufwtr.buffer();
    let mut direct_buffer = bufwtr.buffer();
    let mut indirect_buffer = bufwtr.buffer();

    // Printers print to the above buffers
    let mut exact_printer = StandardBuilder::new()
        .color_specs(exact_color_specs)
        .build(&mut exact_buffer);
    let mut direct_printer = StandardBuilder::new()
        .color_specs(direct_color_specs)
        .build(&mut direct_buffer);
    let mut indirect_printer = StandardBuilder::new()
        .color_specs(indirect_color_specs)
        .build(&mut indirect_buffer);

    // Matcher to color `search_term`
    let matcher = RegexMatcherBuilder::new()
        .case_insensitive(ignore_case)
        .build(search_term)
        .map_err(|err| format!("Can't build regex: {err}"))?;

    // Matcher to find _everything_, so lines without matches are still printed.
    // This can happen if certain columns are missing.
    let matcher_all = RegexMatcherBuilder::new().build(".*")?;

    // Coloring and printing to buffers
    SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(
            &matcher_all,
            padded_matches_exact.join("\n").as_bytes(),
            exact_printer.sink(&matcher),
        )
        .map_err(|err| format!("Can't build searcher: {err}"))?;
    SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(
            &matcher_all,
            padded_matches_direct.join("\n").as_bytes(),
            direct_printer.sink(&matcher),
        )
        .map_err(|err| format!("Can't build searcher: {err}"))?;
    SearcherBuilder::new()
        .line_number(false)
        .build()
        .search_slice(
            &matcher_all,
            padded_matches_indirect.join("\n").as_bytes(),
            indirect_printer.sink(&matcher),
        )
        .map_err(|err| format!("Can't build searcher: {err}"))?;

    Ok([exact_buffer, direct_buffer, indirect_buffer])
}

/// Print matches to screen in correct ordering
pub fn print_matches(
    flip: bool,
    separate: bool,
    multi_line: bool,
    colored_matches: [Buffer; 3],
) -> Result<(), Box<dyn Error>> {
    // Assemble match type string segments
    let mut out: Vec<String> = vec![];
    for buffer in colored_matches.into_iter() {
        let content = String::from_utf8(buffer.into_inner())
            .map_err(|err| format!("Can't get string from buffer: {err}"))?;
        if !content.is_empty() {
            out.push(content);
        }
    }

    if !flip {
        out.reverse();
    }

    // Use newlines as separators, if requested or for multi lines
    let separator = match separate || multi_line {
        true => "\n".to_string(),
        false => "".to_string(),
    };
    // BufferWriter introduces a newline that we need to trim for some reason
    writeln!(io::stdout(), "{}", &out.join(&separator).trim())
        .map_err(|err| format!("Can't write to stdout: {err}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_get_matches() {
        init();

        let cli = cli::Cli::try_parse_from(vec!["nps", "second"]).unwrap();
        let content = "\
            the first line\n\
            the second line\n\
            the third line\
            ";
        let matches = get_matches(&cli, content).unwrap();

        assert_eq!(matches, "the second line\n");
    }

    #[test]
    fn test_convert_case() {
        init();

        let test_string = "abCDef";

        assert_eq!(convert_case(test_string, false), "abCDef");
        assert_eq!(convert_case(test_string, true), "abcdef");
    }

    #[test]
    fn test_format_matches() {
        init();

        let cli_all_columns =
            cli::Cli::try_parse_from(vec!["nps", "-e=true", "mypackage"]).unwrap();
        let cli_no_other_columns =
            cli::Cli::try_parse_from(vec!["nps", "-e=true", "-C=none", "mypackage"]).unwrap();
        let cli_version_column =
            cli::Cli::try_parse_from(vec!["nps", "-e=true", "-C=version", "mypackage"]).unwrap();
        let cli_description_column =
            cli::Cli::try_parse_from(vec!["nps", "-e=true", "-C=description", "mypackage"])
                .unwrap();

        let cli_all_columns_multi_line =
            cli::Cli::try_parse_from(vec!["nps", "-e=true", "-m=true", "mypackage"]).unwrap();
        let cli_no_other_columns_multi_line =
            cli::Cli::try_parse_from(vec!["nps", "-e=true", "-C=none", "-m=true", "mypackage"])
                .unwrap();
        let cli_version_column_multi_line =
            cli::Cli::try_parse_from(vec!["nps", "-e=true", "-C=version", "-m=true", "mypackage"])
                .unwrap();
        let cli_description_column_multi_line = cli::Cli::try_parse_from(vec![
            "nps",
            "-e=true",
            "-C=description",
            "-m=true",
            "mypackage",
        ])
        .unwrap();

        let matches = "\
            mypackage v1 my package description\n\
            mypackage v7\n\
            myotherpackage v2 has description as well\n\
            mypackage_extension v3 words words\n\
            mypackage_extension_2 v4 words words w0rds\n\
            mypackage_extension_3 v8\n\
            mylastpackage v5.0.0 is not mypackage\n\
            no_description v6\
            "
        .to_string();

        // single line
        let exact_matches_all_columns = "\
            mypackage              v7    \n\
            mypackage              v1      my package description\
            ";
        let direct_matches_all_columns = "\
            mypackage_extension_3  v8    \n\
            mypackage_extension_2  v4      words words w0rds\n\
            mypackage_extension    v3      words words\
            ";
        let indirect_matches_all_columns = "\
            no_description         v6    \n\
            mylastpackage          v5.0.0  is not mypackage\n\
            myotherpackage         v2      has description as well\
            ";

        let exact_matches_no_other_columns = "\
            mypackage\n\
            mypackage\
            ";
        let direct_matches_no_other_columns = "\
            mypackage_extension_3\n\
            mypackage_extension_2\n\
            mypackage_extension\
            ";
        let indirect_matches_no_other_columns = "\
            no_description\n\
            mylastpackage\n\
            myotherpackage\
            ";

        let exact_matches_version_column = "\
            mypackage              v7\n\
            mypackage              v1\
            ";
        let direct_matches_version_column = "\
            mypackage_extension_3  v8\n\
            mypackage_extension_2  v4\n\
            mypackage_extension    v3\
            ";
        let indirect_matches_version_column = "\
            no_description         v6\n\
            mylastpackage          v5.0.0\n\
            myotherpackage         v2\
            ";

        let exact_matches_description_column = "\
            mypackage            \n\
            mypackage              my package description\
            ";
        let direct_matches_description_column = "\
            mypackage_extension_3\n\
            mypackage_extension_2  words words w0rds\n\
            mypackage_extension    words words\
            ";
        let indirect_matches_description_column = "\
            no_description       \n\
            mylastpackage          is not mypackage\n\
            myotherpackage         has description as well\
            ";

        // multi-line
        let exact_matches_all_columns_multi_line = "\
            mypackage  v7\n\n\
            mypackage  v1\n    my package description\n\
            ";
        let direct_matches_all_columns_multi_line = "\
            mypackage_extension_3  v8\n\n\
            mypackage_extension_2  v4\n    words words w0rds\n\n\
            mypackage_extension  v3\n    words words\n\
            ";
        let indirect_matches_all_columns_multi_line = "\
            no_description  v6\n\n\
            mylastpackage  v5.0.0\n    is not mypackage\n\n\
            myotherpackage  v2\n    has description as well\n\
            ";

        let exact_matches_no_other_columns_multi_line = "\
            mypackage\n\n\
            mypackage\n\
            ";
        let direct_matches_no_other_columns_multi_line = "\
            mypackage_extension_3\n\n\
            mypackage_extension_2\n\n\
            mypackage_extension\n\
            ";
        let indirect_matches_no_other_columns_multi_line = "\
            no_description\n\n\
            mylastpackage\n\n\
            myotherpackage\n\
            ";

        let exact_matches_version_column_multi_line = "\
            mypackage  v7\n\n\
            mypackage  v1\n\
            ";
        let direct_matches_version_column_multi_line = "\
            mypackage_extension_3  v8\n\n\
            mypackage_extension_2  v4\n\n\
            mypackage_extension  v3\n\
            ";
        let indirect_matches_version_column_multi_line = "\
            no_description  v6\n\n\
            mylastpackage  v5.0.0\n\n\
            myotherpackage  v2\n\
            ";

        let exact_matches_description_column_multi_line = "\
            mypackage\n\n\
            mypackage\n    my package description\n\
            ";
        let direct_matches_description_column_multi_line = "\
            mypackage_extension_3\n\n\
            mypackage_extension_2\n    words words w0rds\n\n\
            mypackage_extension\n    words words\n\
            ";
        let indirect_matches_description_column_multi_line = "\
            no_description\n\n\
            mylastpackage\n    is not mypackage\n\n\
            myotherpackage\n    has description as well\n\
            ";

        let sorted_and_padded_all_columns = format_matches(
            &cli_all_columns,
            crate::DEFAULTS.multi_line_indent,
            matches.clone(),
        )
        .unwrap();
        let sorted_and_padded_no_other_columns = format_matches(
            &cli_no_other_columns,
            crate::DEFAULTS.multi_line_indent,
            matches.clone(),
        )
        .unwrap();
        let sorted_and_padded_version_column = format_matches(
            &cli_version_column,
            crate::DEFAULTS.multi_line_indent,
            matches.clone(),
        )
        .unwrap();
        let sorted_and_padded_description_column = format_matches(
            &cli_description_column,
            crate::DEFAULTS.multi_line_indent,
            matches.clone(),
        )
        .unwrap();

        let sorted_and_padded_all_columns_multi_line = format_matches(
            &cli_all_columns_multi_line,
            crate::DEFAULTS.multi_line_indent,
            matches.clone(),
        )
        .unwrap();
        let sorted_and_padded_no_other_columns_multi_line = format_matches(
            &cli_no_other_columns_multi_line,
            crate::DEFAULTS.multi_line_indent,
            matches.clone(),
        )
        .unwrap();
        let sorted_and_padded_version_column_multi_line = format_matches(
            &cli_version_column_multi_line,
            crate::DEFAULTS.multi_line_indent,
            matches.clone(),
        )
        .unwrap();
        let sorted_and_padded_description_column_multi_line = format_matches(
            &cli_description_column_multi_line,
            crate::DEFAULTS.multi_line_indent,
            matches,
        )
        .unwrap();

        assert_eq!(
            exact_matches_all_columns,
            sorted_and_padded_all_columns.0.join("\n")
        );
        assert_eq!(
            direct_matches_all_columns,
            sorted_and_padded_all_columns.1.join("\n")
        );
        assert_eq!(
            indirect_matches_all_columns,
            sorted_and_padded_all_columns.2.join("\n")
        );

        assert_eq!(
            exact_matches_no_other_columns,
            sorted_and_padded_no_other_columns.0.join("\n")
        );
        assert_eq!(
            direct_matches_no_other_columns,
            sorted_and_padded_no_other_columns.1.join("\n")
        );
        assert_eq!(
            indirect_matches_no_other_columns,
            sorted_and_padded_no_other_columns.2.join("\n")
        );

        assert_eq!(
            exact_matches_version_column,
            sorted_and_padded_version_column.0.join("\n")
        );
        assert_eq!(
            direct_matches_version_column,
            sorted_and_padded_version_column.1.join("\n")
        );
        assert_eq!(
            indirect_matches_version_column,
            sorted_and_padded_version_column.2.join("\n")
        );

        assert_eq!(
            exact_matches_description_column,
            sorted_and_padded_description_column.0.join("\n")
        );
        assert_eq!(
            direct_matches_description_column,
            sorted_and_padded_description_column.1.join("\n")
        );
        assert_eq!(
            indirect_matches_description_column,
            sorted_and_padded_description_column.2.join("\n")
        );

        assert_eq!(
            exact_matches_all_columns_multi_line,
            sorted_and_padded_all_columns_multi_line.0.join("\n")
        );
        assert_eq!(
            direct_matches_all_columns_multi_line,
            sorted_and_padded_all_columns_multi_line.1.join("\n")
        );
        assert_eq!(
            indirect_matches_all_columns_multi_line,
            sorted_and_padded_all_columns_multi_line.2.join("\n")
        );

        assert_eq!(
            exact_matches_no_other_columns_multi_line,
            sorted_and_padded_no_other_columns_multi_line.0.join("\n")
        );
        assert_eq!(
            direct_matches_no_other_columns_multi_line,
            sorted_and_padded_no_other_columns_multi_line.1.join("\n")
        );
        assert_eq!(
            indirect_matches_no_other_columns_multi_line,
            sorted_and_padded_no_other_columns_multi_line.2.join("\n")
        );

        assert_eq!(
            exact_matches_version_column_multi_line,
            sorted_and_padded_version_column_multi_line.0.join("\n")
        );
        assert_eq!(
            direct_matches_version_column_multi_line,
            sorted_and_padded_version_column_multi_line.1.join("\n")
        );
        assert_eq!(
            indirect_matches_version_column_multi_line,
            sorted_and_padded_version_column_multi_line.2.join("\n")
        );

        assert_eq!(
            exact_matches_description_column_multi_line,
            sorted_and_padded_description_column_multi_line.0.join("\n")
        );
        assert_eq!(
            direct_matches_description_column_multi_line,
            sorted_and_padded_description_column_multi_line.1.join("\n")
        );
        assert_eq!(
            indirect_matches_description_column_multi_line,
            sorted_and_padded_description_column_multi_line.2.join("\n")
        );
    }

    #[test]
    fn test_color_matches() {
        init();

        let cli = cli::Cli::try_parse_from(vec!["nps", "-e=true", "mypackage"]).unwrap();
        let exact_matches = vec!["mypackage             v1     my package description".to_string()];
        let direct_matches = vec![
            "mypackage_extension   v1     my package description".to_string(),
            "mypackage_extension_2 v1.0.1 my package description".to_string(),
        ];
        let indirect_matches = vec![
            "mylastpackage         v5.0.0 is not mypackage".to_string(),
            "mylastpackage_2       v1     is not mypackage either".to_string(),
        ];

        let expect_color = [
            "\u{1b}[0m\u{1b}[1m\u{1b}[35mmypackage\u{1b}[0m             v1     my package description\n",
            "\u{1b}[0m\u{1b}[1m\u{1b}[34mmypackage\u{1b}[0m_extension   v1     my package description\n\
                \u{1b}[0m\u{1b}[1m\u{1b}[34mmypackage\u{1b}[0m_extension_2 v1.0.1 my package description\n",
            "mylastpackage         v5.0.0 is not \u{1b}[0m\u{1b}[1m\u{1b}[32mmypackage\u{1b}[0m\n\
                mylastpackage_2       v1     is not \u{1b}[0m\u{1b}[1m\u{1b}[32mmypackage\u{1b}[0m either\n",
        ];
        let expect_no_color = [
            "mypackage             v1     my package description\n",
            "mypackage_extension   v1     my package description\n\
                mypackage_extension_2 v1.0.1 my package description\n",
            "mylastpackage         v5.0.0 is not mypackage\n\
                mylastpackage_2       v1     is not mypackage either\n",
        ];

        let matches = (exact_matches, direct_matches, indirect_matches);

        let colored_matches_color = color_matches(
            &cli.search_term,
            &cli.exact_color,
            &cli.direct_color,
            &cli.indirect_color,
            cli.ignore_case,
            matches.clone(),
            termcolor::ColorChoice::Always,
        )
        .unwrap();
        let colored_matches_no_color = color_matches(
            &cli.search_term,
            &cli.exact_color,
            &cli.direct_color,
            &cli.indirect_color,
            cli.ignore_case,
            matches,
            termcolor::ColorChoice::Never,
        )
        .unwrap();

        for (expect, output) in std::iter::zip(
            [expect_color, expect_no_color].concat(),
            [colored_matches_color, colored_matches_no_color].concat(),
        ) {
            assert_eq!(expect, String::from_utf8(output.into_inner()).unwrap());
        }
    }
}
