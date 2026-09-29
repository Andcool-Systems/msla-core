use std::io::{Error, ErrorKind};
use tokio::fs;

/// Get current cpu temp in celsius
pub async fn get_temp() -> Result<i32, Error> {
    let str = fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").await?;
    let mut temp = str
        .trim()
        .parse::<i32>()
        .map_err(|e| Error::new(ErrorKind::InvalidData, e))?;

    if temp > 200 {
        temp /= 1000;
    }

    Ok(temp)
}
