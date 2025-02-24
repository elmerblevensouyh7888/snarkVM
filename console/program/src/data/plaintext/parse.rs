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

impl<N: Network> Parser for Plaintext<N> {
    /// Parses a string into a plaintext value.
    #[inline]
    fn parse(string: &str) -> ParserResult<Self> {
        /// Parses a sanitized pair: `identifier: plaintext`.
        fn parse_pair<N: Network>(string: &str) -> ParserResult<(Identifier<N>, Plaintext<N>)> {
            // Parse the whitespace and comments from the string.
            let (string, _) = Sanitizer::parse(string)?;
            // Parse the identifier from the string.
            let (string, identifier) = Identifier::parse(string)?;
            // Parse the whitespace from the string.
            let (string, _) = Sanitizer::parse_whitespaces(string)?;
            // Parse the ":" from the string.
            let (string, _) = tag(":")(string)?;
            // Parse the plaintext from the string.
            let (string, plaintext) = Plaintext::parse(string)?;
            // Return the identifier and plaintext.
            Ok((string, (identifier, plaintext)))
        }

        /// Parses a plaintext as a struct: `{ identifier_0: plaintext_0, ..., identifier_n: plaintext_n }`.
        fn parse_struct<N: Network>(string: &str) -> ParserResult<Plaintext<N>> {
            // Parse the whitespace and comments from the string.
            let (string, _) = Sanitizer::parse(string)?;
            // Parse the "{" from the string.
            let (string, _) = tag("{")(string)?;
            // Parse the members.
            let (string, members) = map_res(separated_list1(tag(","), parse_pair), |members: Vec<_>| {
                // Ensure the members has no duplicate names.
                if has_duplicates(members.iter().map(|(name, ..)| name)) {
                    return Err(error("Duplicate member in struct"));
                }
                // Ensure the number of structs is within the maximum limit.
                match members.len() <= N::MAX_STRUCT_ENTRIES {
                    true => Ok(members),
                    false => Err(error(format!("Found a plaintext that exceeds size ({})", members.len()))),
                }
            })(string)?;
            // Parse the whitespace and comments from the string.
            let (string, _) = Sanitizer::parse(string)?;
            // Parse the '}' from the string.
            let (string, _) = tag("}")(string)?;
            // Output the plaintext.
            Ok((string, Plaintext::Struct(IndexMap::from_iter(members.into_iter()), Default::default())))
        }

        // Parse the whitespace from the string.
        let (string, _) = Sanitizer::parse_whitespaces(string)?;
        // Parse to determine the plaintext (order matters).
        alt((
            // Parse a plaintext literal.
            map(Literal::parse, |literal| Self::Literal(literal, Default::default())),
            // Parse a plaintext struct.
            parse_struct,
        ))(string)
    }
}

impl<N: Network> FromStr for Plaintext<N> {
    type Err = Error;

    /// Returns a plaintext from a string literal.
    fn from_str(string: &str) -> Result<Self> {
        match Self::parse(string) {
            Ok((remainder, object)) => {
                // Ensure the remainder is empty.
                ensure!(remainder.is_empty(), "Failed to parse string. Found invalid character in: \"{remainder}\"");
                // Return the object.
                Ok(object)
            }
            Err(error) => bail!("Failed to parse string. {error}"),
        }
    }
}

impl<N: Network> Debug for Plaintext<N> {
    /// Prints the plaintext as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl<N: Network> Display for Plaintext<N> {
    /// Prints the plaintext as a string.
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.fmt_internal(f, 0)
    }
}

