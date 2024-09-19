use std::string::ToString;

use crate::{Approximint, DecimalFormatter, ScientificFormatter, WordFormatter};

#[test]
#[expect(clippy::similar_names)]
fn basics() {
    let thousand = Approximint::new(1000);
    let million = thousand * thousand;
    let billion = thousand * million;
    assert_eq!(million * 0.001, thousand);
    assert_eq!(billion * 0.001, million);
    assert_eq!(billion * 0.000_001, thousand);
    assert_eq!(billion * 0.000_000_001, Approximint::ONE);
    assert_eq!(billion * 3.12, Approximint::approximate(3_120_000_000_u32));
    let negative_million = -thousand * thousand;
    let negative_billion = thousand * negative_million;
    assert_eq!(negative_million * 0.001, -thousand);
    assert_eq!(negative_billion * 0.001, negative_million);
    assert_eq!(negative_billion * 0.000_001, -thousand);
    assert_eq!(negative_billion * 0.000_000_001, -Approximint::ONE);
    assert_eq!(
        negative_billion * 3.12,
        -Approximint::approximate(3_120_000_000_u32)
    );

    assert_eq!(
        billion * 1_000.,
        Approximint::approximate(1_000_000_000_000u64)
    );
    assert_eq!(
        billion * thousand,
        Approximint::approximate(1_000_000_000_000u64)
    );
    assert_eq!(thousand - thousand, Approximint::ZERO);
    assert_eq!(Approximint::ZERO - thousand, -thousand);
}

#[test]
fn formatting() {
    assert_eq!(Approximint::new(123).to_string(), "123");
    assert_eq!(Approximint::new(1234).to_string(), "1,234");
    assert_eq!(Approximint::new(123_456_789).to_string(), "123,456,789");
    assert_eq!(
        DecimalFormatter::from(Approximint::new(1_234_567_890)).to_string(),
        "1,234,567,890"
    );
    assert_eq!(
        DecimalFormatter::from(Approximint::new(1_234_567_890))
            .separator('.')
            .digits_per_separator(4)
            .to_string(),
        "12.3456.7890"
    );
    assert_eq!(
        DecimalFormatter::from(Approximint::new(1_234_567_890))
            .digits_per_separator(0)
            .to_string(),
        "1234567890"
    );
    // Feature, not a bug: don't round by default in display formatting. In a
    // clicker game if your currency total displays the same as what the
    // purchase price is, it should always be able to purchase it. Rounding can
    // make the currency total larger than it actually is.
    assert_eq!(Approximint::new(1_234_567_890).to_string(), "1.234e9");
    // But we support rounding.
    assert_eq!(
        ScientificFormatter::from(Approximint::new(1_234_567_890))
            .rounded()
            .to_string(),
        "1.235e9"
    );
    assert_eq!(
        ScientificFormatter::from(Approximint::new(999_999)).to_string(),
        "9.999e5"
    );
    assert_eq!(
        ScientificFormatter::from(Approximint::new(999_999))
            .rounded()
            .to_string(),
        "1.000e6"
    );
    assert_eq!(
        ScientificFormatter::from(Approximint::new(999_999))
            .rounded()
            .truncate_zeroes()
            .to_string(),
        "1e6"
    );
    assert_eq!(
        ScientificFormatter::from(Approximint::new(1_045_999))
            .rounded()
            .significant_digits(2)
            .to_string(),
        "1.0e6"
    );
    assert_eq!(
        ScientificFormatter::from(Approximint::new(1_045_999))
            .rounded()
            .significant_digits(3)
            .to_string(),
        "1.05e6"
    );
    assert_eq!(
        ScientificFormatter::from(Approximint::new(1_105_999))
            .significant_digits(3)
            .truncate_zeroes()
            .to_string(),
        "1.1e6"
    );
}

#[test]
fn english() {
    assert_eq!(
        WordFormatter::english(Approximint::new(123_000))
            .decimal_before_10_power(0)
            .to_string(),
        "123 thousand"
    );
    assert_eq!(
        WordFormatter::english(Approximint::new(123_100))
            .decimal_before_10_power(0)
            .to_string(),
        "123.1 thousand"
    );
    assert_eq!(
        WordFormatter::english(Approximint::new(123_100))
            .decimal_before_10_power(9)
            .to_string(),
        "123,100"
    );
    assert_eq!(
        WordFormatter::english(Approximint::new(123_456_789))
            .decimal_before_10_power(6)
            .to_string(),
        "123.4 million"
    );
    assert_eq!(
        WordFormatter::english(Approximint::one_e(100) * core::f64::consts::PI).to_string(),
        "3.1 googol"
    );
    assert_eq!(
        WordFormatter::english(Approximint::one_e(100) * Approximint::new(1_000)).to_string(),
        "1,000 googol"
    );
    assert_eq!(
        WordFormatter::english(Approximint::one_e(100) * Approximint::new(999_999_999)).to_string(),
        "999,999,999 googol"
    );
    assert_eq!(
        WordFormatter::english(Approximint::one_e(100) * Approximint::new(1_000_000_000))
            .to_string(),
        "1 billion googol"
    );
    assert_eq!(
        WordFormatter::english(Approximint::one_e(100) * Approximint::one_e(100)).to_string(),
        "1 googol googol"
    );
}

#[test]
fn float_conversion() {
    assert_eq!(Approximint::approximate(123.), Approximint::new(123));
    assert_eq!(
        Approximint::approximate(1_234_567_890.),
        Approximint::new(1_234_567_890)
    );
    assert_eq!(
        Approximint::approximate(1_234_567_890.),
        Approximint::new(1_234_567_890)
    );
    assert_eq!(Approximint::approximate(1.0e100), Approximint::one_e(100));
}
