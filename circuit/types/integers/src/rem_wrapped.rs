// Copyright (C) 2019-2023 Aleo Systems Inc.
// This file is part of the snarkVM library.

// The snarkVM library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The snarkVM library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the snarkVM library. If not, see <https://www.gnu.org/licenses/>.

use super::*;

impl<E: Environment, I: IntegerType> RemWrapped<Self> for Integer<E, I> {
    type Output = Self;

    #[inline]
    fn rem_wrapped(&self, other: &Integer<E, I>) -> Self::Output {
        match (self.is_constant(), other.is_constant()) {
            // If `other` is a constant and is zero, then halt.
            (_, true) if other.eject_value().is_zero() => E::halt("Attempted to divide by zero."),
            // If `self` and `other` are constants, and other is not zero, then directly return the remainder.
            (true, true) => witness!(|self, other| console::Integer::new(self.wrapping_rem(&other))),
            // Handle the remaining cases.
            // Note that `other` is either a constant and non-zero, or not a constant.
            _ => {
                if I::is_signed() {
                    // Divide the absolute value of `self` and `other` in the base field.
                    let unsigned_dividend = self.abs_wrapped().cast_as_dual();
                    // Note that `unsigned_divisor` is zero iff `other` is zero.
                    let unsigned_divisor = other.abs_wrapped().cast_as_dual();
                    // Note that the call to `rem_wrapped` checks that `unsigned_divisor` is not zero.
                    let unsigned_remainder = unsigned_dividend.rem_wrapped(&unsigned_divisor);

                    let signed_remainder = Self { bits_le: unsigned_remainder.bits_le, phantom: Default::default() };

                    // The remainder takes on the same sign as `self` because the division operation rounds towards zero.
                    Self::ternary(&!self.msb(), &signed_remainder, &Self::zero().sub_wrapped(&signed_remainder))
                } else {
                    self.unsigned_division_via_witness(other).1
                }
            }
        }
    }
}

impl<E: Environment, I: IntegerType> Metrics<dyn RemWrapped<Integer<E, I>, Output = Integer<E, I>>> for Integer<E, I> {
    type Case = (Mode, Mode);

    fn count(case: &Self::Case) -> Count {
        match (case.0, case.1) {
            (Mode::Constant, Mode::Constant) => Count::is(I::BITS, 0, 0, 0),
            (Mode::Constant, _) | (_, Mode::Constant) => {
                match (I::is_signed(), 2 * I::BITS < E::BaseField::size_in_data_bits() as u64) {
                    (true, true) => Count::less_than(5 * I::BITS + 1, 0, (9 * I::BITS) + 5, (9 * I::BITS) + 11),
                    (true, false) => Count::less_than(5 * I::BITS + 1, 0, 1610, 1813),
                    (false, true) => Count::less_than(I::BITS + 1, 0, (3 * I::BITS) + 2, (3 * I::BITS) + 5),
                    (false, false) => Count::less_than(I::BITS + 1, 0, 839, 1039),
                }
            }
            (_, _) => match (I::is_signed(), 2 * I::BITS < E::BaseField::size_in_data_bits() as u64) {
                (true, true) => Count::is(4 * I::BITS, 0, (9 * I::BITS) + 5, (9 * I::BITS) + 11),
                (true, false) => Count::is(4 * I::BITS, 0, 1610, 1813),
                (false, true) => Count::is(I::BITS, 0, (3 * I::BITS) + 2, (3 * I::BITS) + 5),
                (false, false) => Count::is(I::BITS, 0, 839, 1039),
            },
        }
    }
}

