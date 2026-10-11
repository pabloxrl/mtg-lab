mod benchmark;
mod capture;
mod commands;
mod latency;
mod native;
#[cfg(test)]
mod profile;
mod resident;
mod script;
mod simulate;
mod trajectories;
use serde_json::json;
use std::{
    fs::OpenOptions,
    io::{self, Read, Write},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
fn execute() -> Result<i32, (i32, String)> {
    // Register before parsing/config I/O; handlers perform only an atomic store.
    let signal = Arc::new(AtomicUsize::new(0));
    for n in [signal_hook::consts::SIGTERM, signal_hook::consts::SIGINT] {
        signal_hook::flag::register_usize(n, Arc::clone(&signal), n as usize)
            .map_err(|e| (3, e.to_string()))?;
    }
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.first().is_none_or(|a| a != "simulate") {
        return commands::execute(&args, &signal);
    }
    if !(args.len() == 3 || args.len() == 5)
        || args[0] != "simulate"
        || args[1] != "--config"
        || (args.len() == 5 && args[3] != "--output")
    {
        return Err((
            2,
            "usage: mtg simulate --config CONFIG.json [--output NEW.jsonl]".into(),
        ));
    }
    // Bounded config read, including FIFOs/nonregular inputs rejected before reading.
    let meta = std::fs::metadata(&args[2]).map_err(|e| (2, e.to_string()))?;
    if !meta.is_file() || meta.len() > 1_048_576 {
        return Err((2, "config must be a regular file of at most 1 MiB".into()));
    }
    let mut bytes = Vec::new();
    std::fs::File::open(&args[2])
        .and_then(|f| f.take(1_048_577).read_to_end(&mut bytes))
        .map_err(|_| (2, "configuration read failed".into()))?;
    if bytes.len() > 1_048_576 {
        return Err((2, "configuration byte bound exceeded".into()));
    }
    let config: simulate::Config = serde_json::from_slice(&bytes)
        .map_err(|_| (2, "invalid simulation configuration".into()))?;
    config.validate().map_err(|e| (2, e))?;
    let mut output: Box<dyn Write> = if args.len() == 5 {
        Box::new(
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&args[4])
                .map_err(|e| (3, e.to_string()))?,
        )
    } else {
        Box::new(io::stdout().lock())
    };
    let start = Instant::now();
    let control = || match signal.load(Ordering::Relaxed) {
        n if n == signal_hook::consts::SIGTERM as usize => Some(simulate::Stop::Sigterm),
        n if n == signal_hook::consts::SIGINT as usize => Some(simulate::Stop::Sigint),
        _ if config
            .deadline_ms
            .is_some_and(|ms| start.elapsed().as_millis() >= ms as u128) =>
        {
            Some(simulate::Stop::Deadline)
        }
        _ => None,
    };
    if config.schema_version >= 2 {
        native::run(&config, &mut output, control)
            .map_err(|_| (3, "simulation output failed".into()))
    } else {
        simulate::run(&config, &mut output, control, simulate::step).map_err(|e| (3, e.to_string()))
    }
}
fn main() {
    let code = match execute() {
        Ok(code) => code,
        Err((code, error)) => {
            let _ = writeln!(
                io::stderr(),
                "{}",
                json!({"schema_version":1,"type":"error","exit_code":code,"message":error})
            );
            code
        }
    };
    std::process::exit(code);
}
