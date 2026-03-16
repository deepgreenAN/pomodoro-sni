use pomodoro_sni::{
    AppConfig, AppError, Args, CliSubcommand, Duration, parse_args, read_config, run_audio_loop,
    run_main_loop, run_timer_loop, write_config,
};

use async_channel::bounded;
use etcetera::{AppStrategy, AppStrategyArgs, choose_app_strategy};
use log::error;

const DOMAIN: &str = "io";
const AUTHOR: &str = "dgan";
const APP_NAME: &str = "pomodoro-sni";

fn read_cache_or_default() -> Result<AppConfig, AppError> {
    let strategy = choose_app_strategy(AppStrategyArgs {
        top_level_domain: DOMAIN.to_owned(),
        author: AUTHOR.to_owned(),
        app_name: APP_NAME.to_owned(),
    })
    .map_err(|e| AppError::CustomError(e.to_string()))?;

    let cache_path = strategy.in_cache_dir("settings.txt");

    if cache_path.exists() {
        Ok(read_config(&cache_path)?)
    } else {
        Ok(AppConfig::default())
    }
}

fn write_cache(app_config: &AppConfig) -> Result<(), AppError> {
    let strategy = choose_app_strategy(AppStrategyArgs {
        top_level_domain: DOMAIN.to_owned(),
        author: AUTHOR.to_owned(),
        app_name: APP_NAME.to_owned(),
    })
    .map_err(|e| AppError::CustomError(e.to_string()))?;

    if !strategy.cache_dir().exists() {
        std::fs::create_dir_all(strategy.cache_dir())?;
    }
    let cache_path = strategy.in_cache_dir("settings.txt");

    write_config(&cache_path, app_config)
}

fn main() -> Result<(), AppError> {
    let Args {
        subcommand,
        log_level,
        app_config,
    } = parse_args(read_cache_or_default()?)?;
    simple_logger::init_with_level(log_level).map_err(|e| AppError::CustomError(e.to_string()))?;

    match subcommand {
        CliSubcommand::Configure => {
            use pomodoro_sni::{PomodoroSniControllerProxyBlocking, run_interactive_config_loop};
            use zbus::blocking::Connection;
            use zbus::fdo::RequestNameFlags;

            let conn = Connection::session()?;
            // 同じ名前を要求した場合に失敗する．
            conn.request_name_with_flags(
                "org.zbus.PomodoroSniClient",
                RequestNameFlags::DoNotQueue.into(),
            )?;

            let controller_proxy = PomodoroSniControllerProxyBlocking::new(&conn)?;

            let initial_config = controller_proxy.get_config()?;
            let apply_cb = |app_config: AppConfig| -> Result<(), AppError> {
                write_cache(&app_config)?;
                controller_proxy.set_config(app_config)?;
                Ok(())
            };

            run_interactive_config_loop(initial_config, apply_cb)?;

            Ok(())
        }
        CliSubcommand::Start => {
            use pomodoro_sni::PomodoroSniControllerProxyBlocking;
            use zbus::blocking::Connection;
            use zbus::fdo::RequestNameFlags;

            let conn = Connection::session()?;
            // 同じ名前を要求した場合に失敗する．
            conn.request_name_with_flags(
                "org.zbus.PomodoroSniClient",
                RequestNameFlags::DoNotQueue.into(),
            )?;

            let controller_proxy = PomodoroSniControllerProxyBlocking::new(&conn)?;
            controller_proxy.external_start()?;

            Ok(())
        }
        CliSubcommand::App => {
            let (timer_cmd_sender, timer_cmd_receiver) = bounded(5);
            let (app_info_sender, app_info_receiver) = bounded(5);
            let (audio_cmd_sender, audio_cmd_receiver) = bounded(5);

            let _timer_job_handle = std::thread::spawn({
                let app_info_sender = app_info_sender.clone();
                move || {
                    if let Err(e) = run_timer_loop(
                        timer_cmd_receiver,
                        app_info_sender,
                        Duration::milliseconds(100),
                    ) {
                        error!("{e}");
                        Err(e)
                    } else {
                        Ok(())
                    }
                }
            });

            let _audio_job_handle = std::thread::spawn(move || {
                if let Err(e) = run_audio_loop(audio_cmd_receiver) {
                    error!("{e}");
                    Err(e)
                } else {
                    Ok(())
                }
            });

            smol::block_on(async move {
                run_main_loop(
                    app_config,
                    timer_cmd_sender,
                    app_info_sender,
                    app_info_receiver,
                    audio_cmd_sender,
                )
                .await
            })?;

            Ok(())
        }
    }
}