impl<E: Environment, I: IntegerType> OutputMode<dyn RemWrapped<Integer<E, I>, Output = Integer<E, I>>>
    for Integer<E, I>
{
    type Case = (Mode, Mode);

    fn output_mode(case: &Self::Case) -> Mode {
        match (case.0, case.1) {
            (Mode::Constant, Mode::Constant) => Mode::Constant,
            (_, _) => Mode::Private,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_circuit_environment::Circuit;

    use test_utilities::*;

    use core::{ops::RangeInclusive, panic::RefUnwindSafe};

    const ITERATIONS: u64 = 32;

    fn check_rem<I: IntegerType + RefUnwindSafe>(
        name: &str,
        first: console::Integer<<Circuit as Environment>::Network, I>,
        second: console::Integer<<Circuit as Environment>::Network, I>,
        mode_a: Mode,
        mode_b: Mode,
    ) {
        let a = Integer::<Circuit, I>::new(mode_a, first);
        let b = Integer::<Circuit, I>::new(mode_b, second);
        if second == console::Integer::zero() {
            match mode_b {
                Mode::Constant => check_operation_halts(&a, &b, Integer::rem_wrapped),
                _ => Circuit::scope(name, || {
                    let _candidate = a.rem_wrapped(&b);
                    assert_count_fails!(RemWrapped(Integer<I>, Integer<I>) => Integer<I>, &(mode_a, mode_b));
                }),
            }
        } else {
            let expected = first.wrapping_rem(&second);
            Circuit::scope(name, || {
                let candidate = a.rem_wrapped(&b);
                assert_eq!(expected, *candidate.eject_value());
                assert_eq!(console::Integer::new(expected), candidate.eject_value());
                assert_count!(RemWrapped(Integer<I>, Integer<I>) => Integer<I>, &(mode_a, mode_b));
                assert_output_mode!(RemWrapped(Integer<I>, Integer<I>) => Integer<I>, &(mode_a, mode_b), candidate);
            })
        }
        Circuit::reset();
    }

    fn run_test<I: IntegerType + RefUnwindSafe>(mode_a: Mode, mode_b: Mode) {
        let mut rng = TestRng::default();

        for _ in 0..ITERATIONS {
            let first = Uniform::rand(&mut rng);
            let second = Uniform::rand(&mut rng);

            let name = format!("Rem: {first} % {second}");
            check_rem::<I>(&name, first, second, mode_a, mode_b);

            let name = format!("Rem by One: {first} % 1");
            check_rem::<I>(&name, first, console::Integer::one(), mode_a, mode_b);

            let name = format!("Rem by Self: {first} % {first}");
            check_rem::<I>(&name, first, first, mode_a, mode_b);

            let name = format!("Rem by Zero: {first} % 0");
            check_rem::<I>(&name, first, console::Integer::zero(), mode_a, mode_b);
        }

        // Check corner cases.
        check_rem::<I>("MAX % 1", console::Integer::MAX, console::Integer::one(), mode_a, mode_b);
        check_rem::<I>("MIN % 1", console::Integer::MIN, console::Integer::one(), mode_a, mode_b);
        check_rem::<I>("1 % 1", console::Integer::one(), console::Integer::one(), mode_a, mode_b);
        check_rem::<I>("0 % 1", console::Integer::zero(), console::Integer::one(), mode_a, mode_b);
        check_rem::<I>("MAX % 0", console::Integer::MAX, console::Integer::zero(), mode_a, mode_b);
        check_rem::<I>("MIN % 0", console::Integer::MIN, console::Integer::zero(), mode_a, mode_b);
        check_rem::<I>("1 % 0", console::Integer::one(), console::Integer::zero(), mode_a, mode_b);
        check_rem::<I>("0 % 0", console::Integer::zero(), console::Integer::zero(), mode_a, mode_b);

        // Check some additional corner cases for signed integers.
        if I::is_signed() {
            check_rem::<I>("MAX % -1", console::Integer::MAX, -console::Integer::one(), mode_a, mode_b);
            check_rem::<I>("MIN % -1", console::Integer::MIN, -console::Integer::one(), mode_a, mode_b);
            check_rem::<I>("1 % -1", console::Integer::one(), -console::Integer::one(), mode_a, mode_b);
        }
    }

    fn run_exhaustive_test<I: IntegerType + RefUnwindSafe>(mode_a: Mode, mode_b: Mode)
    where
        RangeInclusive<I>: Iterator<Item = I>,
    {
        for first in I::MIN..=I::MAX {
            for second in I::MIN..=I::MAX {
                let first = console::Integer::<_, I>::new(first);
                let second = console::Integer::<_, I>::new(second);

                let name = format!("Rem: ({first} % {second})");
                check_rem::<I>(&name, first, second, mode_a, mode_b);
            }
        }
    }

    test_integer_binary!(run_test, i8, rem);
    test_integer_binary!(run_test, i16, rem);
    test_integer_binary!(run_test, i32, rem);
    test_integer_binary!(run_test, i64, rem);
    test_integer_binary!(run_test, i128, rem);

    test_integer_binary!(run_test, u8, rem);
    test_integer_binary!(run_test, u16, rem);
    test_integer_binary!(run_test, u32, rem);
    test_integer_binary!(run_test, u64, rem);
    test_integer_binary!(run_test, u128, rem);

    test_integer_binary!(#[ignore], run_exhaustive_test, u8, rem, exhaustive);
    test_integer_binary!(#[ignore], run_exhaustive_test, i8, rem, exhaustive);
}
