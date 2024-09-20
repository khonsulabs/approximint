#![doc = include_str!(".crate-docs.md")]
#![no_std]
use core::fmt::{Debug, Display, Write};
use core::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};
use core::slice;

#[cfg(any(feature = "std", test))]
extern crate std;

/// An integer type that approximates its value using storage inspired by
/// scientific notation.
///
/// This type supports representing integers in a form of `coefficient *
/// 10^exponent`. The coefficient has a range of `-999_999_999..=999_999_999`,
/// and the maximum exponent is `u32::MAX`. This approach supports a range of
/// `-9.999_999_99e4_294_967_303..=9.999_999_99e4_294_967_303` while retaining 9
/// digits of precision.
#[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Default)]
pub struct Approximint {
    ten_power: u32,
    coefficient: i32,
}

impl Approximint {
    const COEFFICIENT_LIMIT: i32 = 1_000_000_000;
    pub const MAX: Self = Self {
        ten_power: u32::MAX,
        coefficient: 999_999_999,
    };
    pub const MIN: Self = Self {
        ten_power: u32::MAX,
        coefficient: -999_999_999,
    };
    /// A value representing 1.
    pub const ONE: Self = Self {
        ten_power: 0,
        coefficient: 1,
    };
    /// A value representing 0.
    pub const ZERO: Self = Self {
        ten_power: 0,
        coefficient: 0,
    };

    /// Returns `value` as an approximint.
    #[must_use]
    #[inline]
    pub const fn new(value: i32) -> Self {
        Self {
            coefficient: value,
            ten_power: 0,
        }
        .normalize_overflow()
    }

    /// Returns an approximation of `value`.
    #[inline]
    pub fn approximate(value: impl Approximate) -> Self {
        value.approximate()
    }

    /// Returns a value representing 10 raised to the power of `exponent`.
    #[must_use]
    #[inline]
    pub const fn one_e(exponent: u32) -> Self {
        Self {
            coefficient: 1,
            ten_power: exponent,
        }
        .normalize_underflow()
    }

    const fn normalized(self) -> Self {
        self.normalize_underflow().normalize_overflow()
    }

    const fn normalize_underflow(mut self) -> Self {
        if self.coefficient > 0 {
            while self.ten_power > 0 && self.coefficient < 100_000_000 {
                self.coefficient *= 10;
                self.ten_power -= 1;
            }
        } else if self.coefficient < 0 {
            while self.ten_power > 0 && self.coefficient > -100_000_000 {
                self.coefficient *= 10;
                self.ten_power -= 1;
            }
        }

        self
    }

    const fn normalize_overflow(mut self) -> Self {
        if self.coefficient >= Self::COEFFICIENT_LIMIT
            || self.coefficient <= -Self::COEFFICIENT_LIMIT
        {
            if let Some(next_power) = self.ten_power.checked_add(1) {
                self.ten_power = next_power;
                self.coefficient /= 10;
            } else {
                self.coefficient = self.coefficient.signum() * 999_999_999;
            }
        }
        self
    }

    const fn match_powers(left: Self, right: Self) -> (Self, Self) {
        let left = left.normalized();
        let right = right.normalized();

        if left.ten_power < right.ten_power {
            Self::adjusted_powers(left, right)
        } else {
            let (right, left) = Self::adjusted_powers(right, left);
            (left, right)
        }
    }

    const fn adjusted_powers(mut lower: Self, higher: Self) -> (Self, Self) {
        while lower.ten_power < higher.ten_power {
            lower.coefficient /= 10;
            if lower.coefficient == 0 {
                lower.ten_power = higher.ten_power;
                break;
            }
            lower.ten_power += 1;
        }
        (lower, higher)
    }

    /// Returns a [`Display`] implementor that formats this number using English
    /// words.
    pub fn as_english(self) -> WordFormatter<'static> {
        WordFormatter::english(self)
    }

    /// Returns a [`Display`] implementor that formats this number using
    /// scientific notation.
    pub fn as_scientific(self) -> ScientificFormatter {
        ScientificFormatter::from(self)
    }

    /// Returns a [`Display`] implementor that formats this number using decimal
    /// notation.
    pub fn as_decimal(self) -> DecimalFormatter {
        DecimalFormatter::from(self)
    }
}

