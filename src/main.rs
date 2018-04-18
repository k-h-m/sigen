extern crate hound;
extern crate clap;

use std::f32::consts::PI;
use std::i16;
use clap::{App, Arg, SubCommand};

const MAX_AMPL: f32 = i16::MAX as f32;

fn gen_sine(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    (2.0 * PI * x).sin()
}

fn gen_square(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    if x < 0.5 {1.0} else {-1.0}
}

fn gen_saw(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    2.0 * x - 1.0
}

fn gen_triangle(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    if x < 0.5 {1.0 - 4.0 * x} else {4.0 * x - 3.0}
}

fn gen_silence(x: f32) -> f32 {
    assert!(x >= 0.0 && x < 1.0);
    0.0
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

fn plain(file: &str, rate: u32, dur: f32, freq: f32,
         phase: f32, shape: fn(f32) -> f32) -> Result<(), hound::Error> {
  let wav_spec: hound::WavSpec = hound::WavSpec {
    channels: 2,
    sample_rate: rate,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int
  };
  let mut writer = hound::WavWriter::create(file, wav_spec)?;
  for (l,r) in Tick::new(dur, rate as f32, freq, 0.0, phase) {
      writer.write_sample((MAX_AMPL * shape(l)) as i16)?;
      writer.write_sample((MAX_AMPL * shape(r)) as i16)?;
  }
  Ok(())
}

fn combo(file: &str, rate: u32, dur1: f32, dur2: f32,
         freq: f32, shift: f32, shape1: fn(f32) -> f32,
         shape2: fn(f32) -> f32) -> Result<(), hound::Error> {
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
        writer.write_sample((MAX_AMPL * shape1(l)) as i16)?;
        writer.write_sample((MAX_AMPL * shape1(r)) as i16)?;
    }
    for (l,r) in Tick::new(dur2, rate as f32, freq, 0.0, shift * (n as f32)) {
        writer.write_sample((MAX_AMPL * shape2(l)) as i16)?;
        writer.write_sample((MAX_AMPL * shape2(r)) as i16)?;
    }
  }
  Ok(())
}

fn modulate(file: &str, rate: u32, dur: f32, freq1: f32,
            freq2: f32, shape1: fn(f32) -> f32,
            shape2: fn(f32) -> f32) -> Result<(), hound::Error> {
  let wav_spec: hound::WavSpec = hound::WavSpec {
    channels: 2,
    sample_rate: rate,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int
  };
  let mut writer = hound::WavWriter::create(file, wav_spec)?;
  let s1 = Tick::new(dur, rate as f32, freq1, 0.0, 0.0);
  let s2 = Tick::new(dur, rate as f32, freq2, 0.0, 0.0);
  for ((l1,r1),(l2,r2)) in s1.zip(s2) {
    writer.write_sample((MAX_AMPL * shape1(l1) * shape2(l2)) as i16)?;
    writer.write_sample((MAX_AMPL * shape1(r1) * shape2(r2)) as i16)?;
  }
  Ok(())
}

fn parse_shape(shape: &str) -> Option<fn(f32)->f32> {
  match shape {
    "sine" => Some(gen_sine),
    "square" => Some(gen_square),
    "triangle" => Some(gen_triangle),
    "saw" => Some(gen_saw),
    _ => None
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
    .subcommand(SubCommand::with_name("modulate")
      .about("generate modulated wave")
      .arg(Arg::with_name("RATE")
        .help("sample rate in Hz")
        .required(true)
        .index(1))
      .arg(Arg::with_name("DURATION")
        .help("signal duration in Sec")
        .required(true)
        .index(2))
      .arg(Arg::with_name("FREQ1")
        .help("first frequency in Hz")
        .required(true)
        .index(3))
      .arg(Arg::with_name("SHAPE1")
        .help("first shape: sine, square, saw, triangle")
        .required(true)
        .index(4))
      .arg(Arg::with_name("FREQ2")
        .help("second frequency in Hz")
        .required(true)
        .index(5))
      .arg(Arg::with_name("SHAPE2")
        .help("second shape: sine, square, saw, triangle")
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
    let shape = parse_shape(matches.value_of("SHAPE").unwrap()).expect("Invalid value of SHAPE");
    let file = matches.value_of("OUTPUT").unwrap();
    plain(file, rate, dur, freq, phase, shape).unwrap();
  }
  else if let Some(matches) = matches.subcommand_matches("combo") {
    let rate = matches.value_of("RATE").unwrap().parse::<u32>().expect("Invalid value of RATE");
    let freq = matches.value_of("FREQ").unwrap().parse::<f32>().expect("Invalid value of FREQ");
    let dur = matches.value_of("DURATION").unwrap().parse::<f32>().expect("Invalid value of DURATION");
    let sil = matches.value_of("SILENCE").unwrap().parse::<f32>().expect("Invalid value of SILENCE");
    let phase = matches.value_of("PHASE").unwrap().parse::<f32>().expect("Invalid value of PHASE");
    let shape = parse_shape(matches.value_of("SHAPE").unwrap()).expect("Invalid value of SHAPE");
    let file = matches.value_of("OUTPUT").unwrap();
    combo(file, rate, dur, sil, freq, phase, shape, gen_silence).unwrap();
  }
  else if let Some(matches) = matches.subcommand_matches("modulate") {
    let rate = matches.value_of("RATE").unwrap().parse::<u32>().expect("Invalid value of RATE");
    let dur = matches.value_of("DURATION").unwrap().parse::<f32>().expect("Invalid value of DURATION");
    let freq1 = matches.value_of("FREQ1").unwrap().parse::<f32>().expect("Invalid value of FREQ1");
    let freq2 = matches.value_of("FREQ2").unwrap().parse::<f32>().expect("Invalid value of FREQ2");
    let shape1 = parse_shape(matches.value_of("SHAPE1").unwrap()).expect("Invalid value of SHAPE1");
    let shape2 = parse_shape(matches.value_of("SHAPE2").unwrap()).expect("Invalid value of SHAPE2");
    let file = matches.value_of("OUTPUT").unwrap();
    modulate(file, rate, dur, freq1, freq2, shape1, shape2).unwrap();
  }
  else {
    panic!("Invalid subcommand")
  }
}

