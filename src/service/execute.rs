use custom_logger as log;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

#[derive(Debug)]
enum ExitStatus {
    OK,
    WARNING,
    ERROR,
}

pub trait ExecuteInterface {
    fn process_task(script: String) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct Execute {}

impl ExecuteInterface for Execute {
    fn process_task(script: String) -> Result<(), Box<dyn std::error::Error>> {
        let mut exit_status: Option<ExitStatus> = None;
        let mut command = Command::new(script.clone());
        log::debug!("service command to execute {:?}", command);
        let cmd_res = command.stdout(Stdio::piped()).spawn();
        match cmd_res {
            Ok(res) => {
                let mut out = res.stdout.unwrap();
                let mut reader = BufReader::new(&mut out);
                // we use println and not custom_logger to preserve original output
                println!();
                loop {
                    let mut line = String::new();
                    let num_bytes = reader.read_line(&mut line).unwrap();
                    match line.clone() {
                        x if x.contains("exit => 0") => {
                            if exit_status.is_none() {
                                exit_status = Some(ExitStatus::OK);
                            }
                        }
                        x if x.contains("exit => 1") => {
                            if exit_status.is_none() {
                                exit_status = Some(ExitStatus::WARNING);
                            }
                        }
                        _ => {
                            // dont set this in the loop
                            // it will always set to ERROR
                        }
                    }
                    if num_bytes == 0 {
                        println!("=> end of stream\n");
                        break;
                    }
                    print!("{}", line);
                }
                if exit_status.is_none() {
                    exit_status = Some(ExitStatus::ERROR);
                }
                match exit_status.unwrap() {
                    ExitStatus::OK => {
                        log::info!("[process_task] service {} executed successfully", script);
                    }
                    ExitStatus::WARNING => {
                        let err = format!("[process_task] agent {} executed with warning", script);
                        log::warn!("{}", err);
                        return Err(Box::from(err));
                    }
                    ExitStatus::ERROR => {
                        let err = format!("command failed : {} ", script);
                        log::error!("[process_task] {}", err);
                        return Err(Box::from(err));
                    }
                }
            }
            Err(err) => {
                let task_err = format!("command failed : {}", err.to_string().to_lowercase());
                log::error!("[process_task] {}", task_err);
                return Err(Box::from(task_err));
            }
        }
        Ok(())
    }
}
