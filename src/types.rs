use crate::AppError;

use serde::{Deserialize, Serialize};
use zbus::zvariant::{Signature, Type};

use std::path::{Path, PathBuf};

/// タイマー用の分表現
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type, PartialEq, Eq)]
pub struct TimerMinute(u8);

impl TimerMinute {
    pub fn as_u8(&self) -> u8 {
        self.0
    }
    pub fn as_secs(&self) -> u32 {
        self.0 as u32 * 60
    }
    pub fn tens(&self) -> u8 {
        self.0 / 10
    }
    pub fn ones(&self) -> u8 {
        self.0 % 10
    }
    pub fn decrement(&mut self) {
        self.0 = self.0.saturating_sub(1);
    }
}

impl TryFrom<u8> for TimerMinute {
    type Error = AppError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 100 {
            Ok(TimerMinute(value))
        } else {
            Err(AppError::TypeError(
                "TimerMinute must be up to 99".to_owned(),
            ))
        }
    }
}

impl TryFrom<String> for TimerMinute {
    type Error = AppError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value_u8 = value
            .parse::<u8>()
            .map_err(|_| AppError::TypeError(format!("Couldn't parse as u8: {value}")))?;
        value_u8.try_into()
    }
}

impl From<TimerMinute> for String {
    fn from(value: TimerMinute) -> Self {
        value.0.to_string()
    }
}

/// RGB表現
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Type, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

// impl Rgb {
//     pub fn with_alpha(&self, alpha: u8) -> Argb {
//         Argb {
//             a: alpha,
//             r: self.r,
//             g: self.g,
//             b: self.b,
//         }
//     }
// }

impl TryFrom<String> for Rgb {
    type Error = AppError;
    fn try_from(hex: String) -> Result<Self, Self::Error> {
        let stripped_hex = hex.trim_start_matches('#');
        if stripped_hex.len() != 6 {
            return Err(AppError::TypeError(format!("Invalid color hex: {hex}")));
        }

        let red = u8::from_str_radix(&stripped_hex[0..2], 16)
            .map_err(|_| AppError::TypeError(format!("Invalid color hex: {hex}")))?;
        let green = u8::from_str_radix(&stripped_hex[2..4], 16)
            .map_err(|_| AppError::TypeError(format!("Invalid color hex: {hex}")))?;
        let blue = u8::from_str_radix(&stripped_hex[4..6], 16)
            .map_err(|_| AppError::TypeError(format!("Invalid color hex: {hex}")))?;

        Ok(Rgb {
            r: red,
            g: green,
            b: blue,
        })
    }
}

impl From<Rgb> for String {
    fn from(value: Rgb) -> Self {
        let mut buf = String::new();
        buf.push('#');
        buf.push_str(&format!("{:02x}", value.r));
        buf.push_str(&format!("{:02x}", value.g));
        buf.push_str(&format!("{:02x}", value.b));
        buf
    }
}

// /// ARGB表現
// #[derive(Debug, Clone, Copy)]
// pub struct Argb {
//     pub a: u8,
//     pub r: u8,
//     pub g: u8,
//     pub b: u8,
// }

/// 現状wavファイルのみとする．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundFileType {
    Wav,
}

/// 音声ファイルのパスの型以下の特徴を持つ．
/// - `SoundFileType`にあるファイル形式である
/// - 存在している
/// - 読み取り可である
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub struct SoundFilePath {
    sound_file_type: SoundFileType,
    path_buf: PathBuf,
}

/// serdeではStringに変換されるため
impl Type for SoundFilePath {
    const SIGNATURE: &'static Signature = &Signature::Str;
}

impl SoundFilePath {
    pub fn sound_file_type(&self) -> SoundFileType {
        self.sound_file_type
    }
    pub fn path(&self) -> &Path {
        &self.path_buf
    }
}

impl TryFrom<String> for SoundFilePath {
    type Error = AppError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        use std::fs::File;

        let path_buf = PathBuf::from(value);