impl Neg for Approximint {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self {
            ten_power: self.ten_power,
            coefficient: -self.coefficient,
        }
    }
}

impl Add for Approximint {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = Self::match_powers(self, rhs);
        Self {
            // Adding two numbers less than 1 billion will never overflow u32
            coefficient: lhs.coefficient + rhs.coefficient,
            ten_power: lhs.ten_power,
        }
        .normalized()
    }
}

impl AddAssign for Approximint {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Approximint {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let (lhs, rhs) = Self::match_powers(self, rhs);
        Self {
            coefficient: lhs.coefficient - rhs.coefficient,
            ten_power: lhs.ten_power,
        }
        .normalized()
    }
}

impl SubAssign for Approximint {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for Approximint {
    type Output = Self;

    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    fn mul(self, rhs: Self) -> Self::Output {
        let mut coefficient = i64::from(self.coefficient) * i64::from(rhs.coefficient);
        let mut ten_power = self.ten_power + rhs.ten_power;
        while coefficient >= i64::from(Self::COEFFICIENT_LIMIT) {
            if let Some(next_power) = ten_power.checked_add(1) {
                ten_power = next_power;
                coefficient /= 10;
            } else {
                coefficient = 999_999_999;
                break;
            }
        }
        while coefficient <= i64::from(-Self::COEFFICIENT_LIMIT) {
            if let Some(next_power) = ten_power.checked_add(1) {
                ten_power = next_power;
                coefficient /= 10;
            } else {
                coefficient = -999_999_999;
                break;
            }
        }
        Self {
            coefficient: coefficient as i32,
            ten_power,
        }
    }
}

#[cfg(feature = "std")]
impl Mul<f64> for Approximint {
    type Output = Self;

    #[inline]
    #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn mul(self, rhs: f64) -> Self::Output {
        if rhs >= f64::from(Self::COEFFICIENT_LIMIT) {
            self * Self::approximate(rhs)
        } else {
            let coefficient = f64::from(self.coefficient) * rhs;
            let decimals = coefficient.abs().log10();
            let mut places_to_shift = (9.0 - decimals).floor() as i32;
            let ten_power =
                if let Some(ten_power) = self.ten_power.checked_add_signed(-places_to_shift) {
                    ten_power
                } else {
                    places_to_shift = self.ten_power as i32;
                    0
                };

            let shifted = coefficient * 10f64.powi(places_to_shift);
            Self {
                coefficient: shifted.round() as i32,
                ten_power,
            }
            .normalize_overflow()
        }
    }
}

impl From<u8> for Approximint {
    #[inline]
    fn from(value: u8) -> Self {
        Self::new(i32::from(value))
    }
}

impl From<u16> for Approximint {
    #[inline]
    fn from(value: u16) -> Self {
        Self::new(i32::from(value))
    }
}

impl From<i32> for Approximint {
    #[inline]
    fn from(value: i32) -> Self {
        Self::new(value)
    }
}

impl Display for Approximint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.ten_power > 0 {
            Display::fmt(&ScientificFormatter::from(*self), f)
        } else {
            Display::fmt(&DecimalFormatter::from(*self), f)
        }
    }
}

impl Debug for Approximint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.ten_power > 0 {
            // For Debug, always show all digits of precision.
            Display::fmt(
                &ScientificFormatter::from(*self)
                    .significant_digits(9)
                    .truncate_zeroes(),
                f,
            )
        } else {
            Display::fmt(&DecimalFormatter::from(*self), f)
        }
    }
}

/// A [`Display`] implementation that formats an [`Approximint`] using
/// scientific notation.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[must_use]
pub struct ScientificFormatter {
    num: Approximint,
    round: bool,
    settings: ScientificSettings,
}

impl ScientificFormatter {
    /// Sets the character to use between the whole number and decimal digits.
    ///
    /// By default, the decimal character is `.`.
    #[inline]
    pub fn decimal(mut self, decimal: char) -> Self {
        self.settings.decimal = decimal;
        self
    }

