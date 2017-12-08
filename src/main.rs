extern crate hound;
extern crate clap;

use std::f32::consts::PI;
use std::i16;
use clap::{App, Arg, SubCommand};


trait Gen {
    fn new(ampl: f32, freq: f32, phase: f32) -> Self;
    fn gen(&self, x: f32) -> i16;
}

struct Sine {
    ampl: f32,
    freq: f32,
    phase: f32
}

impl Gen for Sine {
    fn new(ampl: f32, freq: f32, phase: f32) -> Sine {
        Sine {ampl, freq, phase}
    }

    fn gen(&self, x: f32) -> i16 {
        let arg = 2.0 * PI * (self.freq * x + self.phase / 360.0);
        (self.ampl * arg.sin()) as i16
    }
}

struct Square {
    ampl: f32,
    freq: f32,
    phase: f32
}

impl Gen for Square {
    fn new(ampl: f32, freq: f32, phase: f32) -> Square {
        Square {ampl, freq, phase}
    }

    fn gen(&self, x: f32) -> i16 {
        let arg = (self.freq * x + self.phase / 360.0).fract();
        if arg < 0.5 {self.ampl as i16} else {-self.ampl as i16}
    }
}

struct Saw {
    ampl: f32,
    freq: f32,
    phase: f32
}

impl Gen for Saw {
    fn new(ampl: f32, freq: f32, phase: f32) -> Saw {
        Saw {ampl, freq, phase}
    }

    fn gen(&self, x: f32) -> i16 {
        let arg = (self.freq * x + self.phase / 360.0).fract();
        if arg < 0.5 {(self.ampl * (1.0 - 4.0 * arg)) as i16}
        else {(self.ampl * (4.0 * arg - 3.0)) as i16}
    }
}

struct Silence {}

impl Gen for Silence {
    fn new(_ampl: f32, _freq: f32, _phase: f32) -> Silence {
        Silence {}
    }

    fn gen(&self, _x: f32) -> i16 {
        0
    }
}

struct Ticks<T: Gen> {
    curr_tick: u32,
    last_tick: u32,
    step: f32,
    generator: T
}

impl<T: Gen> Ticks<T> {
    fn new(duration: f32, sample_rate: f32, generator: T) -> Ticks<T> {
        assert!(duration >= 0.0);
        assert!(sample_rate > 0.0);
        Ticks { curr_tick: 0,
                last_tick: (duration * sample_rate) as u32,
                step: 1.0/sample_rate,
                generator
        }
    }
}

impl<T: Gen> Iterator for Ticks<T> {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        let tick = self.curr_tick;
        if tick >= self.last_tick {return None}
        self.curr_tick += 1;
        Some(self.generator.gen(self.step * (tick as f32)))
    }
}

const WAV_SPEC: hound::WavSpec = hound::WavSpec {
    channels: 2,
    sample_rate: 44100,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int
};

fn wrr<T: Gen>(writer: &mut hound::WavWriter<std::io::BufWriter<std::fs::File>>, dur: f32, freq: f32, phase1: f32, phase2: f32) -> () {
  let ampl = i16::MAX as f32;
  let rate = WAV_SPEC.sample_rate as f32;
  let left = Ticks::new(dur, rate, T::new(ampl, freq, phase1));
  let right = Ticks::new(dur, rate, T::new(ampl, freq, phase2));
  for (l,r) in left.zip(right) {
      writer.write_sample(l).unwrap();
      writer.write_sample(r).unwrap();
  }
}

fn write_plain<T: Gen>(file: &str, dur: f32, freq: f32, phase: f32) -> () {
  let mut writer = hound::WavWriter::create(file, WAV_SPEC).unwrap();
  wrr::<T>(&mut writer, dur, freq, 0.0, phase);
}

