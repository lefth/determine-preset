use std::{collections::HashMap, io::{stderr, Write}, process::exit};

fn parse_string(input: &str) -> HashMap<&str, &str> {
    input
        .split_whitespace()
        .filter_map(|pair| {
            let mut parts = pair.split('=');
            Some((parts.next()?, parts.next()?))
        })
        .collect()
}

pub fn print_preset_from_str(input: &str) {
    match determine_preset_from_str(input) {
        Ok(preset_name) => println!("{}", preset_name),
        Err(error_message) => {
            stderr().write_all(format!("Error: {}\n", error_message).as_bytes()).expect("Failed to write to stderr");
            exit(1);
        }
    }
}

fn determine_preset_from_str(input: &str) -> Result<&str, String> {
    // Parse the input into a HashMap of key-value pairs.
    let mut encoder_settings = parse_string(&input);
    let _ = encoder_settings.remove("me"); // the video has this in numeric format but the reference data is strings.

    // for lookahead-slices, 0 is the same as 1, but the reference table uses 1, not 0.
    match encoder_settings.get_mut("lookahead-slices") {
        Some(k) if *k == "0" => *k = "1",
        _ => (),
    }

    // Determine the preset by matching the settings.
    return determine_preset(&encoder_settings);
}

/// Determines which x265 preset matches the given encoder parameters.
///
/// Returns:
/// - `Ok(&'static str)` if exactly one preset matches.
/// - `Err(String)` if no presets match or if multiple presets match.
pub fn determine_preset(settings: &HashMap<&str, &str>) -> Result<&'static str, String> {
    // Preset configurations from: https://x265.readthedocs.io/en/master/presets.html
    let presets: Vec<(&str, HashMap<&str, &str>)> = vec![
        (
            "ultrafast",
            parse_string("ctu=32 min-cu-size=16 bframes=3 b-adapt=0 rc-lookahead=5 lookahead-slices=8 scenecut=0 ref=1 limit-refs=0 me=dia merange=57 subme=0 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=1 recursion-skip=1 fast-intra=1 b-intra=0 sao=0 signhide=0 weightp=0 weightb=0 aq-mode=0 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
        ),
        (
            "superfast",
            parse_string("ctu=32 min-cu-size=8 bframes=3 b-adapt=0 rc-lookahead=10 lookahead-slices=8 scenecut=40 ref=1 limit-refs=0 me=hex merange=57 subme=1 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=1 recursion-skip=1 fast-intra=1 b-intra=0 sao=0 signhide=1 weightp=0 weightb=0 aq-mode=0 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
        ),
        (
            "veryfast",
            parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=0 rc-lookahead=15 lookahead-slices=8 scenecut=40 ref=2 limit-refs=3 me=hex merange=57 subme=1 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=1 recursion-skip=1 fast-intra=1 b-intra=0 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
        ),
        (
            "faster",
            parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=0 rc-lookahead=15 lookahead-slices=8 scenecut=40 ref=2 limit-refs=3 me=hex merange=57 subme=2 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=1 recursion-skip=1 fast-intra=1 b-intra=0 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
        ),
        (
            "fast",
            parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=0 rc-lookahead=15 lookahead-slices=8 scenecut=40 ref=3 limit-refs=3 me=hex merange=57 subme=2 rect=0 amp=0 limit-modes=0 max-merge=2 early-skip=0 recursion-skip=1 fast-intra=1 b-intra=0 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=2 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
        ),
        (
            "medium",
            parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=2 rc-lookahead=20 lookahead-slices=8 scenecut=40 ref=3 limit-refs=1 me=hex merange=57 subme=2 rect=0 amp=0 limit-modes=0 max-merge=3 early-skip=1 recursion-skip=1 fast-intra=0 b-intra=1 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=3 rdoq-level=0 tu-intra=1 tu-inter=1 limit-tu=0"),
        ),
        (
            // Note: these are not stable/unchanging. I saw a "slow" video with lookahead-slices=6. I'm not sure which version was used to encode it.
            "slow",
            parse_string("ctu=64 min-cu-size=8 bframes=4 b-adapt=2 rc-lookahead=25 lookahead-slices=4 scenecut=40 ref=4 limit-refs=3 me=star merange=57 subme=3 rect=1 amp=0 limit-modes=1 max-merge=3 early-skip=0 recursion-skip=1 fast-intra=0 b-intra=0 sao=1 signhide=1 weightp=1 weightb=0 aq-mode=2 cuTree=1 rdLevel=4 rdoq-level=2 tu-intra=1 tu-inter=1 limit-tu=0"),
        ),
        (
            "slower",
            parse_string("ctu=64 min-cu-size=8 bframes=8 b-adapt=2 rc-lookahead=40 lookahead-slices=1 scenecut=40 ref=5 limit-refs=1 me=star merange=57 subme=4 rect=1 amp=1 limit-modes=1 max-merge=4 early-skip=0 recursion-skip=1 fast-intra=0 b-intra=1 sao=1 signhide=1 weightp=1 weightb=1 aq-mode=2 cuTree=1 rdLevel=6 rdoq-level=2 tu-intra=3 tu-inter=3 limit-tu=4"),
        ),
        (
            "veryslow",
            parse_string("ctu=64 min-cu-size=8 bframes=8 b-adapt=2 rc-lookahead=40 lookahead-slices=1 scenecut=40 ref=5 limit-refs=0 me=star merange=57 subme=4 rect=1 amp=1 limit-modes=0 max-merge=5 early-skip=0 recursion-skip=1 fast-intra=0 b-intra=1 sao=1 signhide=1 weightp=1 weightb=1 aq-mode=2 cuTree=1 rdLevel=6 rdoq-level=2 tu-intra=3 tu-inter=3 limit-tu=0"),
        ),
        (
            "placebo",
            parse_string("ctu=64 min-cu-size=8 bframes=8 b-adapt=2 rc-lookahead=60 lookahead-slices=1 scenecut=40 ref=5 limit-refs=0 me=star merange=92 subme=5 rect=1 amp=1 limit-modes=0 max-merge=5 early-skip=0 recursion-skip=0 fast-intra=0 b-intra=1 sao=1 signhide=1 weightp=1 weightb=1 aq-mode=2 cuTree=1 rdLevel=6 rdoq-level=2 tu-intra=4 tu-inter=4 limit-tu=0"),
        ),
    ];

    // Collect all matching presets.
    let matching_presets: Vec<&str> = presets
        .iter()
        .filter(|(_, preset_settings)| preset_matches(settings, preset_settings))
        .map(|(name, _)| *name)
        .collect();

    // Handle the results of the matching.
    match matching_presets.len() {
        0 => Err("No matching presets found.".to_string()),
        1 => Ok(matching_presets[0]),
        _ => Err(format!(
            "Multiple matching presets found: {:?}",
            matching_presets
        )),
    }
}

/// Checks if the given `input_settings` match a preset's settings.
///
/// The preset matches if all key-value pairs in `input_settings` are present in `preset_settings`.
pub fn preset_matches(
    input_settings: &HashMap<&str, &str>,
    preset_settings: &HashMap<&str, &str>,
) -> bool {
    input_settings
        .iter()
        .all(|(key, value)| {
            let preset_value = preset_settings.get(key);
            preset_value == None || preset_value == Some(value)
        })
}

#[test]
fn test_encoding_params() {
    let input = "Encoding settings                        : cpuid=1111039 / frame-threads=4 / wpp / no-pmode / no-pme / no-psnr / no-ssim / log-level=2 / input-csp=1 / input-res=1860x1080 / interlace=0 / total-frames=0 / level-idc=0 / high-tier=1 / uhd-bd=0 / ref=5 / no-allow-non-conformance / no-repeat-headers / annexb / no-aud / no-eob / no-eos / no-hrd / info / hash=0 / temporal-layers=0 / open-gop / min-keyint=25 / keyint=250 / gop-lookahead=0 / bframes=8 / b-adapt=2 / b-pyramid / bframe-bias=0 / rc-lookahead=40 / lookahead-slices=0 / scenecut=40 / no-hist-scenecut / radl=0 / no-splice / no-intra-refresh / ctu=64 / min-cu-size=8 / rect / amp / max-tu-size=32 / tu-inter-depth=3 / tu-intra-depth=3 / limit-tu=0 / rdoq-level=2 / dynamic-rd=0.00 / no-ssim-rd / signhide / no-tskip / nr-intra=0 / nr-inter=0 / no-constrained-intra / strong-intra-smoothing / max-merge=5 / limit-refs=0 / no-limit-modes / me=3 / subme=4 / merange=57 / temporal-mvp / no-frame-dup / no-hme / weightp / weightb / no-analyze-src-pics / deblock=0:0 / sao / no-sao-non-deblock / rd=6 / selective-sao=4 / no-early-skip / rskip / no-fast-intra / no-tskip-fast / no-cu-lossless / b-intra / no-splitrd-skip / rdpenalty=0 / psy-rd=2.00 / psy-rdoq=1.00 / no-rd-refine / no-lossless / cbqpoffs=0 / crqpoffs=0 / rc=crf / crf=23.0 / qcomp=0.60 / qpstep=4 / stats-write=0 / stats-read=0 / ipratio=1.40 / pbratio=1.30 / aq-mode=2 / aq-strength=1.00 / cutree / zone-count=0 / no-strict-cbr / qg-size=32 / no-rc-grain / qpmax=69 / qpmin=0 / no-const-vbv / sar=0 / overscan=0 / videoformat=5 / range=0 / colorprim=1 / transfer=1 / colormatrix=1 / chromaloc=1 / chromaloc-top=0 / chromaloc-bottom=0 / display-window=0 / cll=0,0 / min-luma=0 / max-luma=1023 / log2-max-poc-lsb=8 / vui-timing-info / vui-hrd-info / slices=1 / no-opt-qp-pps / no-opt-ref-list-length-pps / no-multi-pass-opt-rps / scenecut-bias=0.05 / no-opt-cu-delta-qp / no-aq-motion / no-hdr10 / no-hdr10-opt / no-dhdr10-opt / no-idr-recovery-sei / analysis-reuse-level=0 / analysis-save-reuse-level=0 / analysis-load-reuse-level=0 / scale-factor=0 / refine-intra=0 / refine-inter=0 / refine-mv=1 / refine-ctu-distortion=0 / no-limit-sao / ctu-info=0 / no-lowpass-dct / refine-analysis-type=0 / copy-pic=1 / max-ausize-factor=1.0 / no-dynamic-refine / no-single-sei / no-hevc-aq / no-svt / no-field / qp-adaptation-range=1.00 / scenecut-aware-qp=0conformance-window-offsets / right=0 / bottom=0 / decoder-max-rate=0 / no-vbv-live-multi-pass / no-mcstf / no-sbrc";
    assert_eq!(determine_preset_from_str(input), Ok("veryslow"));
    let input = "ctu=32 min-cu-size=8";
    assert_eq!(determine_preset_from_str(input), Ok("superfast"));
    let input = "ctu=32 min-cu-size=8 bframes=8";
    assert_eq!(determine_preset_from_str(input), Err("No matching presets found.".to_string()));
    let input = "ctu=32";
    assert_eq!(determine_preset_from_str(input), Err("Multiple matching presets found: [\"ultrafast\", \"superfast\"]".to_string()));
}