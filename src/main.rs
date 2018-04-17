extern crate hound;
extern crate clap;

use std::f32::consts::PI;
use std::i16;
use clap::{App, Arg, SubCommand};


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

fn wrr<T: Gen>(writer: &mut hound::WavWriter<std::io::BufWriter<std::fs::File>>, dur: f32, freq: f32, phase1: f32, phase2: f32, rate: u32) -> () {
  let ampl = i16::MAX as f32;
  for (l,r) in Tick::new(dur, rate as f32, freq, phase1, phase2) {
      writer.write_sample((ampl * T::gen(l)) as i16).unwrap();
      writer.write_sample((ampl * T::gen(r)) as i16).unwrap();
  }
}

fn write_plain<T: Gen>(file: &str, dur: f32, freq: f32, phase: f32, rate: u32) -> () {
  let wav_spec: hound::WavSpec = hound::WavSpec {
    channels: 2,
    sample_rate: rate,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int
  };
  let mut writer = hound::WavWriter::create(file, wav_spec).unwrap();
  wrr::<T>(&mut writer, dur, freq, 0.0, phase, rate);
}

fn write_combo<T1: Gen, T2: Gen>(file:&str, dur1: f32, dur2: f32, freq:f32, shift: f32, rate: u32) -> () {
  assert!(shift > 0.0);
  let wav_spec: hound::WavSpec = hound::WavSpec {
    channels: 2,
    sample_rate: rate,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int
  };
  let mut writer = hound::WavWriter::create(file, wav_spec).unwrap();
  for n in 0 .. (360.0/shift) as usize {
    wrr::<T1>(&mut writer, dur1, freq, 0.0, shift * (n as f32), rate);
    wrr::<T2>(&mut writer, dur2, freq, 0.0, shift * (n as f32), rate);
  }
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
        write_plain::<Sine>(file, dur, freq, phase, rate),
      "square" =>
        write_plain::<Square>(file, dur, freq, phase, rate),
      "triangle" =>
        write_plain::<Triangle>(file, dur, freq, phase, rate),
      "saw" =>
        write_plain::<Saw>(file, dur, freq, phase, rate),
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
        write_combo::<Sine, Silence>(file, dur, sil, freq, phase, rate),
      "square" =>
        write_combo::<Square, Silence>(file, dur, sil, freq, phase, rate),
      "triangle" =>
        write_combo::<Triangle, Silence>(file, dur, sil, freq, phase, rate),
      "saw" =>
        write_combo::<Saw, Silence>(file, dur, sil, freq, phase, rate),
      _ =>
        panic!("Invalid value of SHAPE")
    }
  }
  else {panic!("Invalid subcommand")}
}

