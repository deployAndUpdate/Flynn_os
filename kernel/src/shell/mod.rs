pub mod commands;
use crate::driver::serial::SerialPort;

pub fn execute(cmd: &str) {
    let cmd = cmd.trim();
    if cmd.is_empty() {
        return;
    }

    let mut parts = cmd.splitn(2, char::is_whitespace);
    let name = parts.next().unwrap();
    let args = parts.next().unwrap_or("");

    match name {
        "help" => commands::help(),
        "ticks" => commands::ticks(),
        "ps" => commands::ps(),
        "sleep" => {
            if let Ok(ticks) = args.trim().parse::<u32>() {
                commands::sleep_ticks(ticks);
            } else {
                SerialPort::write_str_no_preempt("usage: sleep <ticks>\n");
            }
        }
        "mem" => commands::mem(),
        "preempts" => commands::preempts(),
        "clear" => commands::clear(),
        "say" => commands::say(args),

        _ => {
            SerialPort::write_str_no_preempt("Unknown command\n");
        }
    }
}