        let sound_file_type = match path_buf.extension().map(|os_str| os_str.to_str()) {
            Some(Some("wav")) => SoundFileType::Wav,
            _ => {
                return Err(AppError::TypeError(format!(
                    "Invalid sound file type: {path_buf:?}"
                )));
            }
        };

        // ファイルが開けるかどうかのチェック
        if File::open(&path_buf).is_err() {
            return Err(AppError::TypeError(format!(
                "This path is invalid. {path_buf:?}"
            )));
        }

        Ok(SoundFilePath {
            sound_file_type,
            path_buf,
        })
    }
}

impl From<SoundFilePath> for String {
    fn from(value: SoundFilePath) -> Self {
        value.path_buf.to_str().unwrap().to_owned()
    }
}

/// dbusでOptionが利用できないため，空白文字列をNoneの代わりとする型．
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(try_from = "String", into = "String")]
pub enum StringOpt<T>
where
    T: TryFrom<String, Error = AppError> + Into<String> + Clone,
{
    Value(T),
    Empty,
}

impl<T> PartialEq for StringOpt<T>
where
    T: TryFrom<String, Error = AppError> + Into<String> + Clone + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (StringOpt::Value(x), StringOpt::Value(y)) => x == y,
            (StringOpt::Empty, StringOpt::Empty) => true,
            _ => false,
        }
    }
}

impl<T> Eq for StringOpt<T> where
    T: TryFrom<String, Error = AppError> + Into<String> + Clone + PartialEq
{
}

/// serdeではStringに変換されるため
impl<T> Type for StringOpt<T>
where
    T: TryFrom<String, Error = AppError> + Into<String> + Clone,
{
    const SIGNATURE: &'static Signature = &Signature::Str;
}

impl<T> TryFrom<String> for StringOpt<T>
where
    T: TryFrom<String, Error = AppError> + Into<String> + Clone,
{
    type Error = AppError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(StringOpt::Empty);
        }

        let t_value: T = value.try_into()?;
        Ok(StringOpt::Value(t_value))
    }
}

impl<T> From<StringOpt<T>> for String
where
    T: TryFrom<String, Error = AppError> + Into<String> + Clone,
{
    fn from(value: StringOpt<T>) -> Self {
        match value {
            StringOpt::Empty => String::new(),
            StringOpt::Value(v) => v.into(),
        }
    }
}

/// ロングブレークのポジション．dbusでは列挙体が利用しにくいため文字列に変換する．
/// - EveryRap: 特定のラップ数ごとにロングブレークを置く．
/// - SpecificRap: 特定のラップごとにロングブレークを置く．
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub enum LongBreakPos {
    EveryRap(u32),
    SpecificRap(Vec<u32>),
}

/// dbus(serde)では文字列に変換するため
impl Type for LongBreakPos {
    const SIGNATURE: &'static Signature = &Signature::Str;
}

/// 文字列からの変換は以下のルールで行う
/// - "[3, 4, 5]" -> SpecificRap
/// - "4" -> EveryRap
impl TryFrom<String> for LongBreakPos {
    type Error = AppError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.starts_with('[') {
            let mut out = Vec::new();
            for elm in trimmed.trim_matches(['[', ']']).split(',') {
                out.push(elm.trim().parse::<u32>().map_err(|_| {
                    AppError::TypeError(format!("Couldn't parse as Vec<u32>: {value}"))
                })?);
            }
            Ok(LongBreakPos::SpecificRap(out))
        } else {
            Ok(LongBreakPos::EveryRap(trimmed.parse::<u32>().map_err(
                |_| AppError::TypeError(format!("Couldn't parse as u32: {value}")),
            )?))
        }
    }
}

impl From<LongBreakPos> for String {
    fn from(value: LongBreakPos) -> Self {
        match value {
            LongBreakPos::SpecificRap(raps) => {
                let raps_s = raps
                    .into_iter()
                    .map(|elm| elm.to_string())
                    .collect::<Vec<_>>();

                format!("[{}]", raps_s.join(","))
            }
            LongBreakPos::EveryRap(rap_n) => rap_n.to_string(),
        }
    }
}