impl<N: Network> Plaintext<N> {
    /// Prints the plaintext with the given indentation depth.
    fn fmt_internal(&self, f: &mut Formatter, depth: usize) -> fmt::Result {
        /// The number of spaces to indent.
        const INDENT: usize = 2;

        match self {
            // Prints the literal, i.e. 10field
            Self::Literal(literal, ..) => write!(f, "{:indent$}{literal}", "", indent = depth * INDENT),
            // Prints the struct, i.e. { first: 10i64, second: 198u64 }
            Self::Struct(struct_, ..) => {
                // Print the opening brace.
                write!(f, "{{")?;
                // Print the members.
                struct_.iter().enumerate().try_for_each(|(i, (name, plaintext))| {
                    match plaintext {
                        Self::Literal(literal, ..) => match i == struct_.len() - 1 {
                            true => {
                                // Print the last member without a comma.
                                write!(f, "\n{:indent$}{name}: {literal}", "", indent = (depth + 1) * INDENT)?;
                                // Print the closing brace.
                                write!(f, "\n{:indent$}}}", "", indent = depth * INDENT)
                            }
                            // Print the member with a comma.
                            false => write!(f, "\n{:indent$}{name}: {literal},", "", indent = (depth + 1) * INDENT),
                        },
                        Self::Struct(..) => {
                            // Print the member name.
                            write!(f, "\n{:indent$}{name}: ", "", indent = (depth + 1) * INDENT)?;
                            // Print the member.
                            plaintext.fmt_internal(f, depth + 1)?;
                            // Print the closing brace.
                            match i == struct_.len() - 1 {
                                // Print the last member without a comma.
                                true => write!(f, "\n{:indent$}}}", "", indent = depth * INDENT),
                                // Print the member with a comma.
                                false => write!(f, ","),
                            }
                        }
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm_console_network::Testnet3;

    type CurrentNetwork = Testnet3;

    #[test]
    fn test_parse_literal() -> Result<()> {
        // Sanity check.
        let (remainder, candidate) = Plaintext::<CurrentNetwork>::parse("5u8")?;
        assert_eq!("5u8", candidate.to_string());
        assert_eq!("", remainder);

        Ok(())
    }

    #[test]
    fn test_parse_struct() -> Result<()> {
        // Sanity check.
        let expected = r"{
  foo: 5u8
}";
        let (remainder, candidate) = Plaintext::<CurrentNetwork>::parse("{ foo: 5u8 }")?;
        assert_eq!(expected, candidate.to_string());
        assert_eq!("", remainder);

        let expected = r"{
  foo: 5u8,
  bar: {
    baz: 10field,
    qux: {
      quux: {
        corge: {
          grault: {
            garply: {
              waldo: {
                fred: {
                  plugh: {
                    xyzzy: {
                      thud: true
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}";
        let (remainder, candidate) = Plaintext::<CurrentNetwork>::parse(
            "{ foo: 5u8, bar: { baz: 10field, qux: {quux:{corge :{grault:  {garply:{waldo:{fred:{plugh:{xyzzy: { thud: true}} }}}  }}}}}}",
        )?;
        println!("\nExpected: {expected}\n\nFound: {candidate}\n");
        assert_eq!(expected, candidate.to_string());
        assert_eq!("", remainder);

        Ok(())
    }

    #[test]
    fn test_parse_fails() {
        // Must be non-empty.
        assert!(Plaintext::<CurrentNetwork>::parse("").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("{}").is_err());

        // Invalid characters.
        assert!(Plaintext::<CurrentNetwork>::parse("_").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("__").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("___").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("-").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("--").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("---").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("*").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("**").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("***").is_err());

        // Must not start with a number.
        assert!(Plaintext::<CurrentNetwork>::parse("1").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("2").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("3").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("1foo").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("12").is_err());
        assert!(Plaintext::<CurrentNetwork>::parse("111").is_err());

        // Must fit within the data capacity of a base field element.
        let plaintext =
            Plaintext::<CurrentNetwork>::parse("foo_bar_baz_qux_quux_quuz_corge_grault_garply_waldo_fred_plugh_xyzzy");
        assert!(plaintext.is_err());
    }

    #[test]
    fn test_nested_structs1() {
        let expected = r"{
  r1: {
    c1: 1u8,
    c2: 2u8,
    c3: 1u8
  },
  r2: {
    c1: 2u8,
    c2: 2u8,
    c3: 1u8
  },
  r3: {
    c1: 1u8,
    c2: 2u8,
    c3: 1u8
  }
}";

        let (remainder, candidate) = Plaintext::<CurrentNetwork>::parse(expected).unwrap();
        println!("\nExpected: {expected}\n\nFound: {candidate}\n");
        assert_eq!(expected, candidate.to_string());
        assert_eq!("", remainder);
    }

    #[test]
    fn test_nested_structs2() {
        let expected = r"{
  foo: {
    bar: {
      baz: 1u8
    },
    qux: {
      quux: 2u8
    }
  }
}";

        let (remainder, candidate) = Plaintext::<CurrentNetwork>::parse(expected).unwrap();
        println!("\nExpected: {expected}\n\nFound: {candidate}\n");
        assert_eq!(expected, candidate.to_string());
        assert_eq!("", remainder);
    }
}
