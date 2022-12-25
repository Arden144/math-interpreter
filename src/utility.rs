use std::ops::{Range, RangeFrom};

use nom::{
    error::{ErrorKind, ParseError, VerboseError},
    AsChar, IResult, InputIter, InputLength, Parser, Slice,
};

pub fn inside_parens<I, O, E, F>(mut f: F) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: InputIter + Slice<RangeFrom<usize>> + Slice<Range<usize>>,
    <I as InputIter>::Item: AsChar,
    F: Parser<I, O, E>,
    E: ParseError<I>,
{
    move |input: I| {
        let mut depth = 1;

        let mut iter = input.iter_indices();

        match iter.next().map(|(_, c)| c.as_char()) {
            Some(c) if c == '(' => (),
            _ => return Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Alt))),
        };

        for (i, c) in iter {
            match c.as_char() {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => (),
            };

            if depth == 0 {
                let inner = input.slice(1..i);
                let rest = input.slice(i + 1..);
                let output = f.parse(inner)?.1;
                return Ok((rest, output));
            }
        }

        Err(nom::Err::Error(E::from_error_kind(input, ErrorKind::Alpha)))
    }
}

pub fn alternating_list<I, O, E, F, G>(
    mut sep: G,
    mut f: F,
) -> impl FnMut(I) -> IResult<I, Vec<O>, E>
where
    I: Clone + InputLength,
    F: Parser<I, O, E>,
    G: Parser<I, O, E>,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut res = Vec::new();

        match f.parse(i.clone()) {
            Err(nom::Err::Error(_)) => return Ok((i, res)),
            Err(e) => return Err(e),
            Ok((i1, o)) => {
                res.push(o);
                i = i1;
            }
        }

        loop {
            let len = i.input_len();
            match sep.parse(i.clone()) {
                Err(nom::Err::Error(_)) => return Ok((i, res)),
                Err(e) => return Err(e),
                Ok((i1, s)) => {
                    // infinite loop check: the parser must always consume
                    if i1.input_len() == len {
                        return Err(nom::Err::Error(E::from_error_kind(
                            i1,
                            ErrorKind::SeparatedList,
                        )));
                    }

                    match f.parse(i1.clone()) {
                        Err(nom::Err::Error(_)) => return Ok((i, res)),
                        Err(e) => return Err(e),
                        Ok((i2, o)) => {
                            res.push(s);
                            res.push(o);
                            i = i2;
                        }
                    }
                }
            }
        }
    }
}

fn complete<I, O, F>(i: I, mut f: F) -> IResult<I, O, VerboseError<I>>
where
    F: Parser<I, O, VerboseError<I>>,
{
    f.parse(i)
}

#[cfg(test)]
mod tests {
    use nom::error::convert_error;
    use nom::multi::many0;
    use nom::Finish;

    use nom::bytes::complete::take;

    use super::{complete, inside_parens};

    #[test]
    fn test_inside_parens() {
        let data = "((1 + 1))))()())((()";

        let result = complete(data, inside_parens(many0(take(1usize))))
            .finish()
            .map_err(|e| {
                let str = convert_error(data, e);
                println!("{}", str);
                str
            })
            .unwrap();

        assert_eq!(
            result,
            ("))()())((()", vec!["(", "1", " ", "+", " ", "1", ")"])
        );
    }
}
