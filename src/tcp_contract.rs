use rust_extensions::date_time::DateTimeAsMicroseconds;

pub const LINE_SPLITTER: &[u8; 1] = b" ";

#[derive(Debug, Clone)]
pub enum BidAskTcpContract {
    Ping,
    Pong,
    BidAsk(BidAskTcpModel),
}

impl BidAskTcpContract {
    pub fn is_ping(&self) -> bool {
        match self {
            BidAskTcpContract::Ping => true,
            _ => false,
        }
    }

    pub fn parse(src: &[u8]) -> Result<Self, String> {
        if src == b"PING" {
            return Ok(Self::Ping);
        }
        if src == b"PONG" {
            return Ok(Self::Pong);
        }

        Ok(Self::BidAsk(BidAskTcpModel::deserialize(src)?))
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) -> Result<(), String> {
        match self {
            BidAskTcpContract::Ping => Ok(dest.extend_from_slice(b"PING")),
            BidAskTcpContract::Pong => Ok(dest.extend_from_slice(b"PONG")),
            BidAskTcpContract::BidAsk(bid_ask) => {
                dest.extend_from_slice(bid_ask.serialize()?.as_slice());
                Ok(())
            }
        }
    }

    pub fn is_bid_ask(&self) -> bool {
        match self {
            BidAskTcpContract::Ping => false,
            BidAskTcpContract::Pong => false,
            BidAskTcpContract::BidAsk(_) => true,
        }
    }
}

impl my_tcp_sockets::tcp_connection::TcpContract for BidAskTcpContract {
    fn is_pong(&self) -> bool {
        match self {
            BidAskTcpContract::Pong => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BidAskTcpModel {
    pub date: DateTimeAsMicroseconds,
    pub id: String,
    pub bid_price: String,
    pub ask_price: String,
    pub bid_volume: String,
    pub ask_volume: String,
}

impl BidAskTcpModel {
    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let mut result = Vec::new();

        result.extend_from_slice(self.id.as_bytes()); // 1

        result.extend_from_slice(LINE_SPLITTER);        
        let mut date = serialize_date(&self.date)?;
        result.append(&mut date); // 2

        result.extend_from_slice(LINE_SPLITTER);
        result.extend_from_slice(format!("{}", self.bid_price).as_bytes()); // 3

        result.extend_from_slice(LINE_SPLITTER);
        result.extend_from_slice(format!("{}", self.bid_volume).as_bytes()); // 4

        result.extend_from_slice(LINE_SPLITTER);
        result.extend_from_slice(format!("{}", self.ask_price).as_bytes()); // 5

        result.extend_from_slice(LINE_SPLITTER);
        result.extend_from_slice(format!("{}", self.ask_volume).as_bytes()); // 6

        return Ok(result);
    }

    pub fn deserialize(src: &[u8]) -> Result<Self, String> {
        let chunks = src
            .split(|x| *x == LINE_SPLITTER[0])
            .collect::<Vec<&[u8]>>();

        let instrument = String::from_utf8(chunks[0].to_vec()).unwrap();
        let date = deserialize_date(chunks[1])?;
        let bid_price = String::from_utf8(chunks[2].to_vec()).unwrap();
        let bid_volume = String::from_utf8(chunks[3].to_vec()).unwrap();
        let ask_price = String::from_utf8(chunks[4].to_vec()).unwrap();
        let ask_volume = String::from_utf8(chunks[5].to_vec()).unwrap();

        Ok(Self {
            id: instrument,
            bid_price,
            bid_volume,
            ask_price,
            ask_volume,
            date,
        })
    }
}

pub fn serialize_date(date: &DateTimeAsMicroseconds) -> Result<Vec<u8>, String> {
    Ok(date.to_rfc3339().as_bytes().to_vec())
}

pub fn deserialize_date(date: &[u8]) -> Result<DateTimeAsMicroseconds, String> {
    let date_str = String::from_utf8(date.to_vec());

    let Ok(date_str) = date_str else {
        return Err("Failed to format to utf8".to_string());
    };

    let Some(date) = DateTimeAsMicroseconds::parse_iso_string(&date_str) else {
        return Err("Failed to parse str".to_string());
    };

    Ok(date)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    #[test]
    fn test_ser_der() {
        let bidask = BidAskTcpModel {
            ask_price: "1.13408000".to_string(),
            bid_price: "1.13401000".to_string(),
            id: "GBPUSD".to_string(),
            date: DateTimeAsMicroseconds::now(),
            bid_volume: "1.14545".to_string(),
            ask_volume: "2.16566".to_string(),
        };

        let serialized = bidask.serialize();
        let deserialized = BidAskTcpModel::deserialize(serialized.unwrap().as_slice()).unwrap();

        assert_eq!(deserialized.id, "GBPUSD");
        assert_eq!(deserialized.bid_price.to_string(), "1.13401000");
        assert_eq!(deserialized.ask_price.to_string(), "1.13408000");
        assert_eq!(deserialized.bid_volume.to_string(), "1.14545");
        assert_eq!(deserialized.ask_volume.to_string(), "2.16566");
        assert_eq!(
            deserialized.date.unix_microseconds,
            bidask.date.unix_microseconds
        );
    }

    #[test]
    fn test_parse() {
        let date = DateTimeAsMicroseconds::now();
        let data_str = format!("GBPUSD {} 1.13401000 1.1 1.13408000 1.2", date.to_rfc3339());

        let deserialized = BidAskTcpModel::deserialize(data_str.as_bytes()).unwrap();

        assert_eq!(deserialized.id, "GBPUSD");
        assert_eq!(deserialized.bid_price.to_string(), "1.13401000");
        assert_eq!(deserialized.ask_price.to_string(), "1.13408000");
        assert_eq!(date.to_rfc3339(), deserialized.date.to_rfc3339());
    }
}
