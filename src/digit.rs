mod digit_str;

use crate::{AppError, TimerMinute};
use digit_str::*;

use std::sync::LazyLock;

fn figure_str_to_bitmap(s: &str) -> [[bool; 7]; 16] {
    let trimmed = s.trim_start().trim_end();
    let mut bitmap = [[false; 7]; 16];

    for (i, line) in trimmed.lines().enumerate() {
        for (j, c) in line.chars().enumerate() {
            let bit = match c {
                '■' => true,
                '□' => false,
                _ => panic!("Internal bug. Unknown char: {c}"),
            };

            bitmap[i][j] = bit;
        }
    }

    bitmap
}

static ZERO: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(ZERO_STR));
static ONE: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(ONE_STR));
static TWO: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(TWO_STR));
static THREE: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(THREE_STR));
static FOUR: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(FOUR_STR));
static FIVE: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(FIVE_STR));
static SIX: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(SIX_STR));
static SEVEN: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(SEVEN_STR));
static EIGHT: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(EIGHT_STR));
static NINE: LazyLock<[[bool; 7]; 16]> = LazyLock::new(|| figure_str_to_bitmap(NINE_STR));

enum Digit {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
}

impl Digit {
    pub fn bitmap(&self) -> &[[bool; 7]; 16] {
        use Digit::*;

        match self {
            Zero => &ZERO,
            One => &ONE,
            Two => &TWO,
            Three => &THREE,
            Four => &FOUR,
            Five => &FIVE,
            Six => &SIX,
            Seven => &SEVEN,
            Eight => &EIGHT,
            Nine => &NINE,
        }
    }
}

impl TryFrom<u8> for Digit {
    type Error = AppError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        use Digit::*;

        match value {
            0 => Ok(Zero),
            1 => Ok(One),
            2 => Ok(Two),
            3 => Ok(Three),
            4 => Ok(Four),
            5 => Ok(Five),
            6 => Ok(Six),
            7 => Ok(Seven),
            8 => Ok(Eight),
            9 => Ok(Nine),
            _ => Err(AppError::TypeError(
                "Digit must be single digit.".to_string(),
            )),
        }
    }
}

pub struct DoubleDigit(Digit, Digit);

impl DoubleDigit {
    fn new(tens: Digit, ones: Digit) -> DoubleDigit {
        DoubleDigit(tens, ones)
    }
    #[cfg(test)]
    pub fn bitmap(&self) -> [[bool; 16]; 16] {
        let mut out = [[false; 16]; 16];

        let tens_bitmap = self.0.bitmap();
        let ones_bitmap = self.1.bitmap();

        for i in 0..16 {
            out[i][0..7].copy_from_slice(&tens_bitmap[i]);
            out[i][9..16].copy_from_slice(&ones_bitmap[i]);
        }

        out
    }
    pub fn bitmap_flatten(&self) -> [bool; 256] {
        let mut out = [false; 256];

        let tens_bitmap = self.0.bitmap();
        let ones_bitmap = self.1.bitmap();

        for i in 0..16 {
            out[(16 * i)..(16 * i + 7)].copy_from_slice(&tens_bitmap[i]);
            out[(16 * i + 9)..(16 * i + 16)].copy_from_slice(&ones_bitmap[i]);
        }

        out
    }
}

impl From<TimerMinute> for DoubleDigit {
    fn from(value: TimerMinute) -> Self {
        let tens = Digit::try_from(value.tens()).unwrap();
        let ones = Digit::try_from(value.ones()).unwrap();

        DoubleDigit::new(tens, ones)
    }
}

#[cfg(test)]
mod test {
    use crate::TimerMinute;
    use crate::digit::DoubleDigit;

    #[test]
    fn test_double_digit() {
        for i in 0..100_u8 {
            let double_digit = DoubleDigit::from(TimerMinute::try_from(i).unwrap());

            let bitmap_and_flatten = double_digit
                .bitmap()
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
            let bitmap_flatten = double_digit
                .bitmap_flatten()
                .into_iter()
                .collect::<Vec<_>>();

            assert!(bitmap_and_flatten == bitmap_flatten);
        }
    }
}
