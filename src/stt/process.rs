use crate::chat::client::{ChatClient, OpenAIClient};
use crate::chat::model::{CompletionRequest, InputMessage};
use crate::cli::schema::ApplicationConfig;
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
                panic!("unsupported sample format: {sample_format:?}");
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
                    //log::error!("{} exiting", err);
                    break;
                }
            }
        }
    });

    async_rx
}

pub async fn execute(config: ApplicationConfig) -> Result<(), DeepgramError> {
    let deepgram_api_key = fs::read_to_string(format!("{}", config.spec.deepgram_key_path))?;
    let openai_api_key = fs::read_to_string(format!("{}", config.spec.openai_key_path))?;
    let dg_client = Deepgram::new(&deepgram_api_key.trim())?;
    let mut ai_result = String::new();
    let mut results = dg_client
        .transcription()
        .stream_request()
        .keep_alive()
        .encoding(Encoding::Linear16)
        .sample_rate(44100)
        .channels(2)
        .stream(microphone_as_stream())
        .await?;

    log::info!("using speech-to-text service");
    log::info!("deepgram request id: {}", results.request_id());
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
                        println!(" processing...");
                        let client = OpenAIClient::new(
                            openai_api_key.trim().to_string(),
                            config.spec.api_url.clone(),
                        );
                        let mut messages = Vec::new();
                        messages.push(InputMessage::system(
                            "You are a helpful assistant. Use the context to help the user.",
                        ));
                        messages.push(InputMessage::user(input.clone()));
                        let request = CompletionRequest {
                            model: config.spec.model.clone(),
                            messages: messages.clone(),
                            top_p: config.spec.top_p,
                            temperature: Some(config.spec.temperature),
                            stream: false,
                            max_tokens: config.spec.max_tokens,
                        };
                        let res = client.complete(request).await;
                        match res {
                            Ok(data) => {
                                ai_result = data;
                            }
                            Err(err) => {
                                log::warn!(
                                    "problems encounted with ai-inference {}",
                                    err.to_string()
                                );
                            }
                        }
                        print!("prompt> ");
                        std::io::stdout().flush().unwrap();
                        input = String::new();
                    }
                    "save" => {
                        input.truncate(input.len() - 1);
                        let filename = format!("documents/{}.md", input.replace(" ", "-"));
                        fs::write(&filename, ai_result.clone())?;
                        log::info!("contents saved successfuly to {}", filename);
                        print!("prompt> ");
                        std::io::stdout().flush().unwrap();
                        input = String::new();
                    }
                    "read" => {
                        if input.contains("file") {
                            let files = fs::read_dir("documents")?;
                            for file in files {
                                let name = file.unwrap().file_name().to_string_lossy().to_string();
                                if name.contains(&input) {
                                    ai_result =
                                        fs::read_to_string(format!("documents/{}", name.clone()))?;
                                    log::info!("file {} read successfuly", name);
                                }
                            }
                            print!("prompt> ");
                            std::io::stdout().flush().unwrap();
                            input = String::new();
                        }
                    }
                    "execute" => {
                        if input.contains("scripts") && input.contains("test") {
                            let _ = Execute::process_task("scripts/test.sh".to_string());
                        }
                        print!("prompt> ");
                        std::io::stdout().flush().unwrap();
                    }
                    "cancel" => {
                        println!("cancelled");
                        print!("prompt> ");
                        std::io::stdout().flush().unwrap();
                        input = String::new();
                    }
                    "exit" => {
                        println!();
                        log::warn!("exiting stt service");
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