    /// Performs rounding on the displayed value.
    #[inline]
    pub fn rounded(mut self) -> Self {
        self.round = true;
        self
    }

    /// Sets the number of significant digits to display.
    #[inline]
    pub fn significant_digits(mut self, digits: u8) -> Self {
        if self.round {
            assert!(
                digits <= 8,
                "significant digits must be less than 9 when rounding"
            );
        } else {
            assert!(
                digits <= 9,
                "significant digits must be less than or equal to 9"
            );
        }
        self.settings.significant_digits = digits;
        self
    }

    /// Prevents displaying trailing zeroes.
    #[inline]
    pub fn truncate_zeroes(mut self) -> Self {
        self.settings.keep_trailing_zeroes = false;
        self
    }
}

impl From<Approximint> for ScientificFormatter {
    #[inline]
    fn from(num: Approximint) -> Self {
        Self {
            num,
            round: false,
            settings: ScientificSettings::default(),
        }
    }
}

impl Display for ScientificFormatter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.num.ten_power == 0 && self.num.coefficient == 0 {
            return f.write_str("0");
        }

        let mut info = ScientificInfo::new(self.num);
        if self.round {
            info.round(self.settings.significant_digits);
        }
        info.fmt(f, self.settings)
    }
}

#[derive(Debug, Copy, Clone)]
struct ScientificInfo {
    digits: DigitRing,
    exponent: u64,
    negative: bool,
}

impl ScientificInfo {
    #[expect(clippy::cast_sign_loss)]
    fn new(num: Approximint) -> Self {
        let mut digits = DigitRing::default();

        let (negative, mut coefficient) = if num.coefficient >= 0 {
            (false, num.coefficient as u32)
        } else {
            (true, num.coefficient.unsigned_abs())
        };
        let mut exponent = 0;
        while coefficient > 0 {
            digits.push_back((coefficient % 10) as u8 + b'0');
            coefficient /= 10;
            exponent += 1;
        }

        let exponent = exponent - 1 + u64::from(num.ten_power);
        Self {
            digits,
            exponent,
            negative,
        }
    }

    fn round(&mut self, significant_digits: u8) {
        if significant_digits <= 8 {
            let mut digits_to_round = self
                .digits
                .iter_mut_rev()
                .skip(8 - usize::from(significant_digits));
            let check_digit = digits_to_round.next().expect("not 0");
            if (b'5'..=b'9').contains(check_digit) {
                let mut carry = false;
                for digit in digits_to_round {
                    if *digit == b'9' {
                        *digit = b'0';
                        carry = true;
                    } else {
                        *digit += 1;
                        carry = false;
                        break;
                    }
                }

                // If we still have the carry flag, we need to push a new 1
                // digit.
                if carry {
                    self.digits.push_back(b'1');
                    self.exponent += 1;
                }
            }
        }
    }

    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
        settings: ScientificSettings,
    ) -> core::fmt::Result {
        if self.negative {
            f.write_char('-')?;
        }
        let mut digits = self
            .digits
            .iter()
            .take(usize::from(settings.significant_digits))
            .enumerate();
        while let Some((index, digit)) = digits.next() {
            if !settings.keep_trailing_zeroes
                && index > 0
                && digit == b'0'
                && digits.clone().all(|(_, digit)| digit == b'0')
            {
                break;
            }

            if index == 1 {
                f.write_char(settings.decimal)?;
            }

            f.write_char(char::from(digit))?;
        }

        write!(f, "e{}", self.exponent)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct ScientificSettings {
    decimal: char,
    significant_digits: u8,
    keep_trailing_zeroes: bool,
}

impl Default for ScientificSettings {
    #[inline]
    fn default() -> Self {
        Self {
            decimal: '.',
            significant_digits: 4,
            keep_trailing_zeroes: true,
        }
    }
}

#[derive(Default, Debug, Copy, Clone)]
struct DigitRing {
    digits: [u8; 9],
    first: u8,
}

impl DigitRing {
    fn push_back(&mut self, digit: u8) {
        self.digits[usize::from(self.first)] = digit;
        self.first += 1;
        if usize::from(self.first) == self.digits.len() {
            self.first = 0;
        }
    }

    fn iter(&self) -> DigitRingIter<'_> {
        self.into_iter()
    }

    fn iter_mut_rev(&mut self) -> DigitRingIterMutRev<'_> {
        let (first, second) = self.digits.split_at_mut(usize::from(self.first));
        DigitRingIterMutRev(second.iter_mut(), first.iter_mut())
    }

    const fn len(&self) -> usize {
        self.digits.len()
    }
}

