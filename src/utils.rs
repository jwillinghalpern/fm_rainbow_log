use iso8601::parsers::parse_datetime;

// NOTE: in some quick benchmarks, this fn is about twice as fast as `replace_trailing_cr_with_crlf`
// TODO: try operating on .bytes() instead of .chars()
// TODO: take &mut str
pub(crate) fn replace_trailing_cr_with_crlf_bytes(buf: &mut String) {
    let mut prev_byte = 0;
    let mut new_buf = Vec::with_capacity(buf.len() + 8);
    // I ended up having to use buf.chars() instead of buf.bytes() to preserve "smart" quotes, ugh
    for c in buf.bytes() {
        let curr_byte = c;
        if prev_byte == 13 && curr_byte != 10 {
            new_buf.push(10);
        }
        new_buf.push(c);
        prev_byte = curr_byte;
    }
    *buf = unsafe { String::from_utf8_unchecked(new_buf) };
}

// TODO: try operating on .bytes() instead of .chars()
// TODO: take &mut str
pub(crate) fn replace_trailing_cr_with_crlf(buf: &mut String) {
    let mut prev_byte = 0;
    let mut new_buf = String::new();
    // I ended up having to use buf.chars() instead of buf.bytes() to preserve "smart" quotes, ugh
    for c in buf.chars() {
        let curr_byte = c as u8;
        if prev_byte == 13 && curr_byte != 10 {
            new_buf.push(10 as char);
        }
        new_buf.push(c);
        prev_byte = curr_byte;
    }
    *buf = new_buf;
}

pub(crate) fn is_timestamp(s: &str) -> bool {
    let s = s.replace(' ', "T");
    parse_datetime(s.as_bytes()).is_ok()
}

pub(crate) fn clear_terminal() {
    let _ = clearscreen::clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replace_trailing_cr_with_crlf() {
        let mut buf = String::from("foo\rbar\r\nbaz\nbevis\n");
        replace_trailing_cr_with_crlf(&mut buf);
        assert_eq!(buf, "foo\r\nbar\r\nbaz\nbevis\n");
    }

    #[test]
    fn test_is_timestamp() {
        assert!(is_timestamp("2021-01-01 00:00:00.000Z"));
        assert!(is_timestamp("2021-01-01T00:00:00.000Z"));
        assert!(!is_timestamp("foo"));
    }
}
