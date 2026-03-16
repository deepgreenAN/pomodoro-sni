use crate::{AppConfig, AppError};
use dialoguer::{Input, Select};

/// インタラクティブにコンフィグを設定できるループ
pub fn run_interactive_config_loop<F: FnMut(AppConfig) -> Result<(), AppError>>(
    config: AppConfig,
    mut apply_cb: F,
) -> Result<(), AppError> {
    fn app_select(prompt: &str, items: &[&str]) -> Result<usize, AppError> {
        let index = Select::new()
            .items(items)
            .with_prompt(prompt)
            .default(0)
            .report(false)
            .interact()?;

        Ok(index)
    }

    /// `default_value`がSomeの場合，空白文字列を受け取ると`default_value`となる．Noneの場合空白文字列そのものを受け取る．
    fn app_input<T: TryFrom<String, Error = AppError> + Into<String>>(
        prompt: &str,
        initial_value: T,
        default_value: Option<T>,
    ) -> Result<T, AppError> {
        let validate = |input: &String| -> Result<(), String> {
            T::try_from(input.clone()).map_err(|e| e.to_string())?;
            Ok(())
        };

        let mut input = Input::new()
            .with_prompt(prompt)
            .with_initial_text(initial_value)
            .report(false)
            .validate_with(validate)
            .allow_empty(true);

        if let Some(default_value) = default_value {
            input = input.default(default_value.into()).show_default(false);
        }

        let input_text: String = input.interact_text()?;
        T::try_from(input_text)
    }

    let mut current_config = config;
    let default_config = AppConfig::default();

    let root_setting_items = [
        "⏰ Time",
        "🎨 Color",
        "🎵 Sound",
        "   Long Break Position",
        "   default",
        "✅ apply",
        "❎ close",
    ];

    'root: loop {
        let root_index = app_select("🔧 Settings", &root_setting_items)?;
        match *(root_setting_items.get(root_index).unwrap()) {
            "⏰ Time" => 'time: loop {
                let phases = ["📝 Working", "🍵 Short Break", "🍙 Long Break", "🔙 back"];

                let time_index = app_select("Time Setting", &phases)?;
                match *(phases.get(time_index).unwrap()) {
                    "📝 Working" => {
                        current_config.work_time = app_input(
                            &format!(
                                "Enter the Working Time[min]. (default \"{}\")\n",
                                String::from(default_config.work_time)
                            ),
                            current_config.work_time,
                            Some(default_config.work_time),
                        )?;
                    }
                    "🍵 Short Break" => {
                        current_config.short_break_time = app_input(
                            &format!(
                                "Enter the Short Break Time[min]. (default \"{}\")\n",
                                String::from(default_config.short_break_time)
                            ),
                            current_config.short_break_time,
                            Some(default_config.short_break_time),
                        )?;
                    }
                    "🍙 Long Break" => {
                        current_config.long_break_time = app_input(
                            &format!(
                                "Enter the Long Break Time[min]. (default \"{}\")\n",
                                String::from(default_config.long_break_time)
                            ),
                            current_config.long_break_time,
                            Some(default_config.long_break_time),
                        )?;
                    }
                    "🔙 back" => break 'time,
                    _ => {
                        return Err(AppError::CustomError(
                            "Internal bug. invalid index.".to_owned(),
                        ));
                    }
                }
            },
            "🎨 Color" => 'color: loop {
                let phases = ["📝 Working", "🍵 Short Break", "🍙 Long Break", "🔙 back"];
                let color_types = ["❌ font", "❎ background", "🔙 back"];

                let color_index = app_select("🎨 Color Setting", &phases)?;

                match *(phases.get(color_index).unwrap()) {
                    "📝 Working" => 'color_working: loop {
                        let color_type_index = app_select("Working Color Setting", &color_types)?;
                        match *(color_types.get(color_type_index).unwrap()) {
                            "❌ font" => {
                                current_config.work_font_color = app_input(
                                    &format!(
                                        "Enter the font color while Working. (default \"{}\")\n",
                                        String::from(default_config.work_font_color)
                                    ),
                                    current_config.work_font_color,
                                    Some(default_config.work_font_color),
                                )?;
                            }
                            "❎ background" => {
                                current_config.work_bg_color = app_input(
                                    &format!(
                                        "Enter the background color while Working. (default \"{}\")\n",
                                        String::from(default_config.work_bg_color)
                                    ),
                                    current_config.work_bg_color,
                                    Some(default_config.work_bg_color),
                                )?;
                            }
                            "🔙 back" => break 'color_working,
                            _ => {
                                return Err(AppError::CustomError(
                                    "Internal bug. invalid index.".to_owned(),
                                ));
                            }
                        }
                    },
                    "🍵 Short Break" => 'color_short_break: loop {
                        let color_type_index =
                            app_select("Short Break Color Setting", &color_types)?;
                        match *(color_types.get(color_type_index).unwrap()) {
                            "❌ font" => {
                                current_config.short_break_font_color = app_input(
                                    &format!(
                                        "Enter the font color while Short Break. (default \"{}\")\n",
                                        String::from(default_config.short_break_font_color)
                                    ),
                                    current_config.short_break_font_color,
                                    Some(default_config.short_break_font_color),
                                )?;
                            }
                            "❎ background" => {
                                current_config.short_break_bg_color = app_input(
                                    &format!(
                                        "Enter the background color while Short Break. (default \"{}\")\n",
                                        String::from(default_config.short_break_bg_color)
                                    ),
                                    current_config.short_break_bg_color,
                                    Some(default_config.short_break_bg_color),
                                )?;
                            }
                            "🔙 back" => break 'color_short_break,
                            _ => {
                                return Err(AppError::CustomError(
                                    "Internal bug. invalid index.".to_owned(),
                                ));
                            }
                        }
                    },
                    "🍙 Long Break" => 'color_short_break: loop {
                        let color_type_index =
                            app_select("Long Break Color Setting", &color_types)?;
                        match *(color_types.get(color_type_index).unwrap()) {
                            "❌ font" => {
                                current_config.long_break_font_color = app_input(
                                    &format!(
                                        "Enter the font color while Long Break. (default \"{}\")\n",
                                        String::from(default_config.long_break_font_color)
                                    ),
                                    current_config.long_break_font_color,
                                    Some(default_config.long_break_font_color),
                                )?;
                            }
                            "❎ background" => {
                                current_config.long_break_bg_color = app_input(
                                    &format!(
                                        "Enter the background color while Long Break. (default \"{}\")\n",
                                        String::from(default_config.long_break_bg_color)
                                    ),
                                    current_config.long_break_bg_color,
                                    Some(current_config.long_break_bg_color),
                                )?;
                            }
                            "🔙 back" => break 'color_short_break,
                            _ => {
                                return Err(AppError::CustomError(
                                    "Internal bug. invalid index.".to_owned(),
                                ));
                            }
                        }
                    },
                    "🔙 back" => break 'color,
                    _ => {
                        return Err(AppError::CustomError(
                            "Internal bug. invalid index.".to_owned(),
                        ));
                    }
                }
            },
            "🎵 Sound" => 'sound: loop {
                let setting_items = ["💿 Sound File", "🔊 Volume", "🔙 back"];

                let sound_index = app_select("🎵 Sound Setting", &setting_items)?;
                match *(setting_items.get(sound_index).unwrap()) {
                    "💿 Sound File" => 'sound_file: loop {
                        let sound_types = ["📝 Break2Work", "🍵 Work2Break", "🔙 back"];

                        let sound_file_index = app_select("💿 Sound File Setting", &sound_types)?;
                        match *(sound_types.get(sound_file_index).unwrap()) {
                            "📝 Break2Work" => {
                                current_config.sound_break_to_work = app_input(
                                    &format!(
                                        "Enter the wav file path for Break2Work sound. \nEmpty input means to use the default sound. (default \"{}\")\n",
                                        String::from(default_config.sound_break_to_work.clone())
                                    ),
                                    current_config.sound_break_to_work,
                                    None,
                                )?;
                            }
                            "🍵 Work2Break" => {
                                current_config.sound_work_to_break = app_input(
                                    &format!(
                                        "Enter the wav file path for Work2Break sound. \nEmpty input means to use the default sound. (default \"{}\")\n",
                                        String::from(default_config.sound_work_to_break.clone())
                                    ),
                                    current_config.sound_work_to_break,
                                    None,
                                )?;
                            }
                            "🔙 back" => break 'sound_file,
                            _ => {
                                return Err(AppError::CustomError(
                                    "Internal bug. invalid index.".to_owned(),
                                ));
                            }
                        }
                    },
                    "🔊 Volume" => {
                        current_config.sound_volume = app_input(
                            &format!(
                                "Enter the Sound Volume(0.2 ~ 5.0). (default \"{}\")\n",
                                String::from(default_config.sound_volume)
                            ),
                            current_config.sound_volume,
                            Some(default_config.sound_volume),
                        )?;
                    }
                    "🔙 back" => break 'sound,
                    _ => {
                        return Err(AppError::CustomError(
                            "Internal bug. invalid index.".to_owned(),
                        ));
                    }
                }
            },
            "   Long Break Position" => {
                current_config.long_break_pos = app_input(
                    &format!(
                        "Enter the Long Break Strategy. \nIf you enter a single integer(like \"5\"), Long Break will be inserted at every specified raps.\nIf you enter a list of integer(like \"[2, 5, 7]\"), Long Break will be inserted at specific positions of rap. (default \"{}\")\n",
                        String::from(default_config.long_break_pos.clone()),
                    ),
                    current_config.long_break_pos,
                    Some(default_config.long_break_pos.clone()),
                )?;
            }
            "   default" => {
                current_config = AppConfig::default();
            }
            "✅ apply" => {
                apply_cb(current_config.clone())?;
            }
            "❎ close" => break 'root,
            _ => {
                return Err(AppError::CustomError(
                    "Internal bug. invalid index.".to_owned(),
                ));
            }
        }
    }

    Ok(())
}
