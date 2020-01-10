use log::*;
use reqwest;
use serde_json;

pub struct AllMineFee {

}

pub struct Worker {
    pub name: String,
    pub dna: String,

}

pub struct WorkSources {

}

pub struct WebData {
    client: reqwest::blocking::Client,
    fee: AllMineFee,
    pub minerator: String,
    version: i32,
    pub workers: Vec<Worker>,
    worksources: WorkSources,
}

impl WebData {
    pub fn new() -> WebData {
        WebData {
            client: reqwest::blocking::Client::new(),
            fee: AllMineFee {},
            minerator: "None".to_string(),
            version: 0,
            workers: vec!(),
            worksources: WorkSources {},
        }
    }
    pub fn getdata<'a>(&mut self,matches: &clap::ArgMatches<'a>) {
        let default_host = "http://localhost";
        let host = matches.value_of("host")
        .unwrap_or(&default_host);
        let url = format!("{}/api/status", host);

        debug!("Looking at url: {}", url);
        let response = self.client.get(&url)
            .send().unwrap()
            .text().unwrap();
        let blob: serde_json::Value = serde_json::from_str(&response).unwrap();

        self.workers.clear();

        self.minerator = blob["minerator"].as_str().unwrap()
            .to_string();
        debug!("Read minerator: {}", self.minerator);

        let config = &blob["workers"];
        debug!("config = {}", config);
        // Because we don't know the name of the key for the config, we do weird stuff
        // to find the first key value pair and work on the value
        match config {
            serde_json::Value::Object(thing) => {
                if let Some(dev) = thing.iter().next() {
                    let (_name, device) = dev;
                    match &device["devices"] {
                        serde_json::Value::Array(workers) => 
                        for w in workers {
                            self.workers.push(
                                Worker { 
                                    dna: w["dna"].as_str().unwrap().to_string(), 
                                    name: w["name"].as_str().unwrap().to_string()
                                });
                        },
                        _ => {},
                    }
                }
            },
            _ => {},
        }

        debug!("Read first device dna {}, name {}", self.workers[0].dna, self.workers[0].name);

    }
}