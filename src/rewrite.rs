use nom::error::ParseError;
use nom::sequence::separated_pair;
use nom::{
    branch::alt, character::complete::char, combinator::map, number::complete::double, IResult,
};

use crate::utility::alternating_list;

pub trait Parsable: Sized {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Self, E>;
}

pub trait Evaluate {
    fn evaluate(&self) -> f64;
}

#[derive(Debug)]
pub struct Exponent {
    base: f64,
    power: Box<Degree>,
}

impl Parsable for Exponent {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Self, E> {
        map(
            separated_pair(double, char('^'), Degree::parse),
            |(base, power)| Exponent {
                base,
                power: Box::new(power),
            },
        )(i)
    }
}

#[derive(Debug)]
pub enum Degree {
    Exponent(Exponent),
    Number(f64),
}

impl Parsable for Degree {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Self, E> {
        alt((
            map(Exponent::parse, Degree::Exponent),
            map(double, Degree::Number),
        ))(i)
    }
}

impl Evaluate for Degree {
    fn evaluate(&self) -> f64 {
        match self {
            Degree::Exponent(Exponent { base, power }) => base.powf(power.evaluate()),
            Degree::Number(n) => *n,
        }
    }
}

#[derive(Debug)]
pub enum TermOperator {
    Times,
    Divide,
}

impl Parsable for TermOperator {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Self, E> {
        alt((
            map(char('*'), |_| TermOperator::Times),
            map(char('/'), |_| TermOperator::Divide),
        ))(i)
    }
}

#[derive(Debug)]
pub enum TermPart {
    Operator(TermOperator),
    Degree(Degree),
}

impl TermPart {
    fn degree(&self) -> &Degree {
        match self {
            Self::Degree(degree) => degree,
            _ => panic!("malformed expression"),
        }
    }
}

pub type Term = Vec<TermPart>;

impl Parsable for Term {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Self, E> {
        alternating_list(
            map(TermOperator::parse, TermPart::Operator),
            map(Degree::parse, TermPart::Degree),
        )(i)
    }
}

impl Evaluate for Term {
    fn evaluate(&self) -> f64 {
        use TermOperator::*;
        use TermPart::*;

        assert!(self.len() > 0, "malformed equation");
        let chunks = self[1..].chunks_exact(2);
        assert!(chunks.remainder().len() == 0, "malformed equation");

        chunks.fold(self[0].degree().evaluate(), |acc, chunk| {
            if let [Operator(op), Degree(degree)] = chunk {
                match op {
                    Times => acc * degree.evaluate(),
                    Divide => acc / degree.evaluate(),
                }
            } else {
                panic!("malformed expression")
            }
        })
    }
}

#[derive(Debug)]
pub enum EquationOperator {
    Plus,
    Minus,
}

impl Parsable for EquationOperator {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Self, E> {
        alt((
            map(char('+'), |_| EquationOperator::Plus),
            map(char('-'), |_| EquationOperator::Minus),
        ))(i)
    }
}

#[derive(Debug)]
pub enum EquationPart {
    Operator(EquationOperator),
    Term(Term),
}

impl EquationPart {
    fn term(&self) -> &Term {
        match self {
            Self::Term(term) => term,
            _ => panic!("malformed expression"),
        }
    }
}

pub type Equation = Vec<EquationPart>;

impl Parsable for Equation {
    fn parse<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Self, E> {
        alternating_list(
            map(EquationOperator::parse, EquationPart::Operator),
            map(Term::parse, EquationPart::Term),
        )(i)
    }
}

impl Evaluate for Equation {
    fn evaluate(&self) -> f64 {
        use EquationOperator::*;
        use EquationPart::*;

        assert!(self.len() > 0, "malformed equation");
        let chunks = self[1..].chunks_exact(2);
        assert!(chunks.remainder().len() == 0, "malformed equation");

        chunks.fold(self[0].term().evaluate(), |acc, chunk| {
            if let [Operator(op), Term(term)] = chunk {
                match op {
                    Plus => acc + term.evaluate(),
                    Minus => acc - term.evaluate(),
                }
            } else {
                panic!("malformed expression")
            }
        })
    }
}