/// rodioに対応する音声．標準を1として乗数で表記する．0.2 ~ 5.0の値とする．
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(try_from = "f32", into = "f32")]
pub struct SoundVolume(f32);

impl SoundVolume {
    pub fn as_float(&self) -> f32 {
        self.0
    }
}

/// dbus(serde)ではf32に変換するため
impl Type for SoundVolume {
    const SIGNATURE: &'static Signature = &Signature::F64;
}

#[allow(clippy::manual_range_contains)]
impl TryFrom<f32> for SoundVolume {
    type Error = AppError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value < 0.2 || 5.0 < value {
            Err(AppError::TypeError(format!(
                "SoundVolume must be in the range of [0.2, 5.0]. :{value}"
            )))
        } else {
            Ok(SoundVolume(value))
        }
    }
}

impl From<SoundVolume> for f32 {
    fn from(value: SoundVolume) -> Self {
        value.as_float()
    }
}

impl TryFrom<String> for SoundVolume {
    type Error = AppError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value_f32 = value
            .parse::<f32>()
            .map_err(|_| AppError::TypeError(format!("Couldn't parse as f32. :{value}")))?;
        value_f32.try_into()
    }
}

impl From<SoundVolume> for String {
    fn from(value: SoundVolume) -> Self {
        value.0.to_string()
    }
}

impl PartialEq for SoundVolume {
    fn eq(&self, other: &Self) -> bool {
        const EPSILON: f32 = 1.0e-5;

        (self.0 - other.0).abs() < EPSILON
    }
}

impl Eq for SoundVolume {}

#[cfg(test)]
mod test {
    #[test]
    fn test_sound_file() {
        use super::{SoundFilePath, SoundFileType};
        let valid_path = "./assets/sounds/break2work.wav".to_owned();
        let path = SoundFilePath::try_from(valid_path).unwrap();
        assert!(matches!(path.sound_file_type(), SoundFileType::Wav));

        let invalid_path = "hogehoge.wav".to_owned();
        assert!(SoundFilePath::try_from(invalid_path).is_err());
    }

    #[test]
    fn test_string_opt() {
        use super::{SoundFilePath, StringOpt};
        let empty = "".to_owned();
        assert!(matches!(
            StringOpt::<SoundFilePath>::try_from(empty),
            Ok(StringOpt::Empty)
        ));
        let valid_path = "./assets/sounds/work2break.wav".to_owned();
        assert!(matches!(
            StringOpt::<SoundFilePath>::try_from(valid_path),
            Ok(StringOpt::Value(_))
        ));

        assert_eq!(
            Into::<String>::into(StringOpt::<SoundFilePath>::Empty),
            "".to_owned()
        );

        let valid_path = "./assets/sounds/work2break.wav".to_owned();
        assert_eq!(
            Into::<String>::into(StringOpt::Value(
                SoundFilePath::try_from(valid_path.clone()).unwrap()
            )),
            valid_path
        )
    }

    #[test]
    fn test_long_break_pos() {
        use super::LongBreakPos;

        let every_rap = "5".to_owned();
        assert!(matches!(
            LongBreakPos::try_from(every_rap.clone()).unwrap(),
            LongBreakPos::EveryRap(_)
        ));
        assert_eq!(
            Into::<String>::into(LongBreakPos::try_from(every_rap.clone()).unwrap()),
            every_rap
        );

        let specific_raps = "[1,4,6]".to_owned();
        assert!(matches!(
            LongBreakPos::try_from(specific_raps.clone()).unwrap(),
            LongBreakPos::SpecificRap(_)
        ));
        assert_eq!(
            Into::<String>::into(LongBreakPos::try_from(specific_raps.clone()).unwrap()),
            specific_raps
        );
    }
}
