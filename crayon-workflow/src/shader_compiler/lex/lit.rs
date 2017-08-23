use std::str::FromStr;
use std::{str, i64, f64};

use nom::digit;

#[derive(Debug, Clone, PartialEq)]
pub enum Literial {
    Int(i64),
    Float(f64),
}

fn from_full_dec(digits: &[u8]) -> Literial {
    let s = str::from_utf8(digits).unwrap();
    if let Ok(n) = i64::from_str_radix(s, 10) {
        Literial::Int(n)
    } else {
        let f = f64::from_str(s).unwrap();
        Literial::Float(f)
    }
}

named!(num_lit<Literial>,
    map!(
        recognize!(
            tuple!(
               digit,
               opt!(complete!(preceded!(tag!("."), digit)))
            )
        ),
        from_full_dec
    )
);

named!(pub parse<Literial>, alt!(num_lit));

#[cfg(test)]
mod tests {
    use super::{parse, Literial};
    use nom::*;

    #[test]
    fn num_lit() {
        let good_inputs = ["1", "123", "12.123123", "0.123123"];
        let bad_inputs = ["f5", "13f5", "10.3.1.4", "50e"];

        let good_outputs = vec![
            Literial::Int(1),
            Literial::Int(123),
            Literial::Float(12.123123),
            Literial::Float(0.123123),
        ];

        let bad_outputs = vec![
            IResult::Error(ErrorKind::Alt),
            IResult::Done(&b"f5"[..], Literial::Int(13)),
            IResult::Done(&b".1.4"[..], Literial::Float(10.3)),
            IResult::Done(&b"e"[..], Literial::Int(50)),
        ];

        for (input, expected) in good_inputs.iter().zip(good_outputs.into_iter()) {
            assert_eq!(parse(input.as_bytes()), IResult::Done(&b""[..], expected));
        }

        for (input, expected) in bad_inputs.iter().zip(bad_outputs.into_iter()) {
            assert_eq!(parse(input.as_bytes()), expected);
        }
    }
}