impl<'a> IntoIterator for &'a DigitRing {
    type IntoIter = DigitRingIter<'a>;
    type Item = u8;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let (first, second) = self.digits.split_at(usize::from(self.first));
        DigitRingIter(second.iter(), first.iter())
    }
}

#[derive(Clone)]
struct DigitRingIter<'a>(slice::Iter<'a, u8>, slice::Iter<'a, u8>);

impl<'a> Iterator for DigitRingIter<'a> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let digit = self.1.next_back().or_else(|| self.0.next_back())?;
            if *digit > 0 {
                return Some(*digit);
            }
        }
    }
}

struct DigitRingIterMutRev<'a>(slice::IterMut<'a, u8>, slice::IterMut<'a, u8>);

impl<'a> Iterator for DigitRingIterMutRev<'a> {
    type Item = &'a mut u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().or_else(|| self.1.next())
    }
}

/// A [`Display`] implementation for an [`Approximint`] that uses a word list.
#[derive(Clone, Debug)]
#[must_use]
pub struct WordFormatter<'a> {
    decimal: DecimalFormatter,
    decimal_before: u32,
    words: &'a [(u32, &'a str)],
    round: bool,
}

static ENGLISH: [(u32, &str); 33] = [
    (3, "thousand"),
    (6, "million"),
    (9, "billion"),
    (12, "trillion"),
    (15, "quadrillion"),
    (18, "quintillion"),
    (21, "sextillion"),
    (24, "septillion"),
    (27, "octillion"),
    (30, "nonillion"),
    (33, "decillion"),
    (36, "undecillion"),
    (39, "duodecillion"),
    (42, "tredecillion"),
    (45, "quattuordecillion"),
    (48, "quindecillion"),
    (51, "sexdecillion"),
    (54, "septendecillion"),
    (57, "octodecillion"),
    (60, "novemdecillion"),
    (63, "vigintillion"),
    (66, "unvigintillion"),
    (69, "duovigintillion"),
    (72, "trevigintillion"),
    (75, "quattuorvigintillion"),
    (78, "quinvigintillion"),
    (81, "sexvigintillion"),
    (84, "septenvigintillion"),
    (87, "octovigintillion"),
    (90, "novemvigintillion"),
    (93, "trigintillion"),
    (100, "googol"),
    (303, "centillion"),
];

impl WordFormatter<'static> {
    /// Returns a formatter for the English language.
    #[inline]
    pub fn english(num: Approximint) -> Self {
        Self::new(num, &ENGLISH).decimal_before_10_power(9)
    }
}

