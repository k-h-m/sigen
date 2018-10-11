extern crate hound;
#[macro_use]
extern crate clap;

use clap::{App, Arg, Error, ErrorKind, SubCommand};
use std::f32::consts::PI;
use std::i16;

const MAX_AMPL: f32 = i16::MAX as f32;

arg_enum!{
    enum Shape {
        Saw,
        Sine,
        Square,
        Triangle
    }
}

fn gen_sine(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    (2.0 * PI * x).sin()
}

fn gen_square(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    if x < 0.5 {
        1.0
    } else {
        -1.0
    }
}

fn gen_saw(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    2.0 * x - 1.0
}

fn gen_triangle(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    if x < 0.5 {
        1.0 - 4.0 * x
    } else {
        4.0 * x - 3.0
    }
}

fn gen_silence(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    0.0
}

fn shape2fn(shape: Shape) -> (fn(f32) -> f32) {
    match shape {
        Shape::Saw => gen_saw,
        Shape::Sine => gen_sine,
        Shape::Square => gen_square,
        Shape::Triangle => gen_triangle,
    }
}

struct Tick {
    curr_tick: u32,
    last_tick: u32,
    sample_rate: f32,
    freq: f32,
    left: f32,
    right: f32,
}

impl Tick {
    fn new(duration: f32, sample_rate: f32, freq: f32, phase_left: f32, phase_right: f32) -> Tick {
        assert!(duration >= 0.0);
        assert!(sample_rate > 0.0);
        assert!(freq > 0.0 && freq < sample_rate);
        assert!(phase_left >= 0.0 && phase_left < 360.0);
        assert!(phase_right >= 0.0 && phase_right < 360.0);
        Tick {
            curr_tick: 0,
            last_tick: (duration * sample_rate) as u32,
            left: phase_left * sample_rate / 360.0,
            right: phase_right * sample_rate / 360.0,
            sample_rate,
            freq,
        }
    }
}

impl Iterator for Tick {
    type Item = (f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_tick >= self.last_tick {
            return None;
        }
        self.curr_tick += 1;
        if self.left >= self.sample_rate {
            self.left -= self.sample_rate;
        }
        if self.right >= self.sample_rate {
            self.right -= self.sample_rate;
        }
        let l = self.left / self.sample_rate;
        let r = self.right / self.sample_rate;
        self.left += self.freq;
        self.right += self.freq;
        Some((l, r))
    }
}

