extern crate hound;
extern crate clap;

use std::f32::consts::PI;
use std::i16;
use clap::{App, Arg, SubCommand};

const MAX_AMPL: f32 = i16::MAX as f32;

trait Gen {
    fn gen(f32) -> f32;
}

struct Sine {}

impl Gen for Sine {
    fn gen(x: f32) -> f32 {
        assert!(x >= 0.0 && x < 1.0);
        (2.0 * PI * x).sin()
    }
}

struct Square {}

impl Gen for Square {
    fn gen(x: f32) -> f32 {
        assert!(x >= 0.0 && x < 1.0);
        if x < 0.5 {1.0} else {-1.0}
    }
}

struct Saw {}

impl Gen for Saw {
    fn gen(x: f32) -> f32 {
        assert!(x >= 0.0 && x < 1.0);
        2.0 * x - 1.0
    }
}

struct Triangle {}

impl Gen for Triangle {
    fn gen(x: f32) -> f32 {
        assert!(x >= 0.0 && x < 1.0);
        if x < 0.5 {1.0 - 4.0 * x} else {4.0 * x - 3.0}
    }
}

struct Silence {}

impl Gen for Silence {
    fn gen(x: f32) -> f32 {
        assert!(x >= 0.0 && x < 1.0);
        0.0
    }
}

struct Tick {
    curr_tick: u32,
    last_tick: u32,
    sample_rate: f32,
    freq: f32,
    left: f32,
    right: f32
}

impl Tick {
    fn new(duration: f32, sample_rate: f32, freq: f32, phase_left: f32, phase_right: f32) -> Tick {
        assert!(duration >= 0.0);
        assert!(sample_rate > 0.0);
        assert!(freq > 0.0 && freq < sample_rate);
        assert!(phase_left >= 0.0 && phase_left < 360.0);
        assert!(phase_right >= 0.0 && phase_right < 360.0);
        Tick {  curr_tick: 0,
                last_tick: (duration * sample_rate) as u32,
                left: phase_left * sample_rate / 360.0,
                right: phase_right * sample_rate / 360.0,
                sample_rate, freq
        }
    }
}

