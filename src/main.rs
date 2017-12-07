extern crate hound;
extern crate clap;

use std::f32::consts::PI;
use std::i16;
use clap::{App, Arg, SubCommand};

const WAV_SPEC: hound::WavSpec = hound::WavSpec {
    channels: 2,
    sample_rate: 44100,
    bits_per_sample: 16,
    sample_format: hound::SampleFormat::Int };

fn silence(_ampl:f32, _freq:f32, _phase:f32, _x:f32) -> f32 {
  0.0
}

fn square(ampl:f32, freq:f32, phase:f32, x:f32) -> f32 {
  let arg = freq*x + phase/360.0;
  if arg.fract() < 0.5 {ampl} else {-ampl}
}

fn sine(ampl:f32, freq:f32, phase:f32, x:f32) -> f32 {
  ampl*(2.0*PI*(freq*x + phase/360.0)).sin()
}

fn generate_plain<T: Fn(f32,f32,f32,f32) -> f32>(file:&str, freq:f32, dur:u32, phase:f32, shape:T) -> () {
  let num_samples = WAV_SPEC.sample_rate * dur;
  let ampl = i16::MAX as f32;
  let mut writer = hound::WavWriter::create(file, WAV_SPEC).unwrap();
  for n in 0 .. num_samples {
    let t = n as f32 / WAV_SPEC.sample_rate as f32;
    let left_chan = shape(ampl, freq, 0.0, t) as i16;
    let right_chan = shape(ampl, freq, phase, t) as i16;
    writer.write_sample(left_chan).unwrap();
    writer.write_sample(right_chan).unwrap();
  }
}

fn generate_combo<T: Fn(f32,f32,f32,f32) -> f32>(file:&str, freq:f32, dur:u32, sil:u32, phase:u32, shape:T) -> () {
  let period = dur + sil; 
  let num_samples = WAV_SPEC.sample_rate * period * (1 + 360 / phase);
  let ampl = i16::MAX as f32;
  let mut writer = hound::WavWriter::create(file, WAV_SPEC).unwrap();
  for n in 0 .. num_samples {
    let k = n / WAV_SPEC.sample_rate / period;
    let t = n as f32 / WAV_SPEC.sample_rate as f32;
    let p = (k * phase) as f32;
    let (left_chan, right_chan) = if t < ((k * period + dur) as f32) {(shape(ampl, freq, 0.0, t), shape(ampl, freq, p, t))} 
                                  else {(silence(ampl, freq, 0.0, t), silence(ampl, freq, p, t))};
    writer.write_sample(left_chan as i16).unwrap();
    writer.write_sample(right_chan as i16).unwrap();
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
        .help("signal shape: sine, square")
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
        .help("signal shape: sine, square")
        .required(true)
        .index(5))
      .arg(Arg::with_name("OUTPUT")
        .help("name of output file")
        .required(true)
        .index(6)))
  .get_matches();

  if let Some(matches) = matches.subcommand_matches("plain") {
    let freq = matches.value_of("FREQ").unwrap().parse::<f32>().expect("Invalid value of FREQ");
    let dur = matches.value_of("DURATION").unwrap().parse::<u32>().expect("Invalid value of DURATION");
    let phase = matches.value_of("PHASE").unwrap().parse::<f32>().expect("Invalid value of PHASE");
    let file = matches.value_of("OUTPUT").unwrap();
    match matches.value_of("SHAPE").unwrap() {
      "sine" => generate_plain(file, freq, dur, phase, sine),
      "square" => generate_plain(file, freq, dur, phase, square),
      _ => panic!("Invalid value of SHAPE")
    }
  }
  else if let Some(matches) = matches.subcommand_matches("combo") {
    let freq = matches.value_of("FREQ").unwrap().parse::<f32>().expect("Invalid value of FREQ");
    let dur = matches.value_of("DURATION").unwrap().parse::<u32>().expect("Invalid value of DURATION");
    let sil = matches.value_of("SILENCE").unwrap().parse::<u32>().expect("Invalid value of SILENCE");
    let phase = matches.value_of("PHASE").unwrap().parse::<u32>().expect("Invalid value of PHASE");
    let file = matches.value_of("OUTPUT").unwrap();
    match matches.value_of("SHAPE").unwrap() {
      "sine" => generate_combo(file, freq, dur, sil, phase, sine),
      "square" => generate_combo(file, freq, dur, sil, phase, square),
      _ => panic!("Invalid value of SHAPE")
    }
  }
  else {panic!("Invalid subcommand")}
}

