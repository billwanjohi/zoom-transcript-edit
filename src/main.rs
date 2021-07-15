extern crate clap;
use chrono::{Duration, NaiveTime};
use clap::{App, AppSettings, Arg};
use contracts::debug_requires;
use std::io::{BufRead, LineWriter, Write};
use std::{fs::File, io, path::Path};

struct NaiveTimeEntry {
    time: NaiveTime,
    transcription: String,
}

struct DurationEntry {
    duration: Duration,
    transcription: String,
}

struct ScanState {
    first_time: Option<NaiveTime>,
    previous_time: Option<NaiveTime>,
    days_elapsed: u8,
}

fn main() {
    let matches = App::new("zoom-transcript-edit")
        .version("0.1")
        .author("Bill K. Wanjohi <bill@ka.guru>")
        .setting(AppSettings::ArgRequiredElseHelp)
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    // Calling .unwrap() is safe here because "INPUT" is required (if "INPUT" wasn't
    // required we could have used an 'if let' to conditionally get the value)
    let path = matches.value_of("INPUT").unwrap();
    let lines = match read_lines(path) {
        Err(why) => panic!("couldn't read {}: {}", path, why),
        Ok(it) => it,
    };
    let new_path = match path.rsplit_once('.') {
        None => format!("{}.qda", path),
        Some((prefix, extension)) => format!("{}.qda.{}", prefix, extension),
    };
    let out_file = File::create(&new_path).unwrap();
    let mut line_writer = LineWriter::new(out_file);
    lines.map(|line| line_to_entry(&line.unwrap())).scan(
        ScanState {
            first_time: None,
            previous_time: None,
            days_elapsed: 0,
        },
        |state, entry| {
            if state.first_time.is_none() {
                state.first_time = Some(entry.time);
                state.previous_time = Some(entry.time);
                return Some(DurationEntry {
                    duration: Duration::minutes(0),
                    transcription: entry.transcription,
                });
            } else {
                if state.previous_time.unwrap() > entry.time {
                    state.days_elapsed += 1
                }
                return Some(DurationEntry {
                    duration: time_elapsed(
                        state.first_time.unwrap(),
                        state.days_elapsed,
                        entry.time,
                    ),
                    transcription: entry.transcription,
                });
            };
        },
    ).map(|de| {
        format!("{} {}", reconstitute_time_str(de.duration), de.transcription)}
    )
    .for_each(|line| {
        line_writer.write(&line.into_bytes()).unwrap();
        line_writer.write(b"\n").unwrap();
    });
    println!("Wrote transcript with QDA timestamps to: {}", new_path)
}

// https://doc.rust-lang.org/stable/rust-by-example/std_misc/file/read_lines.html
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn line_to_entry(line: &str) -> NaiveTimeEntry {
    match line.split_once(' ') {
        None => panic!("not a valid transcript entry: {}", line),
        Some((time, transcription)) => NaiveTimeEntry {
            time: NaiveTime::parse_from_str(time, "%H:%M:%S").unwrap(),
            transcription: transcription.to_string(),
        },
    }
}

fn time_elapsed(first_time: NaiveTime, days_elapsed: u8, current_time: NaiveTime) -> Duration {
    Duration::days(days_elapsed as i64) + (current_time - first_time)
}

#[debug_requires(duration.num_hours() >= 0, "Duration cannot be negative")]
#[debug_requires(duration.num_hours() < 24, "Duration cannot exceed one day")]
fn reconstitute_time_str(duration: Duration) -> String {
    (NaiveTime::from_hms(0, 0, 0) + duration)
        .format("%H:%M:%S")
        .to_string()
}