impl<'a> WordFormatter<'a> {
    /// Returns a new formatter for `num` using the given `words`.
    ///
    /// `words` is a slice of pairs of powers of ten and the associated word.
    /// For example, here is a portion of the English word list:
    ///
    /// ```rust
    /// &[
    ///     (3, "thousand"),
    ///     (6, "million"),
    ///     (9, "billion"),
    ///     (12, "trillion"),
    ///     // ...
    /// ];
    /// ```
    ///
    /// The formatter will reduce `num`'s ten-power by the largest matching
    /// word, and repeat the process until the value is too small for any
    /// eligible words. The remaining value will then be formatted using decimal
    /// notation with a single decimal digit when the value is less than 1,000.
    #[inline]
    pub fn new(num: Approximint, words: &'static [(u32, &'static str)]) -> Self {
        Self {
            decimal: DecimalFormatter::from(num),
            decimal_before: 0,
            words,
            round: false,
        }
    }

    /// Performs rounding before formatting the number.
    #[inline]
    pub fn rounded(mut self) -> Self {
        self.round = true;
        self
    }

    /// Prevents using words for powers of ten less than or equal to
    /// `ten_power`.
    ///
    /// The default English formatter sets this to 9, preventing values less
    /// than 1 billion from being converted to words.
    #[inline]
    pub fn decimal_before_10_power(mut self, ten_power: u32) -> Self {
        self.decimal_before = ten_power;
        self
    }

    /// Sets the character to use between grouped integer digits.
    ///
    /// The default separator is `,`.
    #[inline]
    pub fn separator(mut self, separator: char) -> Self {
        self.decimal.separator = separator;
        self
    }

    /// Sets the number of integer digits between each separator character.
    ///
    /// The default is 3.
    #[inline]
    pub fn digits_per_separator(mut self, digits: u8) -> Self {
        self.decimal.digits_per_separator = digits;
        self
    }

    fn format_info(
        &self,
        info: ScientificInfo,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        if info.negative {
            f.write_char('-')?;
        }
        self.format_words(info.exponent, f, |f, exponent| {
            let digits_per_separator = usize::from(self.decimal.digits_per_separator);
            let exponent_usize = usize::try_from(exponent).expect("exponent too large for usize");
            let separator_offset = digits_per_separator - 1 - exponent_usize % digits_per_separator;
            for (index, digit) in info.digits.iter().take(exponent_usize + 2).enumerate() {
                if index == exponent_usize + 1 {
                    if digit == b'0' {
                        break;
                    }
                    f.write_char('.')?;
                } else if index > 0 && (index + separator_offset) % digits_per_separator == 0 {
                    f.write_char(self.decimal.separator)?;
                }
                f.write_char(char::from(digit))?;
            }
            Ok(())
        })
    }

    fn format_words(
        &self,
        exponent: u64,
        f: &mut core::fmt::Formatter<'_>,
        format_exponent: impl FnOnce(&mut core::fmt::Formatter<'_>, u64) -> core::fmt::Result,
    ) -> core::fmt::Result {
        // info treats the leading digit as significant, but for the purpose of
        // this function we need to treat exponent as a count of digits.
        let word = self
            .words
            .windows(2)
            .skip_while(|words| words[0].0 < self.decimal_before)
            .find(|words| u64::from(words[0].0) <= exponent && u64::from(words[1].0) > exponent)
            .map_or_else(
                || self.words.last().expect("at least one word"),
                |words| &words[0],
            );
        let Some(exponent) = exponent.checked_sub(u64::from(word.0)) else {
            return format_exponent(f, exponent);
        };

        if self.round {
            todo!("round");
        }

        if exponent < u64::from(self.decimal_before) {
            format_exponent(f, exponent)?;
        } else {
            self.format_words(exponent, f, format_exponent)?;
        }

        f.write_char(' ')?;
        f.write_str(word.1)
    }
}

impl Display for WordFormatter<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.format_info(ScientificInfo::new(self.decimal.num), f)
    }
}

/// A [`Display`] implementor for an [`Approximint`] that formats using decimal
/// notation.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[must_use]
pub struct DecimalFormatter {
    num: Approximint,
    separator: char,
    digits_per_separator: u8,
    scientific_after: u32,
}

impl DecimalFormatter {
    /// Sets the character to use between grouped integer digits.
    ///
    /// The default separator is `,`.
    #[inline]
    pub fn separator(mut self, separator: char) -> Self {
        self.separator = separator;
        self
    }

    /// Sets the number of integer digits between each separator character.
    ///
    /// The default is 3.
    #[inline]
    pub fn digits_per_separator(mut self, digits: u8) -> Self {
        self.digits_per_separator = digits;
        self
    }
}

impl From<Approximint> for DecimalFormatter {
    #[inline]
    fn from(num: Approximint) -> Self {
        Self {
            num,
            separator: ',',
            digits_per_separator: 3,
            scientific_after: 30,
        }
    }
}

