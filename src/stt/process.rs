use crate::chat::client::OpenAIClient;
use crate::cli::schema::ApplicationConfig;
use crate::prompt::parser::PromptParser;
use crate::service::execute::{Execute, ExecuteInterface};
use bytes::{BufMut, Bytes, BytesMut};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Sample, SampleFormat};
use crossbeam::channel::RecvError;
use custom_logger as log;
use deepgram::Deepgram;
use deepgram::DeepgramError;
use deepgram::common::options::Encoding;
use deepgram::common::stream_response::Word;
use futures::SinkExt;
use futures::channel::mpsc::{self, Receiver as FuturesReceiver};
use futures::stream::StreamExt;
use std::fs;
use std::io::Write;
use std::process;
use std::sync::Arc;
use std::thread;

macro_rules! create_stream {
    ($device:ident, $config:expr, $sync_tx:ident, $sample_type:ty) => {
        $device
            .build_input_stream(
                &$config.into(),
                move |data: &[$sample_type], _: &_| {
                    let mut bytes = BytesMut::with_capacity(data.len() * 2);
                    for sample in data {
                        bytes.put_i16_le(sample.to_sample());
                    }
                    $sync_tx.send(bytes.freeze()).unwrap();
                },
                |_| panic!(),
                None,
            )
            .unwrap()
    };
}

fn microphone_as_stream() -> FuturesReceiver<Result<Bytes, RecvError>> {
    let (sync_tx, sync_rx) = crossbeam::channel::unbounded();
    let (mut async_tx, async_rx) = mpsc::channel(1);

    thread::spawn(move || {
        let host = cpal::default_host();
        let device = host.default_input_device().unwrap();
        let config = device.default_input_config().unwrap();

        let stream = match config.sample_format() {
            SampleFormat::F32 => create_stream!(device, config, sync_tx, f32),
            SampleFormat::I16 => create_stream!(device, config, sync_tx, i16),
            SampleFormat::U16 => create_stream!(device, config, sync_tx, u16),
            sample_format => {
                log::error!("[microphone_as_stream] unsupported sample format: {sample_format:?}");
                process::exit(1);
            }
        };

        stream.play().unwrap();

        loop {
            thread::park();
        }
    });

    tokio::spawn(async move {
        loop {
            let data = sync_rx.recv();
            let x = async_tx.send(data).await;
            match x {
                Ok(_data) => {}
                Err(_err) => {
                    process::exit(1);
                }
            }
        }
    });
    async_rx
}

pub async fn execute(config: ApplicationConfig) -> Result<(), DeepgramError> {
    let deepgram_api_key = fs::read_to_string(format!("{}", config.spec.deepgram_key_path))?;
    let api_key = fs::read_to_string(format!("{}", config.spec.openai_key_path))?;
    let client = Arc::new(OpenAIClient::new(api_key, config.spec.api_url.clone()));
    let mut ep = Execute::new(client, config.clone());
    let dg_client = Deepgram::new(&deepgram_api_key.trim())?;
    let mut results = dg_client
        .transcription()
        .stream_request()
        .keep_alive()
        .encoding(Encoding::Linear16)
        .sample_rate(44100)
        .channels(2)
        .stream(microphone_as_stream())
        .await?;

    log::info!("[execute] using speech-to-text service");
    log::info!("[execute] deepgram request id: {}", results.request_id());
    println!();
    print!("prompt> ");
    std::io::stdout().flush().unwrap();
    let mut input = String::new();
    let mut exit = false;
    while let Some(result) = results.next().await {
        let response = serde_json::to_value(&result.unwrap()).unwrap();
        let alternatives = response
            .get("channel")
            .unwrap()
            .get("alternatives")
            .unwrap();
        let words = alternatives[0].get("words").unwrap().as_array().unwrap();
        for word in words.iter() {
            if word.is_object() {
                let obj: Word = serde_json::from_value(word.clone()).unwrap();
                match obj.word.as_str() {
                    "send" => {
                        let res_input_command =
                            PromptParser::parse(config.spec.working_dir.clone(), input);
                        match res_input_command {
                            Ok(input_command) => {
                                let res = ep.process_task(input_command.clone()).await;
                                match res {
                                    Ok(_data) => {}
                                    Err(err) => {
                                        log::error!("{}", err.to_string());
                                    }
                                }
                            }
                            Err(err) => {
                                log::warn!("{}", err.to_string());
                            }
                        }
                        println!();
                        print!("prompt> ");
                        std::io::stdout().flush().unwrap();
                        input = String::new();
                    }
                    "cancel" => {
                        println!("cancelled");
                        print!("prompt> ");
                        std::io::stdout().flush().unwrap();
                        input = String::new();
                    }
                    "exit" => {
                        println!();
                        log::warn!("[execute] exiting speech to text service");
                        exit = true;
                        break;
                    }
                    _ => {
                        print!("{} ", obj.word);
                        std::io::stdout().flush().unwrap();
                        input.push_str(&format!("{} ", obj.word));
                    }
                }
            }
        }
        if exit {
            break;
        }
    }
    Ok(())
}
