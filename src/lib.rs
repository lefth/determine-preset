use std::{cmp::max, collections::HashMap, io::{stderr, Write}, process::exit};

use atty;
use colored::Colorize;
use clap::{ArgAction, Parser};

#[derive(Default, Clone, Debug, clap::ValueEnum)]
enum ColorMode {
    #[default]
    Auto,
    Always,
    Never,
}

#[derive(Parser, Default)]
/// Read x265 encoding flags (for example from the output of `mediainfo`),
/// and print which preset the video was encoded with.
pub struct Cli {
    /// Path to read the encoding flags from. If omitted, read from STDIN.
    pub input: Option<String>,

    /// In the case of no match, colors are used to show close matches
    /// in verbose mode.
    #[arg(short, long, num_args(0..=1), default_value = "auto", default_missing_value = "auto")]
    color: ColorMode,

    /// In the case of no match, print detailed output about the close matches. -vv gives
    /// more detailed output.
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,
}

#[derive(Default)]
pub struct Determiner {
    cli: Cli,
    presets: Vec<(String, HashMap<String, String>)>,
}

impl Determiner {
    pub fn new(cli: Cli) -> Determiner {
        Determiner {
            cli,
            // Preset configurations from: https://x265.readthedocs.io/en/master/presets.html
            presets: vec![
                (
                    "ultrafast".to_string(),
                    parse_string("ctu=32 min-cu-size=16 bframes=3 b-adapt=0 rc-lookahead=5 lookahead-slices=8 scenecut=0 ref=1 limit-refs=0 me=dia merange=57 subme=0 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=1 recursion-skip=1 fast-intra=1 b-intra=0 sao=0 signhide=0 weightp=0 weightb=0 aq-mode=0 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
                ),
                (
                    "superfast".to_string(),
                    parse_string("ctu=32 min-cu-size=8 bframes=3 b-adapt=0 rc-lookahead=10 lookahead-slices=8 scenecut=40 ref=1 limit-refs=0 me=hex merange=57 subme=1 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=1 recursion-skip=1 fast-intra=1 b-intra=0 sao=0 signhide=1 weightp=0 weightb=0 aq-mode=0 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
                ),
                (
                    "veryfast".to_string(),
                    parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=0 rc-lookahead=15 lookahead-slices=8 scenecut=40 ref=2 limit-refs=3 me=hex merange=57 subme=1 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=1 recursion-skip=1 fast-intra=1 b-intra=0 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
                ),
                (
                    "faster".to_string(),
                    parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=0 rc-lookahead=15 lookahead-slices=8 scenecut=40 ref=2 limit-refs=3 me=hex merange=57 subme=2 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=1 recursion-skip=1 fast-intra=1 b-intra=0 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
                ),
                (
                    "fast".to_string(),
                    parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=0 rc-lookahead=15 lookahead-slices=8 scenecut=40 ref=3 limit-refs=3 me=hex merange=57 subme=2 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=0 recursion-skip=1 fast-intra=1 b-intra=0 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
                ),
                (
                    "medium".to_string(),
                    parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=2 rc-lookahead=20 lookahead-slices=8 scenecut=40 ref=3 limit-refs=1 me=hex merange=57 subme=2 rect=0 amp=0 limit-modes=0 max-merge=3 early-skip=1 recursion-skip=1 fast-intra=0 b-intra=1 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=3 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
                ),
                (
                    // Note: these are not stable/unchanging. I saw a "slow" video with lookahead-slices=6. I'm not sure which version was used to encode it.
                    "slow".to_string(),
                    parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=2 rc-lookahead=25 lookahead-slices=4 scenecut=40 ref=4 limit-refs=3 me=star merange=57 subme=3 rect=1 amp=0 limit-modes=1 max-merge=3 early-skip=0 recursion-skip=1 fast-intra=0 b-intra=0 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=4 rdoq-level=2 tu-intra=1 tu-inter=1 limit-tu=0"),
                ),
                (
                    "slower".to_string(),
                    parse_string("ctu=64 min-cu-size=8 bframes=8 b-adapt=2 rc-lookahead=40 lookahead-slices=1 scenecut=40 ref=5 limit-refs=1 me=star merange=57 subme=4 rect=1 amp=1 limit-modes=1 max-merge=4 early-skip=0 recursion-skip=1 fast-intra=0 b-intra=1 sao=1 signhide=1 weightp=1 weightb=1 aq-mode=2 cuTree=1 rdLevel=6 rdoq-level=2 tu-intra=3 tu-inter=3 limit-tu=4"),
                ),
                (
                    "veryslow".to_string(),
                    parse_string("ctu=64 min-cu-size=8 bframes=8 b-adapt=2 rc-lookahead=40 lookahead-slices=1 scenecut=40 ref=5 limit-refs=0 me=star merange=57 subme=4 rect=1 amp=1 limit-modes=0 max-merge=5 early-skip=0 recursion-skip=1 fast-intra=0 b-intra=1 sao=1 signhide=1 weightp=1 weightb=1 aq-mode=2 cuTree=1 rdLevel=6 rdoq-level=2 tu-intra=3 tu-inter=3 limit-tu=0"),
                ),
                (
                    "placebo".to_string(),
                    parse_string("ctu=64 min-cu-size=8 bframes=8 b-adapt=2 rc-lookahead=60 lookahead-slices=1 scenecut=40 ref=5 limit-refs=0 me=star merange=92 subme=5 rect=1 amp=1 limit-modes=0 max-merge=5 early-skip=0 recursion-skip=0 fast-intra=0 b-intra=1 sao=1 signhide=1 weightp=1 weightb=1 aq-mode=2 cuTree=1 rdLevel=6 rdoq-level=2 tu-intra=4 tu-inter=4 limit-tu=0"),
                ),
            ],
        }
    }

    pub fn print_preset_from_str(&self, input: &str) {
        match self.determine_preset_from_str(input) {
            Ok(preset_name) => println!("{}", preset_name),
            Err(error_message) => {
                writeln!(stderr(), "Error: {}", error_message).expect("Failed to write to stderr");
                exit(1);
            }
        }
    }

    fn determine_preset_from_str(&self, input: &str) -> Result<String, String> {
        // Parse the input into a HashMap of key-value pairs.
        let mut encoder_settings = parse_string(&input);
        let _ = encoder_settings.remove("me"); // the video has this in numeric format but the reference data is strings.

        // for lookahead-slices, 0 is the same as 1, but the reference table uses 1, not 0.
        match encoder_settings.get_mut("lookahead-slices") {
            Some(k) if *k == "0" => *k = "1".to_string(),
            _ => (),
        }

        // Determine the preset by matching the settings.
        self.determine_preset(&encoder_settings)
    }

    /// Gives output for the candidate matches to be compared visually:
    ///
    ///                  | input | slow | veryslow | placebo
    /// ----------------------------------------------------
    /// merange          | 57    | 57   | 57       | 92
    /// aq-mode          | 4     | 2    | 2        | 2
    /// subme            | 3     | 3    | 4        | 5
    /// b-adapt          | 2     | 2    | 2        | 2
    ///
    pub fn partially_matching_presets(&self, settings: &HashMap<String, String>) -> String {
        let use_color = match self.cli.color {
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
            ColorMode::Always => true,
            ColorMode::Never => false,
        };

        fn width_of_values(iter: impl Iterator<Item = impl AsRef<str>>) -> usize {
            let mut max_len = None;
            for val in iter {
                if max_len.map_or(true, |max_len| val.as_ref().len() > max_len) {
                    max_len.replace(val.as_ref().len());
                }
            }
            max_len.expect("One or more element required")
        }

        let preset_enc_params = self.presets.iter().map(|(_, params)| params).next().expect("There must be a preset").iter().map(|(param_name, _)| param_name).collect::<Vec<_>>();
        let input_keys = settings.iter().map(|(param_name, _)| param_name).collect::<Vec<_>>();
        // Filter these to not contain keys that don't match:
        let preset_enc_params = if self.cli.verbose < 2 {
            preset_enc_params.into_iter().filter(|param_name| input_keys.contains(param_name)).collect::<Vec<_>>()
        } else {
            preset_enc_params
        };
        let input_keys = if self.cli.verbose < 2 {
            input_keys.into_iter().filter(|param_name| preset_enc_params.contains(param_name)).collect::<Vec<_>>()
        } else {
            input_keys
        };

        // manipulate the data into more convenient formats:
        let preset_names = self.closest_matches(settings).into_iter().map(|(preset_name, _)| preset_name).take(3).collect::<Vec<_>>();
        // filter by which presets to show
        let presets = self.presets.iter().filter(|(preset_name, _)| preset_names.contains(preset_name)).collect::<Vec<_>>();
        // filter the encoding params within
        let presets = presets.iter().map(|(preset_name, encoder_values)| {
            let encoder_values = encoder_values.iter().filter(|(param_name, _)| {
                preset_enc_params.contains(param_name)
            }).collect::<HashMap<_, _>>();
            (preset_name, encoder_values)
        }).collect::<HashMap<_, _>>();
        let settings = settings.iter().filter(|(param_name, _)| input_keys.contains(param_name)).collect::<HashMap<_, _>>();

        // Find the widths for padding:
        let width_of_parameters = width_of_values(settings.keys());
        let width_of_input_values = max("input".len(), width_of_values(settings.values()));

        // width of the values, not the keys:
        let width_per_preset= presets.iter().map(|(preset_name, values)| {
            let width = max(preset_name.len(), values.values().map(|v| v.len()).max().expect("Preset must have values"));
            (preset_name, width)
        }).collect::<HashMap<_, _>>();

        fn add_finished_row(row: &mut String, table: &mut String) {
            row.push('\n');
            table.push_str(&row);
            row.clear();
        }

        let mut table = String::with_capacity(10_000);
        let mut row = String::with_capacity(200);
        row.push_str(&" ".repeat(width_of_parameters));
        row.push_str(" | ");
        row.push_str("input");
        let padding = width_of_input_values - "input".len();
        row.push_str(&" ".repeat(padding));

        for preset_name in presets.keys() {
            row.push_str(" | ");
            row.push_str(preset_name);
            let padding = width_per_preset[preset_name] - preset_name.len();
            row.push_str(&" ".repeat(padding));
        }
        add_finished_row(&mut row, &mut table);
        row.push_str(&"-".repeat(table.len() - 1));
        add_finished_row(&mut row, &mut table);


        let default = &"-".to_string();
        for encoder_param in settings.keys() {
            // print the parameter first
            let value = encoder_param;
            row.push_str(value);
            let padding = width_of_parameters - value.len();
            row.push_str(&" ".repeat(padding));

            // print the found encoded parameter value next
            row.push_str(" | ");
            let value = settings.get(encoder_param).unwrap_or(&default);
            let padding = width_of_input_values - value.len(); // calculate before adding color sequences
            let value = if use_color {
                &value.bold().to_string()
            } else {
                value
            };
            row.push_str(value);
            row.push_str(&" ".repeat(padding));

            for (preset_name, preset_values) in presets.iter() {
                row.push_str(" | ");
                let value = preset_values.get(encoder_param).unwrap_or(&default);
                let padding = width_per_preset[preset_name] - value.len(); // calculate before adding color sequences
                let is_match = settings.get(encoder_param) == Some(value);
                let value = if is_match && use_color {
                    value.green().to_string()
                } else {
                    value.to_string()
                };

                row.push_str(value.as_str());
                row.push_str(&" ".repeat(padding));
            }

            add_finished_row(&mut row, &mut table);
        }

        table
    }

    pub fn closest_matches(&self, settings: &HashMap<String, String>) -> Vec<(String, usize)> {
        let mut matches = self.presets.iter().map(|(preset, preset_settings)| {
            let match_count = settings
                .iter()
                .filter(|(key, value)| {
                    preset_settings.get(*key) == Some(value)
                })
                .count();
            (preset.to_owned(), match_count)
        }).collect::<Vec<_>>();
        matches.sort_by_key(|(_, match_count)| *match_count);
        matches.reverse();
        matches
    }

    /// Determines which x265 preset matches the given encoder parameters.
    pub fn determine_preset(&self, settings: &HashMap<String, String>) -> Result<String, String> {
        // Collect all matching presets.
        let matching_presets = self.presets
            .iter()
            .filter(|(_, preset_settings)| self.preset_matches(settings, preset_settings))
            .map(|(name, _)| name.to_owned())
            .collect::<Vec<_>>();

        // Handle the results of the matching.
        match matching_presets.len() {
            0 if self.cli.verbose > 0 => Err(format!("No matching presets found. Partial matches:\n\n{}", self.partially_matching_presets(settings))),
            0 => Err(format!("No matching presets found. Closest matches:\n:{:?}", self.closest_matches(settings))),
            1 => Ok(matching_presets[0].to_string()),
            _ => Err(format!(
                "Multiple matching presets found: {:?}",
                matching_presets
            )),
        }
    }

    /// Checks if the given `input_settings` match a preset's settings.
    ///
    /// The preset matches if all key-value pairs in `input_settings` are present in `preset_settings`.
    pub fn preset_matches(&self,
        input_settings: &HashMap<String, String>,
        preset_settings: &HashMap<String, String>,
    ) -> bool {
        input_settings
            .iter()
            .all(|(key, value)| {
                let preset_value = preset_settings.get(key);
                preset_value == None || preset_value == Some(value)
            })
    }
}

fn parse_string(input: &str) -> HashMap<String, String> {
    input
        .split_whitespace()
        .filter_map(|pair| {
            let mut parts = pair.split('=');
            Some((parts.next()?.to_string(), parts.next()?.to_string()))
        })
        .collect()
}

#[test]
fn test_encoding_params() {
    let input = "Encoding settings                        : cpuid=1111039 / frame-threads=4 / wpp / no-pmode / no-pme / no-psnr / no-ssim / log-level=2 / input-csp=1 / input-res=1860x1080 / interlace=0 / total-frames=0 / level-idc=0 / high-tier=1 / uhd-bd=0 / ref=5 / no-allow-non-conformance / no-repeat-headers / annexb / no-aud / no-eob / no-eos / no-hrd / info / hash=0 / temporal-layers=0 / open-gop / min-keyint=25 / keyint=250 / gop-lookahead=0 / bframes=8 / b-adapt=2 / b-pyramid / bframe-bias=0 / rc-lookahead=40 / lookahead-slices=0 / scenecut=40 / no-hist-scenecut / radl=0 / no-splice / no-intra-refresh / ctu=64 / min-cu-size=8 / rect / amp / max-tu-size=32 / tu-inter-depth=3 / tu-intra-depth=3 / limit-tu=0 / rdoq-level=2 / dynamic-rd=0.00 / no-ssim-rd / signhide / no-tskip / nr-intra=0 / nr-inter=0 / no-constrained-intra / strong-intra-smoothing / max-merge=5 / limit-refs=0 / no-limit-modes / me=3 / subme=4 / merange=57 / temporal-mvp / no-frame-dup / no-hme / weightp / weightb / no-analyze-src-pics / deblock=0:0 / sao / no-sao-non-deblock / rd=6 / selective-sao=4 / no-early-skip / rskip / no-fast-intra / no-tskip-fast / no-cu-lossless / b-intra / no-splitrd-skip / rdpenalty=0 / psy-rd=2.00 / psy-rdoq=1.00 / no-rd-refine / no-lossless / cbqpoffs=0 / crqpoffs=0 / rc=crf / crf=23.0 / qcomp=0.60 / qpstep=4 / stats-write=0 / stats-read=0 / ipratio=1.40 / pbratio=1.30 / aq-mode=2 / aq-strength=1.00 / cutree / zone-count=0 / no-strict-cbr / qg-size=32 / no-rc-grain / qpmax=69 / qpmin=0 / no-const-vbv / sar=0 / overscan=0 / videoformat=5 / range=0 / colorprim=1 / transfer=1 / colormatrix=1 / chromaloc=1 / chromaloc-top=0 / chromaloc-bottom=0 / display-window=0 / cll=0,0 / min-luma=0 / max-luma=1023 / log2-max-poc-lsb=8 / vui-timing-info / vui-hrd-info / slices=1 / no-opt-qp-pps / no-opt-ref-list-length-pps / no-multi-pass-opt-rps / scenecut-bias=0.05 / no-opt-cu-delta-qp / no-aq-motion / no-hdr10 / no-hdr10-opt / no-dhdr10-opt / no-idr-recovery-sei / analysis-reuse-level=0 / analysis-save-reuse-level=0 / analysis-load-reuse-level=0 / scale-factor=0 / refine-intra=0 / refine-inter=0 / refine-mv=1 / refine-ctu-distortion=0 / no-limit-sao / ctu-info=0 / no-lowpass-dct / refine-analysis-type=0 / copy-pic=1 / max-ausize-factor=1.0 / no-dynamic-refine / no-single-sei / no-hevc-aq / no-svt / no-field / qp-adaptation-range=1.00 / scenecut-aware-qp=0conformance-window-offsets / right=0 / bottom=0 / decoder-max-rate=0 / no-vbv-live-multi-pass / no-mcstf / no-sbrc";
    assert_eq!(Determiner::default().determine_preset_from_str(input), Ok("veryslow".to_string()));
    let input = "ctu=32 min-cu-size=8";
    assert_eq!(Determiner::default().determine_preset_from_str(input), Ok("superfast".to_string()));
    let input = "ctu=32 min-cu-size=8 bframes=8";
    assert_eq!(Determiner::default().determine_preset_from_str(input), Err("No matching presets found. Closest matches:\n:[(\"placebo\", 2), (\"veryslow\", 2), (\"slower\", 2), (\"superfast\", 2), (\"slow\", 1), (\"medium\", 1), (\"fast\", 1), (\"faster\", 1), (\"veryfast\", 1), (\"ultrafast\", 1)]".to_string()));
    let input = "ctu=32";
    assert_eq!(Determiner::default().determine_preset_from_str(input), Err("Multiple matching presets found: [\"ultrafast\", \"superfast\"]".to_string()));
}