use std::time::{SystemTime, UNIX_EPOCH};

use nvml_wrapper::Nvml;
use sysinfo::Pid;

fn main() -> anyhow::Result<()> {
    let nvml = Nvml::init()?;

    let sys: sysinfo::System = sysinfo::System::new_all(); 

    for i in 0..nvml.device_count()? {
        let device = nvml.device_by_index(i)?;
        let processes = device.running_graphics_processes()?;
        let timestamp = get_timestamp_ms()?;

        let utilization = device.process_utilization_stats(Some(timestamp - 1000))?;

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
                    }
                }
            }
        }
    }

    Ok(())
}

fn get_timestamp_ms() -> anyhow::Result<u64> {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH)?;
    Ok(since_epoch.as_millis() as u64)
}
