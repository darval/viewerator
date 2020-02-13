use log::*;
use reqwest;
use serde::Deserialize;
use serde_json;
use std::fs;

pub struct Worker {
    pub name: String,
    pub dna: String,
    pub input_power: f32,
    pub input_power_health: String,
    pub aux_current: f32,
    pub aux_current_health: String,
    pub pex_current: f32,
    pub pex_current_health: String,
    pub aux_12v: f32,
    pub aux_12v_health: String,
    pub pex_12v: f32,
    pub pex_12v_health: String,
    pub vccint: f32,
    pub vccint_current: f32,
    pub vccint_current_health: String,
    pub vrctrl_temp: f32,
    pub vrctrl_temp_health: String,
    pub phase0_status_global: u32,
    pub phase0_temperature: f32,
    pub phase0_temperature_health: String, 
    pub phase0_vout: f32,
    pub phase1_status_global: u32,
    pub phase1_temperature: f32,
    pub phase1_temperature_health: String, 
    pub phase1_vout: f32,
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
#[allow(non_snake_case)]
pub struct Clock {
    pub badNonces: f32,
    pub health: String,
    pub multiplier: f32,
    pub totalNonces: f32,
}

#[derive(Deserialize,Debug, Clone, Copy)]
#[allow(non_snake_case)]
pub struct StatDetail {
    pub accepted: f32,
    pub calculated: f32,
    pub endTime: f32,
    pub found: f32,
    pub requested: f32,
    pub startTime: f32,
    pub submitted: f32,
    pub valid: f32,
}

impl StatDetail {
    pub fn new() -> StatDetail {
        StatDetail {
            accepted: 0.0,
            calculated: 0.0,
            endTime: 0.0,
            found: 0.0,
            requested: 0.0,
            startTime: 0.0,
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
#[allow(non_snake_case)]
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
    pub workers: Vec<Worker>,
}

impl WebData {
    pub fn new() -> WebData {
        WebData {
            client: reqwest::blocking::Client::new(),
            minerator: "None".to_string(),
            workers: vec!(),
        }
    }
    
    pub fn getdata<'a>(&mut self,matches: &clap::ArgMatches<'a>) {
        let input_file = matches.value_of("input_file").unwrap_or("");
        if input_file != "" {
            let input = fs::read_to_string(input_file).unwrap();
            self.process_response(input);
        } else {
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
                    self.process_response(response);
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

    fn process_response(&mut self, response: String) {
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
                            let pc = format!("{}", w["bmc"]["adc"]["pex12VCurrent"].to_string());
                            let pex_current: f32 = serde_json::from_str(&*pc).unwrap();
                            let av = format!("{}", w["bmc"]["adc"]["aux12V"].to_string());
                            let aux_12v: f32 = serde_json::from_str(&*av).unwrap();
                            let pv = format!("{}", w["bmc"]["adc"]["pex12V"].to_string());
                            let pex_12v: f32 = serde_json::from_str(&*pv).unwrap();
                            let vi = format!("{}", w["bmc"]["adc"]["vccint"].to_string());
                            let vccint: f32 = serde_json::from_str(&*vi).unwrap();
                            let vc = format!("{}", w["bmc"]["adc"]["vccintCurrent"].to_string());
                            let vccint_current: f32 = serde_json::from_str(&*vc).unwrap();
                            let vt = format!("{}", w["bmc"]["temperature"].to_string());
                            let vrctrl_temp: f32 = serde_json::from_str(&*vt).unwrap();
                            let psg = format!("{}", w["bmc"]["phases"][0]["statusGlobal"].to_string());
                            let phase0_status_global: u32 = serde_json::from_str(&*psg).unwrap();
                            let pt = format!("{}", w["bmc"]["phases"][0]["temperature"].to_string());
                            let phase0_temperature: f32 = serde_json::from_str(&*pt).unwrap();
                            let pv = format!("{}", w["bmc"]["phases"][0]["vout"].to_string());
                            let phase0_vout: f32 = serde_json::from_str(&*pv).unwrap();
                            let psg = format!("{}", w["bmc"]["phases"][1]["statusGlobal"].to_string());
                            let phase1_status_global: u32 = serde_json::from_str(&*psg).unwrap();
                            let pt = format!("{}", w["bmc"]["phases"][1]["temperature"].to_string());
                            let phase1_temperature: f32 = serde_json::from_str(&*pt).unwrap();
                            let pv = format!("{}", w["bmc"]["phases"][1]["vout"].to_string());
                            let phase1_vout: f32 = serde_json::from_str(&*pv).unwrap();

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
                                    pex_current,
                                    pex_current_health: w["bmc"]["health"]["inputCurrentPEX"].as_str().unwrap().to_string(),
                                    aux_12v,
                                    aux_12v_health: w["bmc"]["health"]["inputVoltageAUX"].as_str().unwrap().to_string(),
                                    pex_12v,
                                    pex_12v_health: w["bmc"]["health"]["inputVoltagePEX"].as_str().unwrap().to_string(),
                                    vccint,
                                    vccint_current,
                                    vccint_current_health: w["bmc"]["health"]["vccintCurrent"].as_str().unwrap().to_string(),
                                    vrctrl_temp,
                                    vrctrl_temp_health: w["bmc"]["health"]["vrCtrl"].as_str().unwrap().to_string(),
                                    phase0_status_global,
                                    phase0_temperature,
                                    phase0_temperature_health: w["bmc"]["health"]["vrPower"].as_str().unwrap().to_string(),
                                    phase0_vout,
                                    phase1_status_global,
                                    phase1_temperature,
                                    phase1_temperature_health: w["bmc"]["health"]["vrPower"].as_str().unwrap().to_string(),
                                    phase1_vout,
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

    }
}