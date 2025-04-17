use std::fmt;
use std::fmt::Formatter;

#[derive(Clone)]
pub enum SessionType {
    TDS,
    MySQL,
    FlightSQL,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let name = match self {
            SessionType::TDS => "TDS".to_string(),
            SessionType::MySQL => "MySQL".to_string(),
            SessionType::FlightSQL => "FlightSQL".to_string(),
        };
        write!(f, "{}", name)
    }
}