impl Iterator for Tick {
    type Item = (f32,f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_tick >= self.last_tick {
            return None
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
        Some((l,r))
    }
}

fn plain<T: Gen>(file: &str, dur: f32, freq: f32, phase: f32, rate: u32) -> Result<(), hound::Error> {
  let wav_spec: hound::WavSpec = hound::WavSpec {
    channels: 2,
    sample_rate: rate,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int
  };
  let mut writer = hound::WavWriter::create(file, wav_spec)?;
  for (l,r) in Tick::new(dur, rate as f32, freq, 0.0, phase) {
      writer.write_sample((MAX_AMPL * T::gen(l)) as i16)?;
      writer.write_sample((MAX_AMPL * T::gen(r)) as i16)?;
  }
  Ok(())
}

fn combo<T1, T2>(file:&str, dur1: f32, dur2: f32, freq:f32, shift: f32, rate: u32) -> Result<(), hound::Error>
    where T1: Gen, T2: Gen {
  assert!(shift > 0.0);
  let wav_spec: hound::WavSpec = hound::WavSpec {
    channels: 2,
    sample_rate: rate,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int
  };
  let mut writer = hound::WavWriter::create(file, wav_spec)?;
  for n in 0 .. (360.0 / shift) as usize {
    for (l,r) in Tick::new(dur1, rate as f32, freq, 0.0, shift * (n as f32)) {
        writer.write_sample((MAX_AMPL * T1::gen(l)) as i16)?;
        writer.write_sample((MAX_AMPL * T1::gen(r)) as i16)?;
    }
    for (l,r) in Tick::new(dur2, rate as f32, freq, 0.0, shift * (n as f32)) {
        writer.write_sample((MAX_AMPL * T2::gen(l)) as i16)?;
        writer.write_sample((MAX_AMPL * T2::gen(r)) as i16)?;
    }
  }
  Ok(())
}

fn main() {
  let matches = App::new("Signal generator")
    .subcommand(SubCommand::with_name("plain")
      .about("generate plain wave")
      .arg(Arg::with_name("RATE")
        .help("sample rate in Hz")
        .required(true)
        .index(1))
      .arg(Arg::with_name("FREQ")
        .help("signal frequency in Hz")
        .required(true)
        .index(2))
      .arg(Arg::with_name("DURATION")
        .help("signal duration in Sec")
        .required(true)
        .index(3))
      .arg(Arg::with_name("PHASE")
        .help("phase shift in Degree")
        .required(true)
        .index(4))
      .arg(Arg::with_name("SHAPE")
        .help("signal shape: sine, square, saw, triangle")
        .required(true)
        .index(5))
      .arg(Arg::with_name("OUTPUT")
        .help("name of output file")
        .required(true)
        .index(6)))
    .subcommand(SubCommand::with_name("combo")
      .about("generate combo wave")
      .arg(Arg::with_name("RATE")
        .help("sample rate in Hz")
        .required(true)
        .index(1))
      .arg(Arg::with_name("FREQ")
        .help("signal frequency in Hz")
        .required(true)
        .index(2))
      .arg(Arg::with_name("DURATION")
        .help("signal duration in Sec")
        .required(true)
        .index(3))
      .arg(Arg::with_name("SILENCE")
        .help("silence duration in Sec")
        .required(true)
        .index(4))
      .arg(Arg::with_name("PHASE")
        .help("phase shift in Degree")
        .required(true)
        .index(5))
      .arg(Arg::with_name("SHAPE")
        .help("signal shape: sine, square, saw, triangle")
        .required(true)
        .index(6))
      .arg(Arg::with_name("OUTPUT")
        .help("name of output file")
        .required(true)
        .index(7)))
  .get_matches();

  if let Some(matches) = matches.subcommand_matches("plain") {
    let rate = matches.value_of("RATE").unwrap().parse::<u32>().expect("Invalid value of RATE");
    let freq = matches.value_of("FREQ").unwrap().parse::<f32>().expect("Invalid value of FREQ");
    let dur = matches.value_of("DURATION").unwrap().parse::<f32>().expect("Invalid value of DURATION");
    let phase = matches.value_of("PHASE").unwrap().parse::<f32>().expect("Invalid value of PHASE");
    let file = matches.value_of("OUTPUT").unwrap();
    match matches.value_of("SHAPE").unwrap() {
      "sine" =>
        plain::<Sine>(file, dur, freq, phase, rate).unwrap(),
      "square" =>
        plain::<Square>(file, dur, freq, phase, rate).unwrap(),
      "triangle" =>
        plain::<Triangle>(file, dur, freq, phase, rate).unwrap(),
      "saw" =>
        plain::<Saw>(file, dur, freq, phase, rate).unwrap(),
      _ =>
        panic!("Invalid value of SHAPE")
    }
  }
  else if let Some(matches) = matches.subcommand_matches("combo") {
    let rate = matches.value_of("RATE").unwrap().parse::<u32>().expect("Invalid value of RATE");
    let freq = matches.value_of("FREQ").unwrap().parse::<f32>().expect("Invalid value of FREQ");
    let dur = matches.value_of("DURATION").unwrap().parse::<f32>().expect("Invalid value of DURATION");
    let sil = matches.value_of("SILENCE").unwrap().parse::<f32>().expect("Invalid value of SILENCE");
    let phase = matches.value_of("PHASE").unwrap().parse::<f32>().expect("Invalid value of PHASE");
    let file = matches.value_of("OUTPUT").unwrap();
    match matches.value_of("SHAPE").unwrap() {
      "sine" =>
        combo::<Sine, Silence>(file, dur, sil, freq, phase, rate).unwrap(),
      "square" =>
        combo::<Square, Silence>(file, dur, sil, freq, phase, rate).unwrap(),
      "triangle" =>
        combo::<Triangle, Silence>(file, dur, sil, freq, phase, rate).unwrap(),
      "saw" =>
        combo::<Saw, Silence>(file, dur, sil, freq, phase, rate).unwrap(),
      _ =>
        panic!("Invalid value of SHAPE")
    }
  }
  else {panic!("Invalid subcommand")}
}

