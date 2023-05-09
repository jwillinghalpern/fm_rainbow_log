use std::str::FromStr;

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
        if s.starts_with('#') {
            let s = s.trim_start_matches('#');
            if s.len() != 6 {
                return Err("RGB color must be 6 characters long".into());
            }
            let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| "invalid RGB color")?;
            let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| "invalid RGB color")?;
            let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| "invalid RGB color")?;
            Ok(ColorType::RGB(r, g, b))
        } else if s.starts_with("rgb") {
            let s = s.trim_start_matches("rgb");
            let s = s.trim_start_matches('(');
            let s = s.trim_end_matches(')');
            let mut split = s.split(',');
            let r = split
                .next()
                .ok_or("invalid RGB color")?
                .trim()
                .parse::<u8>()
                .map_err(|_| "invalid RGB color")?;
            let g = split
                .next()
                .ok_or("invalid RGB color")?
                .trim()
                .parse::<u8>()
                .map_err(|_| "invalid RGB color")?;
            let b = split
                .next()
                .ok_or("invalid RGB color")?
                .trim()
                .parse::<u8>()
                .map_err(|_| "invalid RGB color")?;
            // println!("rgb: {} {} {}", r, g, b);
            Ok(ColorType::RGB(r, g, b))
        } else {
            Ok(ColorType::ANSI(s.to_string()))
        }
    }
}
