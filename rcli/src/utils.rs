use anyhow::Result;
use chrono::{Duration, Local};
use std::{fs::File, io::Read};

pub fn get_reader(input: &str) -> anyhow::Result<Box<dyn Read>> {
    let reader: Box<dyn Read> = if input == "-" {
        Box::new(std::io::stdin())
    } else {
        Box::new(File::open(input)?)
    };
    Ok(reader)
}

pub fn current_timestamp_sec() -> i64 {
    Local::now().timestamp()
}

pub fn parse_str_to_timestamp(duration: &str) -> Result<i64> {
    let Some(end) = duration.chars().last() else {
        anyhow::bail!("Empty duration");
    };

    match end {
        'd' => {
            let days = duration.trim_end_matches('d').parse::<i64>()?;
            let timestamp = Local::now() + Duration::days(days);
            Ok(timestamp.timestamp())
        }
        'h' => {
            let hours = duration.trim_end_matches('h').parse::<i64>()?;
            let timestamp = Local::now() + Duration::hours(hours);
            Ok(timestamp.timestamp())
        }
        'm' => {
            let minutes = duration.trim_end_matches('m').parse::<i64>()?;
            let timestamp = Local::now() + Duration::minutes(minutes);
            Ok(timestamp.timestamp())
        }
        's' => {
            let seconds = duration.trim_end_matches('s').parse::<i64>()?;
            let timestamp = Local::now() + Duration::seconds(seconds);
            Ok(timestamp.timestamp())
        }

        v => anyhow::bail!("Unsupported format: {}", v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_str_to_timestamp_should_works() {
        let timestamp1 = parse_str_to_timestamp("60s").unwrap();
        let timestamp2 = current_timestamp_sec() + 60;
        assert_eq!(timestamp1, timestamp2);

        let timestamp1 = parse_str_to_timestamp("1m").unwrap();
        let timestamp2 = current_timestamp_sec() + 60;
        assert_eq!(timestamp1, timestamp2);

        let timestamp1 = parse_str_to_timestamp("1h").unwrap();
        let timestamp2 = current_timestamp_sec() + 60 * 60;
        assert_eq!(timestamp1, timestamp2);

        let timestamp1 = parse_str_to_timestamp("1d").unwrap();
        let timestamp2 = current_timestamp_sec() + 24 * 60 * 60;
        assert_eq!(timestamp1, timestamp2);
    }

    #[test]
    fn test_parse_str_to_timestamp_should_err() {
        let Err(err) = parse_str_to_timestamp("fjsdaflasdj") else {
            panic!(r#"parse_str_to_timestamp("fjsdaflasdj") should error but got None"#)
        };
        assert_eq!(err.to_string(), "Unsupported format: j");

        let Err(err) = parse_str_to_timestamp("") else {
            panic!(r#"parse_str_to_timestamp("") should error but got None"#)
        };
        assert_eq!(err.to_string(), "Empty duration")
    }
}
