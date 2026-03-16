use crate::{AppConfig, AppError};

use lexopt::{
    Parser, ValueExt,
    prelude::{Long, Short},
};
use log::Level;

const HELP: &str = r###"The pomodoro timer displays on the statusbar as an implementation of SNI(StatusNotifierItem). 

Commands:
    pomodoro-sni app [Options]
        Pomodoro timer application.

    pomodoro-sni configure
        Interactive CLI application which communicates with the main application and configures it.

    pomodoro-sni start
        Start the timer from external application.

Options:
    -h|--help
        Print help.

    -d|--debug
        Enable debug logging.

    --work-time <int>
        Working time[min].

    --short-break-time <int>
        Short break time[min].

    --long-break-time
        Long break time[min].

    --work-font-color <string>
        Font color as hex (like "#ffffff") while Working.

    --work-bg-color <string>
        Background color as hex (like "#ffffff") while Working.

    --short-break-font-color <string>
        Font color as hex (like "#ffffff") while Short Break.

    --short-break-bg-color <string>
        Background color as hex (like "#ffffff") while Short Break.

    --long-break-font-color <string>
        Font color as hex (like "#ffffff") while Long Break.

    --long-break-bg-color <string>
        Background color as hex (like "#ffffff") while Long Break.

    --break2work-path <string>
        The wav file path for break2work sound.

    --work2break-path <string>
        The wav file path for work2break sound.

    --long-break-pos <integer or list<integer>>
        The Long Break strategy as an integer or a list of integer (like "5" or "[1, 3]").

    --sound-volume <float>
        The volume of sound [0.2, 5.0]."###;

/// CLIのサブコマンド
#[derive(Debug, Clone)]
pub enum CliSubcommand {
    /// アプリケーション
    App,
    /// インタラクティブな設定アプリ
    Configure,
    /// 外部からの開始
    Start,
}

#[derive(Debug, Clone)]
pub struct Args {
    pub subcommand: CliSubcommand,
    pub log_level: Level,
    pub app_config: AppConfig,
}

pub fn parse_args(mut app_config: AppConfig) -> Result<Args, AppError> {
    let mut log_level = Level::Info;

    let mut parser = Parser::from_env();

    let subcommand_name = match parser.value() {
        Ok(os_str) => match os_str.to_str() {
            Some("app") => "app",
            Some("configure") => "configure",
            Some("start") => "start",
            Some("-h") | Some("--help") => {
                println!("{HELP}");
                std::process::exit(0);
            }
            _ => {
                return Err(AppError::ArgError(r#"pomodoro-sni has two subcommand "pomodoro-sni app" and "pomodoro-sni configure"."#.to_owned()));
            }
        },
        Err(_) => "app", // バイナリ名単体の場合
    };

    match subcommand_name {
        "app" => {
            while let Some(arg) = parser.next()? {
                match arg {
                    Short('h') | Long("help") => {
                        println!("{HELP}");
                        std::process::exit(0);
                    }
                    Short('d') | Long("debug") => {
                        log_level = Level::Debug;
                    }
                    Long("work-time") => {
                        app_config.work_time = parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("short-break-time") => {
                        app_config.short_break_time =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("long-break-time") => {
                        app_config.short_break_time =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("work-font-color") => {
                        app_config.work_font_color =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("work-bg-color") => {
                        app_config.work_bg_color = parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("short-break-font-color") => {
                        app_config.short_break_font_color =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("short-break-bg-color") => {
                        app_config.short_break_bg_color =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("long-break-font-color") => {
                        app_config.long_break_font_color =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("long-break-bg-color") => {
                        app_config.long_break_bg_color =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("break2work-path") => {
                        app_config.sound_break_to_work =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("work2break-path") => {
                        app_config.sound_work_to_break =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("long-break-pos") => {
                        app_config.long_break_pos =
                            parser.value()?.parse::<String>()?.try_into()?;
                    }
                    Long("sound-volume") => {
                        app_config.sound_volume = parser.value()?.parse::<String>()?.try_into()?;
                    }
                    _ => Err(arg.unexpected())?,
                }
            }

            Ok(Args {
                subcommand: CliSubcommand::App,
                log_level,
                app_config,
            })
        }
        "configure" => Ok(Args {
            subcommand: CliSubcommand::Configure,
            log_level,
            app_config,
        }),
        "start" => Ok(Args {
            subcommand: CliSubcommand::Start,
            log_level,
            app_config,
        }),
        _ => unreachable!("Internal bug."),
    }
}