fn plain(
    file: &str,
    rate: u32,
    dur: f32,
    freq: f32,
    phase: f32,
    shape: Shape,
) -> Result<(), hound::Error> {
    let wav_spec = hound::WavSpec {
        channels: 2,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let gen = shape2fn(shape);
    let mut writer = hound::WavWriter::create(file, wav_spec)?;
    for (l, r) in Tick::new(dur, rate as f32, freq, 0.0, phase) {
        writer.write_sample((MAX_AMPL * gen(l)) as i16)?;
        writer.write_sample((MAX_AMPL * gen(r)) as i16)?;
    }
    Ok(())
}

fn combo(
    file: &str,
    rate: u32,
    dur1: f32,
    dur2: f32,
    freq: f32,
    shift: f32,
    shape: Shape,
) -> Result<(), hound::Error> {
    assert!(shift > 0.0);
    let wav_spec = hound::WavSpec {
        channels: 2,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let gen = shape2fn(shape);
    let mut writer = hound::WavWriter::create(file, wav_spec)?;
    for n in 0..(360.0 / shift) as usize {
        for (l, r) in Tick::new(dur1, rate as f32, freq, 0.0, shift * (n as f32)) {
            writer.write_sample((MAX_AMPL * gen(l)) as i16)?;
            writer.write_sample((MAX_AMPL * gen(r)) as i16)?;
        }
        for (l, r) in Tick::new(dur2, rate as f32, freq, 0.0, shift * (n as f32)) {
            writer.write_sample((MAX_AMPL * gen_silence(l)) as i16)?;
            writer.write_sample((MAX_AMPL * gen_silence(r)) as i16)?;
        }
    }
    Ok(())
}

fn modulate(
    file: &str,
    rate: u32,
    dur: f32,
    freq1: f32,
    freq2: f32,
    shape1: Shape,
    shape2: Shape,
) -> Result<(), hound::Error> {
    let wav_spec: hound::WavSpec = hound::WavSpec {
        channels: 2,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let gen1 = shape2fn(shape1);
    let gen2 = shape2fn(shape2);
    let mut writer = hound::WavWriter::create(file, wav_spec)?;
    let s1 = Tick::new(dur, rate as f32, freq1, 0.0, 0.0);
    let s2 = Tick::new(dur, rate as f32, freq2, 0.0, 0.0);
    for ((l1, r1), (l2, r2)) in s1.zip(s2) {
        writer.write_sample((MAX_AMPL * gen1(l1) * gen2(l2)) as i16)?;
        writer.write_sample((MAX_AMPL * gen1(r1) * gen2(r2)) as i16)?;
    }
    Ok(())
}

fn main() {
    let matches = App::new("Signal generator")
        .version(crate_version!())
        .arg(
            Arg::with_name("rate")
                .short("r")
                .long("rate")
                .value_name("SAMPLE_RATE")
                .takes_value(true)
                .default_value("44100")
                .help("Sets a sample rate in Hz"),
        ).arg(
            Arg::with_name("OUTPUT")
                .help("name of output file")
                .required(true)
                .index(1),
        ).subcommand(
            SubCommand::with_name("plain")
                .about("Generates a plain wave")
                .arg(
                    Arg::with_name("FREQ")
                        .help("signal frequency in Hz")
                        .required(true)
                        .index(1),
                ).arg(
                    Arg::with_name("DURATION")
                        .help("signal duration in Sec")
                        .required(true)
                        .index(2),
                ).arg(
                    Arg::with_name("PHASE")
                        .help("phase shift in Degree")
                        .required(true)
                        .index(3),
                ).arg(
                    Arg::with_name("SHAPE")
                        .help("shape of signal")
                        .required(true)
                        .possible_values(&Shape::variants())
                        .index(4),
                ),
        ).subcommand(
            SubCommand::with_name("combo")
                .about("Generates a combo wave")
                .arg(
                    Arg::with_name("FREQ")
                        .help("signal frequency in Hz")
                        .required(true)
                        .index(1),
                ).arg(
                    Arg::with_name("DURATION")
                        .help("signal duration in Sec")
                        .required(true)
                        .index(2),
                ).arg(
                    Arg::with_name("SILENCE")
                        .help("silence duration in Sec")
                        .required(true)
                        .index(3),
                ).arg(
                    Arg::with_name("PHASE")
                        .help("phase shift in Degree")
                        .required(true)
                        .index(4),
                ).arg(
                    Arg::with_name("SHAPE")
                        .help("shape of signal")
                        .required(true)
                        .possible_values(&Shape::variants())
                        .index(5),
                ),
        ).subcommand(
            SubCommand::with_name("modulate")
                .about("Generates a modulated wave")
                .arg(
                    Arg::with_name("DURATION")
                        .help("signal duration in Sec")
                        .required(true)
                        .index(1),
                ).arg(
                    Arg::with_name("FREQ1")
                        .help("first frequency in Hz")
                        .required(true)
                        .index(2),
                ).arg(
                    Arg::with_name("SHAPE1")
                        .help("first shape")
                        .required(true)
                        .possible_values(&Shape::variants())
                        .index(3),
                ).arg(
                    Arg::with_name("FREQ2")
                        .help("second frequency in Hz")
                        .required(true)
                        .index(4),
                ).arg(
                    Arg::with_name("SHAPE2")
                        .help("second shape")
                        .required(true)
                        .possible_values(&Shape::variants())
                        .index(5),
                ),
        ).get_matches();

    let rate = value_t!(matches.value_of("rate"), u32).unwrap_or_else(|e| e.exit());
    let file = matches.value_of("OUTPUT").unwrap();

    if let Some(m) = matches.subcommand_matches("plain") {
        let freq = value_t!(m.value_of("FREQ"), f32).unwrap_or_else(|e| e.exit());
        let dur = value_t!(m.value_of("DURATION"), f32).unwrap_or_else(|e| e.exit());
        let phase = value_t!(m.value_of("PHASE"), f32).unwrap_or_else(|e| e.exit());
        let shape = value_t!(m.value_of("SHAPE"), Shape).unwrap_or_else(|e| e.exit());
        plain(file, rate, dur, freq, phase, shape).unwrap();
    } else if let Some(m) = matches.subcommand_matches("combo") {
        let freq = value_t!(m.value_of("FREQ"), f32).unwrap_or_else(|e| e.exit());
        let dur = value_t!(m.value_of("DURATION"), f32).unwrap_or_else(|e| e.exit());
        let sil = value_t!(m.value_of("SILENCE"), f32).unwrap_or_else(|e| e.exit());
        let phase = value_t!(m.value_of("PHASE"), f32).unwrap_or_else(|e| e.exit());
        let shape = value_t!(m.value_of("SHAPE"), Shape).unwrap_or_else(|e| e.exit());
        combo(file, rate, dur, sil, freq, phase, shape).unwrap();
    } else if let Some(m) = matches.subcommand_matches("modulate") {
        let dur = value_t!(m.value_of("DURATION"), f32).unwrap_or_else(|e| e.exit());
        let freq1 = value_t!(m.value_of("FREQ1"), f32).unwrap_or_else(|e| e.exit());
        let freq2 = value_t!(m.value_of("FREQ2"), f32).unwrap_or_else(|e| e.exit());
        let shape1 = value_t!(m.value_of("SHAPE1"), Shape).unwrap_or_else(|e| e.exit());
        let shape2 = value_t!(m.value_of("SHAPE2"), Shape).unwrap_or_else(|e| e.exit());
        modulate(file, rate, dur, freq1, freq2, shape1, shape2).unwrap();
    } else {
        Error::with_description("Invalid subcommnad", ErrorKind::InvalidSubcommand).exit()
    }
}
