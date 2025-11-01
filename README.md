# Overview

A simple chat service with save, read and tool call features

## Requirements

This service uses cerebras (openai schema) as its base, it can be configured to use any openai schema enabled service, along with the relevant api token

## Usage

Clone this repo

cd rust-aichat-service

```
make build
```

Launch normal chat client workflow

```
./target/release/rust-aichat-service --config config.json 
```







