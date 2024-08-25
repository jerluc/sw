use dirs;
use serde::{Deserialize, Serialize};
use std::{
    env,
    error::Error,
    fmt::Display,
    fs::{create_dir_all, write, File},
    io::{BufReader, BufWriter},
    path::PathBuf,
    process::{exit, Command},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Serialize, Deserialize)]
struct Timing {
    command: String,
    args: Vec<String>,
    start_epoch_ms: u128,
    duration_s: f64,
}

#[derive(Serialize, Deserialize)]
struct StopWatchHistory {
    timings: Vec<Timing>,
}

struct CommandStats {
    command: String,
    args: Vec<String>,
    total: usize,
    min_duration_s: f64,
    max_duration_s: f64,
    mean_duration_s: f64,
    std_dev_duration_s: f64,
}

impl Display for CommandStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut command_with_args = vec![self.command.clone()];
        command_with_args.extend(self.args.clone());
        write!(
            f,
            "Timing statistics for command [{}]",
            command_with_args.join(" ")
        )?;
        write!(f, "\n               Total: {}", self.total)?;
        write!(
            f,
            "\n      Duration (min): {:?}",
            Duration::from_secs_f64(self.min_duration_s)
        )?;
        write!(
            f,
            "\n      Duration (max): {:?}",
            Duration::from_secs_f64(self.max_duration_s)
        )?;
        write!(
            f,
            "\n     Duration (mean): {:?}",
            Duration::from_secs_f64(self.mean_duration_s)
        )?;
        write!(
            f,
            "\n  Duration (std dev): {:?}",
            Duration::from_secs_f64(self.std_dev_duration_s)
        )?;
        Ok(())
    }
}

fn ensure_history_file() -> Result<PathBuf, Box<dyn Error>> {
    let base_dir = dirs::data_local_dir().unwrap().join("sw");

    if !base_dir.exists() {
        create_dir_all(base_dir.clone())?;
    }

    let history_path = base_dir.join("history.json");

    if !history_path.exists() {
        write(
            history_path,
            serde_json::to_string(&StopWatchHistory {
                timings: Vec::new(),
            })?,
        )?;
    }
    Ok(base_dir.join("history.json"))
}

fn get_history() -> Result<StopWatchHistory, Box<dyn Error>> {
    let history_path = ensure_history_file()?;

    if let Ok(file) = File::open(history_path) {
        let reader = BufReader::new(file);
        let history = serde_json::from_reader(reader)?;

        Ok(history)
    } else {
        Ok(StopWatchHistory {
            timings: Vec::new(),
        })
    }
}

fn std_deviation(mean: f64, durations: Vec<f64>) -> f64 {
    let count = durations.len();
    let variance = durations
        .iter()
        .map(|value| {
            let diff = mean - (*value);

            diff * diff
        })
        .sum::<f64>()
        / count as f64;

    variance.sqrt()
}

fn update_history(
    command_with_args: Vec<String>,
    start: SystemTime,
    duration: Duration,
) -> Result<CommandStats, Box<dyn Error>> {
    let command = &command_with_args[0];
    let args = &command_with_args[1..].to_vec();

    let history_path = ensure_history_file()?;
    let file = File::create_new(history_path).unwrap_or_else(|_| {
        let path = dirs::data_local_dir()
            .unwrap()
            .join("sw")
            .join("history.json");
        File::options().write(true).open(path).unwrap()
    });

    let writer = BufWriter::new(file);

    let old_history = get_history().unwrap();
    let mut timings = old_history.timings.clone();
    timings.push(Timing {
        command: command.to_owned(),
        args: args.clone(),
        start_epoch_ms: start.duration_since(UNIX_EPOCH.into()).unwrap().as_millis(),
        duration_s: duration.as_secs_f64(),
    });

    // TODO: Need to truncate history at some point...

    let new_history = StopWatchHistory { timings };

    serde_json::to_writer(writer, &new_history).unwrap();

    let mut durations_for_command = new_history
        .timings
        .iter()
        .filter(|t| t.command == *command && t.args == *args)
        .map(|t| t.duration_s)
        .collect::<Vec<f64>>();

    // NOTE: Have to sort this manually since Ord is not implemented for f64
    durations_for_command.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let total = durations_for_command.len();
    let sum_s = durations_for_command.iter().sum::<f64>();
    let mean_duration_s = sum_s / (total as f64);

    Ok(CommandStats {
        command: command.to_owned(),
        args: args.clone(),
        total,
        min_duration_s: *durations_for_command.first().unwrap(),
        max_duration_s: *durations_for_command.last().unwrap(),
        mean_duration_s,
        std_dev_duration_s: std_deviation(mean_duration_s, durations_for_command),
    })
}

fn main() {
    // Skip the entrypoint name
    let command_with_args: Vec<String> = env::args().skip(1).collect();
    if command_with_args.len() == 0 {
        println!("Usage: sw <CMD> [ARGS...]");
        exit(1)
    }

    let command = &command_with_args[0];
    let args = &command_with_args[1..];

    let start = SystemTime::now();
    if let Ok(status) = Command::new(command).args(args).status() {
        let duration = start.elapsed().unwrap();
        println!("Took {:?}", duration);
        let stats = update_history(command_with_args, start, duration).unwrap();
        if stats.total > 1 {
            println!("{}", stats);
        }
        exit(status.code().unwrap())
    } else {
        println!("Couldn't run");
        exit(255)
    }
}
