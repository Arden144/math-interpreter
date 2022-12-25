#![feature(decl_macro)]
#![feature(type_alias_impl_trait)]
#![feature(option_result_contains)]
#![allow(dead_code)]
#![feature(test)]

extern crate test;

mod rewrite;
mod utility;

use std::io;

use nom::error::VerboseError;
use rand::distributions::Standard;
use rand::prelude::Distribution;
use rand::Rng;

use crate::rewrite::{Equation, Evaluate, Parsable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Plus,
    Minus,
    Times,
    Divide,
    Exponent,
}

impl Distribution<Op> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Op {
        match rng.gen_range(0..5) {
            0 => Op::Plus,
            1 => Op::Minus,
            2 => Op::Times,
            3 => Op::Divide,
            _ => Op::Exponent,
        }
    }
}

fn main() {
    let mut input = String::new();

    use std::io::Write;
    print!("Enter equation: ");
    io::stdout().flush().unwrap();

    io::stdin().read_line(&mut input).unwrap();
    input.retain(|c| !c.is_whitespace());

    println!(
        "{} = {:#?}",
        &input,
        Equation::parse::<VerboseError<&str>>(&input)
            .unwrap()
            .1
            .evaluate()
    );
}

// 5x^2 - 4x + 2

//           Equation
// 5x^2          -    4x    +    2
// 5 * (x ^ 2)      4 * x
//      x ^ (2)
//           2

#[cfg(test)]
mod tests {
    use crate::rewrite::{Equation, Parsable};

    use super::*;
    use rand::{rngs::StdRng, SeedableRng};
    use std::fmt::Write;
    use test::Bencher;

    fn build_string<const N: usize>() -> String {
        let mut rng = StdRng::seed_from_u64(0);
        let mut str = String::with_capacity(12 * N - 1);

        write!(&mut str, "{}", rng.gen::<i32>()).unwrap();

        for _ in 1..N {
            str.push(match rng.gen::<Op>() {
                Op::Plus => '+',
                Op::Minus => '-',
                Op::Times => '*',
                Op::Divide => '/',
                Op::Exponent => '^',
            });
            write!(&mut str, "{}", rng.gen::<i32>()).unwrap();
        }

        str
    }

    #[test]
    fn test_parse() {
        let str = build_string::<100_000>();

        assert!(Equation::parse::<()>(&str).unwrap().0.len() == 0);
    }

    #[bench]
    fn bench_parse(b: &mut Bencher) {
        let str = build_string::<10_000>();

        b.iter(|| Equation::parse::<()>(&str).unwrap());
    }

    #[bench]
    fn bench_evaluate(b: &mut Bencher) {
        let str = build_string::<10_000>();
        let equation = Equation::parse::<()>(&str).unwrap().1;

        b.iter(|| equation.evaluate());
    }
}
