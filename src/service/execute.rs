use crate::chat::client::ChatClient;
use crate::chat::model::{CompletionRequest, InputMessage};
use crate::cli::schema::ApplicationConfig;
use custom_logger as log;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
use std::sync::Arc;

#[derive(Debug)]
enum ExitStatus {
    OK,
    WARNING,
    ERROR,
}

pub trait ExecuteInterface {
    fn new(client: Arc<dyn ChatClient>, config: ApplicationConfig) -> Self;
    async fn process_task(
        &mut self,
        input_command: String,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct Execute {
    pub client: Arc<dyn ChatClient>,
    pub config: ApplicationConfig,
    pub memory_map: HashMap<String, String>,
    pub messages: Vec<InputMessage>,
}

impl ExecuteInterface for Execute {
    fn new(client: Arc<dyn ChatClient>, config: ApplicationConfig) -> Self {
        let system_prompt = InputMessage {
            role: "system".to_string(),
            content: config.spec.system_prompt.clone(),
        };
        return Execute {
            client,
            config,
            memory_map: HashMap::new(),
            messages: vec![system_prompt],
        };
    }

    async fn process_task(
        &mut self,
        input_command: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match input_command.clone() {
            x if x.contains("none") => {
                log::warn!("[process_task] input command is not well formed");
                Ok(())
            }
            x if x.contains("show") => {
                let res_data = self.memory_map.get("current");
                match res_data {
                    Some(data) => {
                        println!("{}", data);
                    }
                    None => {
                        log::warn!("[process_task] no 'current' content found");
                    }
                }
                Ok(())
            }
            x if x.contains("read") => {
                // we need to handle the error gracefully so we don't crash
                let res_data = fs::read_to_string(format!(
                    "{}/staging/current.md",
                    self.config.spec.working_dir
                ));
                match res_data {
                    Ok(data) => {
                        self.memory_map.insert("current".to_string(), data);
                        log::info!(
                            "[process_task] succesfully read 'staging/current.md' from disk"
                        );
                    }
                    Err(err) => {
                        log::warn!("{}", err.to_string().to_lowercase());
                    }
                }
                Ok(())
            }
            x if x.contains("execute") => {
                // we are confident the parser has the format <command> <file_path>
                let script = input_command.split(" ").nth(1).unwrap();
                let mut exit_status: Option<ExitStatus> = None;
                let mut command = Command::new(format!("{}", script));
                log::debug!("[process_task] service command to execute {:?}", command);

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
                                    // it will always be set to ERROR
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
                                log::info!(
                                    "[process_task] service {} executed successfully",
                                    script
                                );
                            }
                            ExitStatus::WARNING => {
                                let err = format!(
                                    "[process_task] agent {} executed with warning",
                                    script
                                );
                                log::warn!("{}", err);
                                return Err(Box::from(err));
                            }
                            ExitStatus::ERROR => {
                                let err = format!("[process_task] command failed : {} ", script);
                                log::error!("[process_task] {}", err);
                                return Err(Box::from(err));
                            }
                        }
                    }
                    Err(err) => {
                        let task_err = format!(
                            "[process_task] command failed : {}",
                            err.to_string().to_lowercase()
                        );
                        log::error!("[process_task] {}", task_err);
                        return Err(Box::from(task_err));
                    }
                }
                Ok(())
            }
            _ => {
                let res_content = self.memory_map.get("current");
                let full_prompt = match res_content {
                    Some(content) => {
                        format!("{} {}", input_command.clone(), content)
                    }
                    None => input_command.to_owned(),
                };
                // 0 should be role = system
                match self.messages.get(1) {
                    Some(_) => {
                        // dont alter role
                        self.messages[1].content = full_prompt
                    }
                    None => {
                        self.messages.push(InputMessage::user(full_prompt));
                    }
                }
                log::debug!("[process_task] prompt {:?}", self.messages,);

                let request = CompletionRequest {
                    model: self.config.spec.model.clone(),
                    messages: self.messages.clone(),
                    top_p: self.config.spec.top_p,
                    temperature: Some(self.config.spec.temperature),
                    stream: self.config.spec.stream,
                    max_tokens: self.config.spec.max_tokens,
                };

                let res = self.client.complete(request).await;
                match res {
                    Ok(data) => {
                        let file_name =
                            format!("{}/staging/inference.md", self.config.spec.working_dir);
                        fs::write(file_name.clone(), data)?;
                        fs::set_permissions(file_name, fs::Permissions::from_mode(0o777))?;
                    }
                    Err(err) => {
                        log::error!("[process_task] {}", err.to_string());
                    }
                }
                Ok(())
            }
        }
    }
}
