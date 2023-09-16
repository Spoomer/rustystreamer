use std::cmp;

use actix_web::http::header::HeaderValue;
use crate::util::MultiThreadableError;

pub struct RangeHeader {
    pub unit: String,
    pub start: u64,
    pub end: u64,
}

impl RangeHeader {
    pub fn parse(range: &HeaderValue, size: u64) -> Result<Self, Box<MultiThreadableError>> {
        let range: Vec<&str> = range.to_str()?.split(&['=', '-'][..]).collect();
        if range.len() < 2 {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid Range-Header",
            )))
        } else if range.len() < 3 || range[2].is_empty() {
            Ok(Self {
                unit: range[0].to_string(),
                start: range[1].parse()?,
                end: cmp::min(
                    range[1].parse::<u64>()? + super::consts::CHUNK_SIZE as u64 - 1,
                    size - 1,
                ),
            })
        } else {
            Ok(Self {
                unit: range[0].to_string(),
                start: range[1].parse()?,
                end: cmp::min(range[2].parse()?, size - 1),
            })
        }
    }
}
