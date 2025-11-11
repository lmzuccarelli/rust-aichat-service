#!/bin/bash

cd ~/Projects/rust-jira-service/
./target/release/jira-service --config config/jira-service-config.json --issues WRIG-1327,WRIG-1323,WRIG-1341

