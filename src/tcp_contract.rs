use super::BidAskTcpDateTime;

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

    pub fn parse(src: &str) -> Self {
        if src == "PING" {
            return Self::Ping;
        }
        if src == "PONG" {
            return Self::Pong;
        }

        Self::BidAsk(BidAskTcpModel::parse(src).unwrap())
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) {
        match self {
            BidAskTcpContract::Ping => dest.extend_from_slice(b"PING"),
            BidAskTcpContract::Pong => dest.extend_from_slice(b"PONG"),
            BidAskTcpContract::BidAsk(bid_ask) => bid_ask.serialize(dest),
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
    pub date_time: BidAskTcpDateTime,
    pub id: String,
    pub bid: String,
    pub ask: String,
}

impl BidAskTcpModel {
    pub fn parse(src: &str) -> Option<Self> {
        let mut date_time = None;
        let mut id = None;
        let mut bid = None;
        let mut ask = None;
        let mut no = 0;

        for line in src.split(' ') {
            match no {
                0 => id = Some(line.to_string()),
                1 => {
                    date_time = BidAskTcpDateTime::parse(line).into();
                }
                2 => bid = Some(line),
                3 => ask = Some(line),
                _ => {}
            }
            no += 1;
        }

        let date_time = date_time?;
        let id = id?;
        let bid = bid?;
        let ask = ask?;

        Self {
            date_time,
            id,
            bid: bid.to_string(),
            ask: ask.to_string(),
        }
        .into()
    }

    pub fn serialize(&self, dest: &mut Vec<u8>) {
        dest.extend_from_slice(self.id.as_bytes());

        dest.push(' ' as u8);
        self.date_time.serialize(dest);
        dest.push(' ' as u8);

        dest.extend_from_slice(self.bid.to_string().as_bytes());
        dest.push(' ' as u8);
        dest.extend_from_slice(self.ask.to_string().as_bytes());
    }
}

#[cfg(test)]
mod tests {

    use rust_extensions::date_time::DateTimeAsMicroseconds;
    use super::*;

    #[test]
    fn test_ser_der() {
        let bidask = BidAskTcpModel{
            ask: "1.13408000".to_string(),
            bid: "1.13401000".to_string(),
            id: "GBPUSD".to_string(),
            date_time: BidAskTcpDateTime::Our(DateTimeAsMicroseconds::now()),
        };
        let mut bidask_vec: Vec<u8> = Vec::new();
        bidask.serialize(&mut bidask_vec);
        let bidask_str = std::str::from_utf8(&bidask_vec).unwrap();
        let deserialized =
        BidAskTcpModel::parse(bidask_str).unwrap();

        let date_time = deserialized.date_time.unwrap_as_our_date();

        assert_eq!(deserialized.id, "GBPUSD");
        assert_eq!(deserialized.bid.to_string(), "1.13401000");
        assert_eq!(deserialized.ask.to_string(), "1.13408000");
        assert_eq!(date_time.to_rfc3339(), bidask.date_time.unwrap_as_our_date().to_rfc3339());
    }

    #[test]
    fn test_parse() {
        let result =
        BidAskTcpModel::parse("GBPUSD 20220921123348100 1.13401000 1.13408000").unwrap();

        let date_time = result.date_time.unwrap_as_our_date();

        assert_eq!(result.id, "GBPUSD");
        assert_eq!(result.bid.to_string(), "1.13401000");
        assert_eq!(result.ask.to_string(), "1.13408000");
        assert_eq!("2022-09-21T12:33:48.100", &date_time.to_rfc3339()[..23]);
    }

    #[test]
    fn test_our_time_parse() {
        let src = "GBPUSD 20220921123348000 1.13401000 1.13408000";
        let result = BidAskTcpModel::parse(src).unwrap();
        let date_time = result.date_time.unwrap_as_our_date();

        assert_eq!(result.id, "GBPUSD");
        assert_eq!(result.bid.to_string(), "1.13401000");
        assert_eq!(result.ask.to_string(), "1.13408000");
        assert_eq!("2022-09-21T12:33:48+00", &date_time.to_rfc3339()[..22]);
    }
    #[test]
    fn test_our_with_zero_ms() {
        let src = "GBPUSD 20220921123348000 1.13401000 1.13408000";
        let result = BidAskTcpModel::parse(src).unwrap();
        let date_time = result.date_time.unwrap_as_our_date();

        assert_eq!(result.id, "GBPUSD");
        assert_eq!(result.bid.to_string(), "1.13401000");
        assert_eq!(result.ask.to_string(), "1.13408000");
        assert_eq!("2022-09-21T12:33:48", &date_time.to_rfc3339()[..19]);
    }
}