impl Display for DecimalFormatter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.num.ten_power == 0 && self.num.coefficient == 0 {
            return f.write_str("0");
        } else if self.num.ten_power >= self.scientific_after {
            return Display::fmt(&ScientificFormatter::from(self.num), f);
        }

        // To avoid allocations, we need to figure out how many total digits we
        // have so that we can emit separators along the way.
        let info = ScientificInfo::new(self.num);

        if info.negative {
            f.write_char('-')?;
        }

        let digits_per_separator = usize::from(self.digits_per_separator);
        let exponent_usize = usize::try_from(info.exponent).expect("exponent too large for usize");
        let separator_offset = if digits_per_separator > 0 {
            digits_per_separator - 1 - exponent_usize % digits_per_separator
        } else {
            0
        };

        let mut index = 0;
        for digit in info
            .digits
            .iter()
            .take(info.digits.len().min(exponent_usize + 1))
        {
            if self.digits_per_separator > 0
                && index > 0
                && (index + separator_offset) % digits_per_separator == 0
            {
                f.write_char(self.separator)?;
            }
            f.write_char(char::from(digit))?;
            index += 1;
        }

        for index in index..=exponent_usize {
            if self.digits_per_separator > 0
                && (index + separator_offset) % digits_per_separator == 0
            {
                f.write_char(self.separator)?;
            }
            f.write_char('0')?;
        }
        Ok(())
    }
}

/// A value that can be approximated into an [`Approximint`].
pub trait Approximate {
    /// Returns this value as an integer approximation.
    fn approximate(self) -> Approximint;
}

impl Approximate for u32 {
    #[inline]
    #[expect(clippy::cast_possible_wrap)]
    fn approximate(mut self) -> Approximint {
        let mut ten_power = 0;
        while self >= Approximint::COEFFICIENT_LIMIT as u32 {
            ten_power += 1;
            self /= 10;
        }

        Approximint {
            coefficient: self as i32,
            ten_power,
        }
    }
}

impl Approximate for u64 {
    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    fn approximate(mut self) -> Approximint {
        let mut ten_power = 0;
        while self >= Approximint::COEFFICIENT_LIMIT as u64 {
            ten_power += 1;
            self /= 10;
        }

        Approximint {
            coefficient: self as i32,
            ten_power,
        }
    }
}

impl Approximate for usize {
    #[inline]
    #[cfg(any(target_pointer_width = "16", target_pointer_width = "32"))]
    fn approximate(self) -> Approximint {
        Approximint::from(self as u32)
    }

    #[inline]
    #[cfg(not(any(target_pointer_width = "16", target_pointer_width = "32")))]
    #[expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    fn approximate(mut self) -> Approximint {
        let mut ten_power = 0;
        while self >= Approximint::COEFFICIENT_LIMIT as usize {
            ten_power += 1;
            self /= 10;
        }

        Approximint {
            coefficient: self as i32,
            ten_power,
        }
    }
}

impl Approximate for u128 {
    #[inline]
    #[expect(clippy::cast_possible_truncation)]
    fn approximate(mut self) -> Approximint {
        let mut ten_power = 0;
        while self >= Approximint::COEFFICIENT_LIMIT as u128 {
            ten_power += 1;
            self /= 10;
        }

        Approximint {
            coefficient: self as i32,
            ten_power,
        }
    }
}

#[cfg(feature = "std")]
impl Approximate for f64 {
    #[inline]
    #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn approximate(self) -> Approximint {
        let coefficient = self;
        let decimals = coefficient.log10();
        let mut places_to_shift = (9.0 - decimals).floor() as i32;
        let ten_power = if places_to_shift < 0 {
            (-places_to_shift) as u32
        } else {
            places_to_shift = 0;
            0
        };

        let shifted = coefficient * 10f64.powi(places_to_shift);
        Approximint {
            coefficient: shifted.round() as i32,
            ten_power,
        }
        .normalize_overflow()
    }
}

#[cfg(feature = "std")]
impl Approximate for f32 {
    #[inline]
    fn approximate(self) -> Approximint {
        f64::from(self).approximate()
    }
}

#[cfg(test)]
mod tests;
