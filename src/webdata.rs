use log::*;
use reqwest;
use serde::Deserialize;
use serde_json;

pub struct AllMineFee {

}

pub struct Worker {
    pub name: String,
    pub dna: String,
    pub sysmons: SysMons,
    pub cores: Cores,

}

pub struct WorkSources {

}

#[derive(Deserialize, Debug)]
pub struct SysMon {
    pub health: String,
    pub temperature: f32,
    pub vccaux: f32,
    pub vccbram: f32,
    pub vccint: f32,
}

#[derive(Deserialize)]
pub struct SysMons {
    pub sysmon: Vec<SysMon>,
}

#[derive(Deserialize,Debug)]
pub struct Clock {
    pub badNonces: f32,
    pub health: String,
    pub multiplier: f32,
    pub totalNonces: f32,
}

#[derive(Deserialize,Debug)]
pub struct StatDetail {
    pub accepted: f32,
    pub calculated: f32,
    pub endTime: i64,
    pub found: f32,
    pub requested: f32,
    pub startTime: i64,
    pub submitted: f32,
    pub valid: f32,
}

#[derive(Deserialize,Debug)]
pub struct Stats {
    pub minute: StatDetail,
    pub name: String,
    pub total: StatDetail,
}

#[derive(Deserialize,Debug)]
pub struct Core {
    pub clock: Clock,
    pub stats: Stats,
}

#[derive(Deserialize)]
pub struct Cores {
    pub cores: Vec<Core>,
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
        let send = self.client.get(&url).send();
        let resp = match send {
            Err(e) => {
                match e.status() {
                    Some(err) => info!("Problem parsing info {}", err),
                    None => info!("No status given"),
                }
                if e.is_redirect() {
                    info!("server redirecting too many times or making loop");
                }
                return
            },
            Ok(resp) => resp
        };
        match resp.text() {
            Ok(response) => {
                let blob: serde_json::Value = serde_json::from_str(&response).unwrap();

                self.workers.clear();
        
                self.minerator = blob["minerator"].as_str().unwrap()
                    .to_string();
                debug!("Read minerator: {}", self.minerator);
        
                let config = &blob["workers"];
                // Because we don't know the name of the key for the config, we do weird stuff
                // to find the first key value pair and work on the value
                match config {
                    serde_json::Value::Object(thing) => {
                        if let Some(dev) = thing.iter().next() {
                            let (_name, device) = dev;
                            match &device["devices"] {
                                serde_json::Value::Array(workers) => 
                                for w in workers {
                                    let s = format!("{{ \"sysmon\": {} }}", w["sysmon"].to_string());
                                    let sysmons: SysMons = serde_json::from_str(&*s).unwrap();
        
                                    let c = format!("{{ \"cores\": {} }}", w["cores"].to_string());
                                    debug!("cores = {}", c);
                                    let cores: Cores = serde_json::from_str(&*c).unwrap();
        
                                    self.workers.push(
                                        Worker { 
                                            dna: w["dna"].as_str().unwrap().to_string(), 
                                            name: w["name"].as_str().unwrap().to_string(),
                                            sysmons,
                                            cores,
                                        });
                                },
                                _ => {},
                            }
                        }
                    },
                    _ => {},
                }
        
        //        debug!("Read first device dna {}, name {}", self.workers[0].dna, self.workers[0].name);
        
            },
            Err(e) => {
                match e.status() {
                    Some(err) => info!("Problem parsing info {}", err),
                    None => info!("No status given"),
                }
                if e.is_redirect() {
                    info!("server redirecting too many times or making loop");
                }
            }
        }
    }
}