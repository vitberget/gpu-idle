use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use nvml_wrapper::Nvml;
use sysinfo::Pid;

#[derive(Parser, Debug)]
#[command(author,version,about,long_about=None)]
struct Args {
    /// Command to run when GPU in use
    #[arg(short,long)]
    cmd: Option<String>,

    /// Time window to check
    #[arg(short,long, default_value_t=1000)]
    delta: u64
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let nvml = Nvml::init()?;

    let sys: sysinfo::System = sysinfo::System::new_all(); 

    for i in 0..nvml.device_count()? {
        let device = nvml.device_by_index(i)?;
        let processes = device.running_graphics_processes()?;
        let timestamp = get_timestamp_ms()?;

        let utilization = device.process_utilization_stats(Some(timestamp - args.delta))?;

        for process in processes {
            let name = sys.process(Pid::from_u32(process.pid))
                .map(|p| p.name());

            if let Some(name) = name {
                if name == "firefox" {
                    let util = utilization.iter()
                        .find(|sample| sample.pid == process.pid);

                    if let Some(util) = util {
                        println!("PID {}: {name}", process.pid);
                        println!("Util: {:?}", util);
                        do_command(&args);
                        return Ok(())
                    }
                }
            }
        }
    }

    Ok(())
}

fn do_command(args: &Args) {
    if let Some(cmd) = &args.cmd {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("Failed");
        println!("I ran   : {}", cmd);
        println!("  Output: {:?}", output);
    } else {
        println!("No cmd given");
    }
}

fn get_timestamp_ms() -> anyhow::Result<u64> {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH)?;
    Ok(since_epoch.as_millis() as u64)
}
