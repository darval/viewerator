use mylib::*;
use std::fs;

#[test]
fn parse_4bcu() {
    let mut wd = webdata::WebData::new();
    let input = fs::read_to_string("tests/data/4bcu.json").unwrap();
    wd.process_response(input);
    assert_eq!(wd.workers.len(), 4);
}

#[test]
fn parse_4bcu1() {
    let mut wd = webdata::WebData::new();
    let input = fs::read_to_string("tests/data/4bcu1.json").unwrap();
    wd.process_response(input);
    assert_eq!(wd.workers.len(), 4);
}

#[test]
fn parse_2bcu1cvp() {
    let mut wd = webdata::WebData::new();
    let input = fs::read_to_string("tests/data/2bcu1cvp.json").unwrap();
    wd.process_response(input);
    assert_eq!(wd.workers.len(), 3);
}

#[test]
fn parse_1cvp_00() {
    let mut wd = webdata::WebData::new();
    let input = fs::read_to_string("tests/data/1cvp_00.00.00.json").unwrap();
    wd.process_response(input);
    assert_eq!(wd.workers.len(), 1);
}