fn write_combo<T1: Gen, T2: Gen>(file:&str, dur1: f32, dur2: f32, freq:f32, shift: f32) -> () {
  assert!(shift > 0.0);
  let mut writer = hound::WavWriter::create(file, WAV_SPEC).unwrap();
  for n in 0 .. (360.0/shift) as usize {
    wrr::<T1>(&mut writer, dur1, freq, 0.0, shift * (n as f32));
    wrr::<T2>(&mut writer, dur2, freq, 0.0, shift * (n as f32));
  }
}

fn main() {
  let matches = App::new("Signal generator")
    .subcommand(SubCommand::with_name("plain")
      .about("generate plain wave")
      .arg(Arg::with_name("FREQ")
        .help("signal frequency in Hz")
        .required(true)
        .index(1))
      .arg(Arg::with_name("DURATION")
        .help("signal duration in Sec")
        .required(true)
        .index(2))
      .arg(Arg::with_name("PHASE")
        .help("phase shift in Degree")
        .required(true)
        .index(3))
      .arg(Arg::with_name("SHAPE")
        .help("signal shape: sine, square, saw")
        .required(true)
        .index(4))
      .arg(Arg::with_name("OUTPUT")
        .help("name of output file")
        .required(true)
        .index(5)))
    .subcommand(SubCommand::with_name("combo")
      .about("generate combo wave")
      .arg(Arg::with_name("FREQ")
        .help("signal frequency in Hz")
        .required(true)
        .index(1))
      .arg(Arg::with_name("DURATION")
        .help("signal duration in Sec")
        .required(true)
        .index(2))
      .arg(Arg::with_name("SILENCE")
        .help("silence duration in Sec")
        .required(true)
        .index(3))
      .arg(Arg::with_name("PHASE")
        .help("phase shift in Degree")
        .required(true)
        .index(4))
      .arg(Arg::with_name("SHAPE")
        .help("signal shape: sine, square, saw")
        .required(true)
        .index(5))
      .arg(Arg::with_name("OUTPUT")
        .help("name of output file")
        .required(true)
        .index(6)))
  .get_matches();

  if let Some(matches) = matches.subcommand_matches("plain") {
    let freq = matches.value_of("FREQ").unwrap().parse::<f32>().expect("Invalid value of FREQ");
    let dur = matches.value_of("DURATION").unwrap().parse::<f32>().expect("Invalid value of DURATION");
    let phase = matches.value_of("PHASE").unwrap().parse::<f32>().expect("Invalid value of PHASE");
    let file = matches.value_of("OUTPUT").unwrap();
    match matches.value_of("SHAPE").unwrap() {
      "sine" =>
        write_plain::<Sine>(file, dur, freq, phase),
      "square" =>
        write_plain::<Square>(file, dur, freq, phase),
      "saw" =>
        write_plain::<Saw>(file, dur, freq, phase),
      _ =>
        panic!("Invalid value of SHAPE")
    }
  }
  else if let Some(matches) = matches.subcommand_matches("combo") {
    let freq = matches.value_of("FREQ").unwrap().parse::<f32>().expect("Invalid value of FREQ");
    let dur = matches.value_of("DURATION").unwrap().parse::<f32>().expect("Invalid value of DURATION");
    let sil = matches.value_of("SILENCE").unwrap().parse::<f32>().expect("Invalid value of SILENCE");
    let phase = matches.value_of("PHASE").unwrap().parse::<f32>().expect("Invalid value of PHASE");
    let file = matches.value_of("OUTPUT").unwrap();
    match matches.value_of("SHAPE").unwrap() {
      "sine" =>
        write_combo::<Sine, Silence>(file, dur, sil, freq, phase),
      "square" =>
        write_combo::<Square, Silence>(file, dur, sil, freq, phase),
      "saw" =>
        write_combo::<Saw, Silence>(file, dur, sil, freq, phase),
      _ =>
        panic!("Invalid value of SHAPE")
    }
  }
  else {panic!("Invalid subcommand")}
}

