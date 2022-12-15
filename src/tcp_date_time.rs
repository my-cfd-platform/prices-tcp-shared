use chrono::{Datelike, Timelike};
use rust_extensions::date_time::DateTimeAsMicroseconds;

const OUR_MARKER: u8 = 'O' as u8;

#[derive(Debug, Clone)]
pub enum BidAskTcpDateTime {
    Our(DateTimeAsMicroseconds),
}

impl BidAskTcpDateTime {
    pub fn unwrap_as_our_date(&self) -> &DateTimeAsMicroseconds {
        match self {
            BidAskTcpDateTime::Our(data) => data,
        }
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) {
        match &self {
            BidAskTcpDateTime::Our(date_time) => {
                dest.push(OUR_MARKER as u8);
                date_time_to_string(dest, date_time);
            }
        };
    }

    pub fn parse(src: &str) -> Self {
        let date_time = parse_date_time(&src);
        BidAskTcpDateTime::Our(date_time)
    }
}

fn parse_date_time(line: &str) -> DateTimeAsMicroseconds {
    let year: i32 = line[1..5].parse().unwrap();
    let month: u32 = line[5..7].parse().unwrap();
    let day: u32 = line[7..9].parse().unwrap();
    let hour: u32 = line[9..11].parse().unwrap();
    let min: u32 = line[11..13].parse().unwrap();
    let sec: u32 = line[13..15].parse().unwrap();

    let millis_str = &line[15..];
    let mut millis: i64 = millis_str.parse().unwrap();

    match millis_str.len() {
        1 => {
            millis *= 100_000;
        }
        2 => {
            millis *= 10_000;
        }
        3 => {
            millis *= 1_000;
        }
        4 => {
            millis *= 100;
        }
        5 => {
            millis *= 10;
        }
        _ => {}
    }

    DateTimeAsMicroseconds::create(year, month, day, hour, min, sec, millis)
}

fn date_time_to_string(result: &mut Vec<u8>, dt: &DateTimeAsMicroseconds) {
    let dt = dt.to_chrono_utc();

    result.extend_from_slice(dt.year().to_string().as_bytes());

    push_with_leading_zero(result, dt.month() as u8);
    push_with_leading_zero(result, dt.day() as u8);
    push_with_leading_zero(result, dt.hour() as u8);
    push_with_leading_zero(result, dt.minute() as u8);
    push_with_leading_zero(result, dt.second() as u8);

    let mut ms_as_string = dt.nanosecond().to_string();

    let ms_as_slice = if ms_as_string.len() < 6 {
        while ms_as_string.len() < 3 {
            ms_as_string.push('0');
        }

        &ms_as_string
    } else {
        &ms_as_string[..6]
    };

    result.extend_from_slice(ms_as_slice.as_bytes());
}

fn push_with_leading_zero(result: &mut Vec<u8>, value: u8) {
    if value < 10 {
        result.push('0' as u8);
        let value = '0' as u8 + value;
        result.push(value);
    } else {
        result.extend_from_slice(value.to_string().as_bytes());
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse() {
        let result = BidAskTcpDateTime::parse("GBPUSD 20220921123348100 1.13401000 1.13408000");

        let date_time = result.unwrap_as_our_date();

        assert_eq!("2022-09-21T12:33:48.100", &date_time.to_rfc3339()[..23]);
    }
}
