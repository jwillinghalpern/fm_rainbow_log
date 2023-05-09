use std::str::FromStr;

#[cfg_attr(test, derive(PartialEq, Debug))]
pub(crate) enum ColorType {
    RGB(u8, u8, u8),
    ANSI(String),
}

impl Default for ColorType {
    fn default() -> Self {
        ColorType::ANSI("".to_string())
    }
}

impl FromStr for ColorType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim().to_lowercase();
        let invalid_rgb_msg = "invalid RGB color";
        let invalid_hex_msg = "invalid hex color";
        if s.starts_with('#') {
            let s = s.trim_start_matches('#');
            if s.len() != 6 {
                return Err("RGB color must be 6 characters long".into());
            }
            let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| invalid_hex_msg)?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| invalid_hex_msg)?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| invalid_hex_msg)?;
            Ok(ColorType::RGB(r, g, b))
        } else if s.starts_with("rgb") {
            let s = s.trim_start_matches("rgb");
            let s = s.trim_start_matches(" ").trim_start_matches('(');
            let s = s.trim_end_matches(" ").trim_end_matches(')');
            let mut split = s.split(',');
            let r = split
                .next()
                .ok_or(invalid_rgb_msg)?
                .trim()
                .parse::<u8>()
                .map_err(|_| invalid_rgb_msg)?;
            let g = split
                .next()
                .ok_or(invalid_rgb_msg)?
                .trim()
                .parse::<u8>()
                .map_err(|_| invalid_rgb_msg)?;
            let b = split
                .next()
                .ok_or(invalid_rgb_msg)?
                .trim()
                .parse::<u8>()
                .map_err(|_| invalid_rgb_msg)?;
            Ok(ColorType::RGB(r, g, b))
        } else {
            Ok(ColorType::ANSI(s.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb() {
        let rgb = ColorType::from_str("rgb(255, 255, 255)").unwrap();
        assert_eq!(rgb, ColorType::RGB(255, 255, 255));

        let rgb = ColorType::from_str("     rgb    (   0   , 0   , 0)       ");
        assert_eq!(rgb, Ok(ColorType::RGB(0, 0, 0)));
    }
    #[test]
    fn test_rgb_invalid_num() {
        let rgb = ColorType::from_str("rgb(0, 0, 256)");
        assert!(rgb.is_err());
        let rgb = ColorType::from_str("rgb(0, 0, -1)");
        assert!(rgb.is_err())
    }

    #[test]
    fn test_hex() {
        let rgb = ColorType::from_str("#ffffff").unwrap();
        assert_eq!(rgb, ColorType::RGB(255, 255, 255));
    }
    #[test]
    fn test_hex_invalid() {
        let rgb = ColorType::from_str("#ffff");
        assert!(rgb.is_err());
        let rgb = ColorType::from_str("#fffffg");
        assert!(rgb.is_err());
    }

    #[test]
    fn test_ansi() {
        let rgb = ColorType::from_str("red").unwrap();
        assert_eq!(rgb, ColorType::ANSI("red".to_string()));
    }
}
