use std::str::FromStr;

use clap::{App, Arg};
use failure::Error;

mod svg;
mod model;
mod raster;

const SAMPLE_RATE: u32 = 44100;

const X: u16 = std::u16::MAX / 256;

fn main() -> Result<(), Error> {
    let args = App::new("oscillator")
        .version("0.0")
        .author("Dustin Frisch <fooker@lab.sh>")
        .about("Renders SVGs into a WAV for display on XY-Oscilloscopes")
        .arg(Arg::with_name("verbose")
            .long("verbose")
            .short("v")
            .multiple(true)
            .help("increase log verbosity"))
        .arg(Arg::with_name("framerate")
            .long("framerate")
            .short("f")
            .default_value("23")
            .help("framerate to render"))
        .arg(Arg::with_name("width")
            .long("width")
            .short("w")
            .default_value("512")
            .help("SVG width"))
        .arg(Arg::with_name("height")
            .long("height")
            .short("h")
            .default_value("512")
            .help("SVG height"))
        .arg(Arg::with_name("flip-x")
            .long("flip-x")
            .help("Flip X axis"))
        .arg(Arg::with_name("flip-y")
            .long("flip-y")
            .help("Flip Y axis"))
        .arg(Arg::with_name("fuzziness")
            .long("fuzziness")
            .value_name("FUZZ")
            .help("fuzziness for deduplication"))
        .arg(Arg::with_name("pattern")
            .index(1)
            .default_value("*.svg")
            .help("pattern matching the SVG files to render"))
        .arg(Arg::with_name("output")
            .index(2)
            .default_value("output.wav")
            .help("render the output to this file"))
        .get_matches();

    simple_logger::init_with_level(match args.occurrences_of("verbose") {
        0 => log::Level::Warn,
        1 => log::Level::Info,
        2 => log::Level::Debug,
        _ => log::Level::Trace,
    }).unwrap();

    // Open the WAV file for writing
    let mut wav = hound::WavWriter::create(args.value_of("output").unwrap(),
                                           hound::WavSpec {
                                               channels: 2,
                                               sample_rate: SAMPLE_RATE,
                                               bits_per_sample: 16,
                                               sample_format: hound::SampleFormat::Int,
                                           })?;

    let size = (
        f32::from_str(args.value_of("width").unwrap())?,
        f32::from_str(args.value_of("height").unwrap())?,
    );

    let flip_x = args.is_present("flip-x");
    let flip_y = args.is_present("flip-y");

    // The number of rasterized coordinates is equals to the number samples required to render the
    // frame once. The samples must be stretched inside a frame to nearly fill a frame.
    let samples_per_frame = SAMPLE_RATE / u32::from_str(args.value_of("framerate").unwrap())?;

    for _ in 0..23 {
        for path in glob::glob(args.value_of("pattern").unwrap())? {
            let path = path?;
            if !path.is_file() {
                log::warn!("Skipping '{:?}': not a file", path);
                continue;
            }

            log::info!("Processing '{:?}'", path);

            // Get path segments from SVG file
            let segments = svg::parse_file(&path, size)?;
            log::debug!("Found {} path segments", segments.len());

            // Scale path segments down
            let segments = segments.into_iter()
                .map(|segment| segment / X)
                .collect::<Vec<_>>();

            // Deduplicate path segments
            let segments = args.value_of("fuzziness")
                .map(u16::from_str).transpose()?
                .map(|fuzziness| {
                    let deduped = model::dedup_segments(&segments, fuzziness);
                    log::debug!("Removed {} segments by deduplication", segments.len() - deduped.len());
                    return deduped;
                }).unwrap_or(segments);

            // Rasterize lines into a set of discrete coordinates
            let pixels = segments.into_iter()
                .flat_map(|segment| raster::rasterize(segment))
                .collect::<Vec<_>>();

            // Calculate how often each pixel must be repeated to stretch the frame to the frame length
            let repetitions = samples_per_frame / (pixels.len() as u32);
            dbg!(pixels.len());
            dbg!(samples_per_frame);

            // Write the samples of the frame to the WAV file by converting the u16 to i16 by shifting
            // the zero-point to the center
            for pixel in pixels {
                let pixel = pixel * X;
                for _ in 0..repetitions {
                    wav.write_sample(if flip_x { -pixel.x } else { pixel.x })?;
                    wav.write_sample(if flip_y { -pixel.y } else { pixel.y })?;
                }
            }
        }
    }

    wav.flush()?;

    return Ok(());
}
