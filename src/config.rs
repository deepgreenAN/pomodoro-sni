// use std::collections::HashMap;
use std::path::Path;
use std::stringify;

use crate::{AppError, LongBreakPos, Rgb, SoundFilePath, SoundVolume, StringOpt, TimerMinute};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use zbus::zvariant::Type;

/// アプリケーションのコンフィグ．
#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq, Eq)]
pub struct AppConfig {
    pub work_font_color: Rgb,
    pub work_bg_color: Rgb,
    pub short_break_font_color: Rgb,
    pub short_break_bg_color: Rgb,
    pub long_break_font_color: Rgb,
    pub long_break_bg_color: Rgb,
    pub work_time: TimerMinute,
    pub short_break_time: TimerMinute,
    pub long_break_time: TimerMinute,
    pub long_break_pos: LongBreakPos,
    pub sound_break_to_work: StringOpt<SoundFilePath>,
    pub sound_work_to_break: StringOpt<SoundFilePath>,
    pub sound_volume: SoundVolume,
}

#[rustfmt::skip]
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            work_font_color: Rgb { r: 255, g: 255, b: 255 },
            work_bg_color: Rgb { r: 0, g: 0, b: 0 },
            short_break_font_color: Rgb { r: 255, g: 255, b: 255 },
            short_break_bg_color: Rgb { r: 0, g: 0, b: 0 },
            long_break_font_color: Rgb { r: 255, g: 255, b: 255 },
            long_break_bg_color: Rgb { r: 0, g: 0, b: 0 },
            work_time: 25.try_into().unwrap(),
            short_break_time: 10.try_into().unwrap(),
            long_break_time: 15.try_into().unwrap(),
            long_break_pos: LongBreakPos::EveryRap(4),
            sound_break_to_work: StringOpt::Empty,
            sound_work_to_break: StringOpt::Empty,
            sound_volume: 1.0_f32.try_into().unwrap()
        }
    }
}

impl AppConfig {
    /// 設定ファイル作成用
    fn string_map(&self) -> IndexMap<String, String> {
        let AppConfig {
            work_font_color,
            work_bg_color,
            short_break_font_color,
            short_break_bg_color,
            long_break_font_color,
            long_break_bg_color,
            work_time,
            short_break_time,
            long_break_time,
            long_break_pos,
            sound_break_to_work,
            sound_work_to_break,
            sound_volume,
        } = &self;

        let mut map = IndexMap::<String, String>::new();
        map.insert(
            stringify!(work_font_color).to_owned(),
            (*work_font_color).into(),
        );
        map.insert(
            stringify!(work_bg_color).to_owned(),
            (*work_bg_color).into(),
        );
        map.insert(
            stringify!(short_break_font_color).to_owned(),
            (*short_break_font_color).into(),
        );
        map.insert(
            stringify!(short_break_bg_color).to_owned(),
            (*short_break_bg_color).into(),
        );
        map.insert(
            stringify!(long_break_font_color).to_owned(),
            (*long_break_font_color).into(),
        );
        map.insert(
            stringify!(long_break_bg_color).to_owned(),
            (*long_break_bg_color).into(),
        );
        map.insert(stringify!(work_time).to_owned(), (*work_time).into());
        map.insert(
            stringify!(short_break_time).to_owned(),
            (*short_break_time).into(),
        );
        map.insert(
            stringify!(long_break_time).to_owned(),
            (*long_break_time).into(),
        );
        map.insert(
            stringify!(long_break_pos).to_owned(),
            long_break_pos.clone().into(),
        );
        map.insert(
            stringify!(sound_break_to_work).to_owned(),
            sound_break_to_work.clone().into(),
        );
        map.insert(
            stringify!(sound_work_to_break).to_owned(),
            sound_work_to_break.clone().into(),
        );
        map.insert(stringify!(sound_volume).to_owned(), (*sound_volume).into());

        map
    }

    fn try_from_string_map(string_map: IndexMap<String, String>) -> Result<Self, AppError> {
        fn get_and_try_from<T: TryFrom<String, Error = AppError>>(
            map: &IndexMap<String, String>,
            key: &str,
        ) -> Result<T, AppError> {
            map.get(key)
                .ok_or(AppError::CustomError(
                    "Could'nt get value from string map.".to_owned(),
                ))?
                .clone()
                .try_into()
        }
        let work_font_color = get_and_try_from(&string_map, stringify!(work_font_color))?;
        let work_bg_color = get_and_try_from(&string_map, stringify!(work_bg_color))?;
        let short_break_font_color =
            get_and_try_from(&string_map, stringify!(short_break_font_color))?;
        let short_break_bg_color = get_and_try_from(&string_map, stringify!(short_break_bg_color))?;
        let long_break_font_color =
            get_and_try_from(&string_map, stringify!(long_break_font_color))?;
        let long_break_bg_color = get_and_try_from(&string_map, stringify!(long_break_bg_color))?;
        let work_time = get_and_try_from(&string_map, stringify!(work_time))?;
        let short_break_time = get_and_try_from(&string_map, stringify!(short_break_time))?;
        let long_break_time = get_and_try_from(&string_map, stringify!(long_break_time))?;
        let long_break_pos = get_and_try_from(&string_map, stringify!(long_break_pos))?;
        let sound_break_to_work = get_and_try_from(&string_map, stringify!(sound_break_to_work))?;
        let sound_work_to_break = get_and_try_from(&string_map, stringify!(sound_work_to_break))?;
        let sound_volume = get_and_try_from(&string_map, stringify!(sound_volume))?;

        Ok(Self {
            work_font_color,
            work_bg_color,
            short_break_font_color,
            short_break_bg_color,
            long_break_font_color,
            long_break_bg_color,
            work_time,
            short_break_time,
            long_break_time,
            long_break_pos,
            sound_break_to_work,
            sound_work_to_break,
            sound_volume,
        })
    }
}

/// Configの読み込み
pub fn read_config(path: &Path) -> Result<AppConfig, AppError> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    let file = BufReader::new(File::open(path)?);

    let mut map = IndexMap::<String, String>::new();

    for line in file.lines() {
        let line = line?;
        let line_vec = line
            .trim()
            .split('=')
            .map(|elm| elm.trim())
            .collect::<Vec<_>>();
        let key = line_vec
            .first()
            .ok_or(AppError::CustomError("Invalid config file.".to_owned()))?
            .to_string();
        let value = line_vec
            .get(1)
            .ok_or(AppError::CustomError("Invalid config file.".to_owned()))?
            .to_string();

        map.insert(key, value);
    }

    AppConfig::try_from_string_map(map)
}

/// Configの書き出し
pub fn write_config(path: &Path, app_config: &AppConfig) -> Result<(), AppError> {
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create(path)?;
    let app_config_map = app_config.string_map();

    for (key, value) in app_config_map.into_iter() {
        file.write_all(format!("{key}={value}\n").as_bytes())?;
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::config::{read_config, write_config};

    #[test]
    fn test_write_and_read() {
        use super::AppConfig;
        use std::path::PathBuf;

        let test_config_path = PathBuf::from("./temp/test_config.txt");
        write_config(&test_config_path, &AppConfig::default()).unwrap();

        assert_eq!(
            read_config(&test_config_path).unwrap(),
            AppConfig::default()
        );
    }
}
