use log::*;
use reqwest;
use serde::Deserialize;
use serde_json;

pub struct Worker {
    pub name: String,
    pub dna: String,
    pub input_power: f32,
    pub input_power_health: String,
    pub aux_current: f32,
    pub aux_current_health: String,
    pub sysmons: SysMons,
    pub cores: Cores,
    pub fee: Algo,
    pub worksource: Algo,
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

#[derive(Deserialize,Debug, Clone, Copy)]
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

impl StatDetail {
    pub fn new() -> StatDetail {
        StatDetail {
            accepted: 0.0,
            calculated: 0.0,
            endTime: 0,
            found: 0.0,
            requested: 0.0,
            startTime: 0,
            submitted: 0.0,
            valid: 0.0,
        }
    }
}

#[derive(Deserialize,Debug, Clone)]
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

#[derive(Deserialize, Debug, Clone)]
pub struct Algo {
    pub difficulty: f64,
    pub hashesPerDiff1: f64,
    pub stats: Stats,
}

impl Algo {
    pub fn new() -> Algo {
        Algo { 
            difficulty: 0.0, 
            hashesPerDiff1: 0.0, 
            stats: Stats { 
                minute: StatDetail::new(), 
                name: "None".to_string(),
                total: StatDetail::new(), 
            }}
    }
}

pub struct WebData {
    client: reqwest::blocking::Client,
    pub minerator: String,
    version: i32,
    pub workers: Vec<Worker>,
}

impl WebData {
    pub fn new() -> WebData {
        WebData {
            client: reqwest::blocking::Client::new(),
            minerator: "None".to_string(),
            version: 0,
            workers: vec!(),
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
                let blob: serde_json::Value = match serde_json::from_str(&response) {
                    Ok(blob) => blob,
                    Err(err) => {
                        warn!("Error parsing json: {}", err);
                        return;
                    }

                };
                self.workers.clear();
        
                self.minerator = blob["minerator"].as_str().unwrap()
                    .to_string();
                debug!("Read minerator: {}", self.minerator);

                let thing = &blob["fee"]["allmine-fee-v1"][0]["algo"];
                let mut fee = Algo::new();
                match thing {
                    serde_json::Value::Object(a) => {
                        if let Some(al) = a.iter().next() {
                            let (_algo_name, algo) = al;
                            let algo_str = algo.to_string();
                            let algo: Algo = serde_json::from_str(&*algo_str).unwrap();
                            fee = algo;
                            }
                        // _ => {},

                    },
                    _ => {},
                }
                
                let thing = &blob["worksources"];
                let mut worksource = Algo::new();
                match thing {
                    serde_json::Value::Object(a) => {
                        if let Some(al) = a.iter().next() {
                            let (_algo_name, algo) = al;
                            let algo_str = algo[0].to_string();
                            let algo: Algo = serde_json::from_str(&*algo_str).unwrap();
                            worksource = algo;
                            }
                        // _ => {},

                    },
                    _ => {},
                }
        
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
                                    let ip = format!("{}", w["bmc"]["adc"]["inputPower"].to_string());
                                    let input_power: f32 = serde_json::from_str(&*ip).unwrap();
                                    let ac = format!("{}", w["bmc"]["adc"]["aux12VCurrent"].to_string());
                                    let aux_current: f32 = serde_json::from_str(&*ac).unwrap();

                                    let s = format!("{{ \"sysmon\": {} }}", w["sysmon"].to_string());
                                    let sysmons: SysMons = serde_json::from_str(&*s).unwrap();
        
                                    let c = format!("{{ \"cores\": {} }}", w["cores"].to_string());
                                    let cores: Cores = serde_json::from_str(&*c).unwrap();
        
                                    self.workers.push(
                                        Worker { 
                                            dna: w["dna"].as_str().unwrap().to_string(), 
                                            name: w["name"].as_str().unwrap().to_string(),
                                            input_power,
                                            input_power_health: w["bmc"]["health"]["inputPower"].as_str().unwrap().to_string(),
                                            aux_current,
                                            aux_current_health: w["bmc"]["health"]["inputCurrentAUX"].as_str().unwrap().to_string(),
                                            sysmons,
                                            cores,
                                            fee: fee.clone(),
                                            worksource: worksource.clone(),
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