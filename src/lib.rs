mod args;
mod audio;
mod config;
mod dbus;
mod digit;
mod error;
mod interactive;
pub mod terminal;
mod timer;
mod types;

pub use args::{Args, CliSubcommand, parse_args};
pub use audio::{AudioCommand, SoundFileType, run_audio_loop};
pub use config::{AppConfig, read_config, write_config};
pub use dbus::PomodoroSniControllerProxyBlocking;
use dbus::{DbusMenu, PomodoroSniController, StatusNotifierItem, StatusNotifierWatcherProxy};
use digit::DoubleDigit;
pub use error::{AppError, try_send_error_to_app_error};
pub use interactive::run_interactive_config_loop;
pub use timer::{Duration, TimerCommand, run_timer_loop};
pub use types::{LongBreakPos, Rgb, SoundFilePath, SoundVolume, StringOpt, TimerMinute};

use async_channel::{Receiver, Sender};
use log::info;
use zbus::conn::Builder;

/// メインループで利用するタイマーと設定の情報
#[derive(Debug, Clone)]
pub enum AppInfo {
    /// -----------------------------
    /// 以下はタイマーが送信する．
    ///
    /// タイマーが開始した
    TimerStarted(TimerMinute),
    /// 秒が経過．データは残り時間．
    Seconds(u32),
    /// 分が経過．データは残り時間．
    Minutes(TimerMinute),
    /// タイマーがポーズ状態になった
    TimerPaused,
    /// タイマーが再開した
    TimerResumed,
    /// タイマーがリセットされた
    TimerReset,
    /// タイマーがスキップされた
    TimerSkipped,
    /// タイマーが完了した．
    TimerFinished,
    /// タイマースレッドが終了した．
    TimerTerminated,
    /// -----------------------------
    /// 以下はControllerが送信する．
    ///
    /// 構成を受け取った
    ReceivedConfigure(AppConfig),
    /// 外部スタートコマンドを受け取った．
    ReceivedExternalStart,
}

/// タイマーのフェイズ
#[derive(Debug, Clone, Copy)]
enum Phase {
    Working,
    ShortBreak,
    LongBreak,
}

impl From<Phase> for String {
    fn from(value: Phase) -> Self {
        match value {
            Phase::Working => "Working".to_owned(),
            Phase::ShortBreak => "ShortBreak".to_owned(),
            Phase::LongBreak => "LongBreak".to_owned(),
        }
    }
}

pub const APP_SERVICE_NAME: &str = "org.zbus.PomodoroSni";
pub const SNI_OBJECT_PATH: &str = "/StatusNotifierItem";
pub const DBUS_MENU_OBJECT_PATH: &str = "/DbusMenu";
pub const CONTROLLER_OBJECT_PATH: &str = "/PomodoroSniController";

/// タイマーの結果を表示する+設定の変更を反映するメインのループ．
pub async fn run_main_loop(
    app_config: AppConfig,
    timer_cmd_sender: Sender<TimerCommand>,
    app_info_sender: Sender<AppInfo>,
    app_info_receiver: Receiver<AppInfo>,
    audio_cmd_sender: Sender<AudioCommand>,
) -> Result<(), AppError> {
    let bitmap = DoubleDigit::from(app_config.work_time).bitmap_flatten();

    let sni = StatusNotifierItem::new(bitmap, app_config.work_font_color, app_config.work_bg_color);
    let dbus_menu = DbusMenu::new(timer_cmd_sender, app_config.work_time);
    let controller = PomodoroSniController::new(app_config.clone(), app_info_sender);

    let conn = Builder::session()?
        .name(APP_SERVICE_NAME)?
        .allow_name_replacements(false)
        .replace_existing_names(false)
        .serve_at(SNI_OBJECT_PATH, sni)?
        .serve_at(DBUS_MENU_OBJECT_PATH, dbus_menu)?
        .serve_at(CONTROLLER_OBJECT_PATH, controller)?
        .build()
        .await?;

    let unique_name = conn
        .unique_name()
        .ok_or(AppError::ZbusError(
            "Couldn't get the unique name.".to_owned(),
        ))?
        .to_string();
    info!("Dbus service unique name: {unique_name}");

    // watcherへの登録
    let watcher_proxy = StatusNotifierWatcherProxy::new(&conn).await?;

    watcher_proxy
        .register_status_notifier_item(APP_SERVICE_NAME.to_string())
        .await?;

    let mut current_phase = Phase::Working;
    let mut current_app_config = app_config;
    let mut rap_counter = 1;

    while let Ok(app_info) = app_info_receiver.recv().await {
        match app_info {
            AppInfo::TimerStarted(start_minutes) => {
                // 音声の再生
                let sound_type = match &current_phase {
                    Phase::Working => SoundFileType::Break2Work,
                    Phase::LongBreak | Phase::ShortBreak => SoundFileType::Work2Break,
                };

                audio_cmd_sender
                    .try_send(AudioCommand::Play {
                        sound_file_type: sound_type,
                    })
                    .map_err(|e| {
                        try_send_error_to_app_error(e, "run_main_loop: audio_cmd_sender".to_owned())
                    })?;

                // アイコン画像の変更
                let bitmap = DoubleDigit::from(start_minutes).bitmap_flatten();
                let (font_color, bg_color) = match &current_phase {
                    Phase::Working => (
                        current_app_config.work_font_color,
                        current_app_config.work_bg_color,
                    ),
                    Phase::ShortBreak => (
                        current_app_config.short_break_font_color,
                        current_app_config.short_break_bg_color,
                    ),
                    Phase::LongBreak => (
                        current_app_config.long_break_font_color,
                        current_app_config.long_break_bg_color,
                    ),
                };
                {
                    let sni_iface_ref = conn
                        .object_server()
                        .interface::<_, StatusNotifierItem>(SNI_OBJECT_PATH)
                        .await?;
                    let mut sni_refmut = sni_iface_ref.get_mut().await;
                    sni_refmut.bitmap = bitmap;
                    sni_refmut.font_color = font_color;
                    sni_refmut.bg_color = bg_color;

                    sni_refmut
                        .emit_new_icon(sni_iface_ref.signal_emitter())
                        .await?;
                    sni_refmut
                        .icon_pixmap_changed(sni_iface_ref.signal_emitter())
                        .await?;
                }
            }
            AppInfo::Minutes(rest_minutes) => {
                // アイコン画像の変更
                let bitmap = DoubleDigit::from(rest_minutes).bitmap_flatten();

                {
                    let sni_iface_ref = conn
                        .object_server()
                        .interface::<_, StatusNotifierItem>(SNI_OBJECT_PATH)
                        .await?;
                    let mut sni_refmut = sni_iface_ref.get_mut().await;
                    sni_refmut.bitmap = bitmap;
                    sni_refmut
                        .emit_new_icon(sni_iface_ref.signal_emitter())
                        .await?;
                    sni_refmut
                        .icon_pixmap_changed(sni_iface_ref.signal_emitter())
                        .await?;
                }
            }
            AppInfo::TimerFinished => {
                // 次のタイマーの開始
                // フェイズの切り替え
                match &current_phase {
                    Phase::Working => match &current_app_config.long_break_pos {
                        LongBreakPos::EveryRap(every) => {
                            if rap_counter % every == 0 {
                                current_phase = Phase::LongBreak;
                            } else {
                                current_phase = Phase::ShortBreak;
                            }
                        }
                        LongBreakPos::SpecificRap(specific_raps) => {
                            if specific_raps.contains(&rap_counter) {
                                current_phase = Phase::LongBreak;
                            } else {
                                current_phase = Phase::ShortBreak;
                            }
                        }
                    },
                    Phase::LongBreak | Phase::ShortBreak => {
                        current_phase = Phase::Working;
                    }
                };

                let next_time = match &current_phase {
                    Phase::Working => current_app_config.work_time,
                    Phase::ShortBreak => current_app_config.short_break_time,
                    Phase::LongBreak => current_app_config.long_break_time,
                };

                {
                    let dbus_menu_iface_ref = conn
                        .object_server()
                        .interface::<_, DbusMenu>(DBUS_MENU_OBJECT_PATH)
                        .await?;
                    let mut dbus_menu_refmut = dbus_menu_iface_ref.get_mut().await;

                    dbus_menu_refmut.set_time(next_time);
                    let mut update_properties = Vec::new();

                    update_properties.append(&mut (dbus_menu_refmut.set_phase(current_phase)?));
                    if let Phase::Working = &current_phase {
                        rap_counter += 1;
                        update_properties.append(&mut (dbus_menu_refmut.set_rap(rap_counter)?));
                    }
                    dbus_menu_refmut.pending()?;
                    update_properties.append(&mut (dbus_menu_refmut.start()?));

                    dbus_menu_refmut
                        .emit_items_properties_updated(
                            dbus_menu_iface_ref.signal_emitter(),
                            update_properties,
                        )
                        .await?;
                }
            }
            AppInfo::TimerPaused => {}
            AppInfo::TimerResumed => {}
            AppInfo::TimerReset => {
                // リセット後の処理
                current_phase = Phase::Working;
                let next_time = current_app_config.work_time;
                rap_counter = 1;

                {
                    let dbus_menu_iface_ref = conn
                        .object_server()
                        .interface::<_, DbusMenu>(DBUS_MENU_OBJECT_PATH)
                        .await?;
                    let mut dbus_menu_refmut = dbus_menu_iface_ref.get_mut().await;

                    dbus_menu_refmut.set_time(next_time);
                    let mut update_properties = Vec::new();

                    update_properties.append(&mut (dbus_menu_refmut.set_phase(current_phase)?));
                    update_properties.append(&mut (dbus_menu_refmut.set_rap(rap_counter)?));

                    dbus_menu_refmut
                        .emit_items_properties_updated(
                            dbus_menu_iface_ref.signal_emitter(),
                            update_properties,
                        )
                        .await?;
                }

                // アイコン画像の変更
                let bitmap = DoubleDigit::from(next_time).bitmap_flatten();
                let (font_color, bg_color) = (
                    current_app_config.work_font_color,
                    current_app_config.work_bg_color,
                );
                {
                    let sni_iface_ref = conn
                        .object_server()
                        .interface::<_, StatusNotifierItem>(SNI_OBJECT_PATH)
                        .await?;
                    let mut sni_refmut = sni_iface_ref.get_mut().await;
                    sni_refmut.bitmap = bitmap;
                    sni_refmut.font_color = font_color;
                    sni_refmut.bg_color = bg_color;
                    sni_refmut
                        .emit_new_icon(sni_iface_ref.signal_emitter())
                        .await?;
                    sni_refmut
                        .icon_pixmap_changed(sni_iface_ref.signal_emitter())
                        .await?;
                }
            }
            AppInfo::TimerSkipped => {
                // 次のタイマーの状態へ変更
                // フェイズの切り替え
                // フェイズの切り替え
                match &current_phase {
                    Phase::Working => match &current_app_config.long_break_pos {
                        LongBreakPos::EveryRap(every) => {
                            if rap_counter % every == 0 {
                                current_phase = Phase::LongBreak;
                            } else {
                                current_phase = Phase::ShortBreak;
                            }
                        }
                        LongBreakPos::SpecificRap(specific_raps) => {
                            if specific_raps.contains(&rap_counter) {
                                current_phase = Phase::LongBreak;
                            } else {
                                current_phase = Phase::ShortBreak;
                            }
                        }
                    },
                    Phase::LongBreak | Phase::ShortBreak => {
                        current_phase = Phase::Working;
                    }
                };

                let next_time = match &current_phase {
                    Phase::Working => current_app_config.work_time,
                    Phase::ShortBreak => current_app_config.short_break_time,
                    Phase::LongBreak => current_app_config.long_break_time,
                };
                {
                    let dbus_menu_iface_ref = conn
                        .object_server()
                        .interface::<_, DbusMenu>(DBUS_MENU_OBJECT_PATH)
                        .await?;
                    let mut dbus_menu_refmut = dbus_menu_iface_ref.get_mut().await;

                    dbus_menu_refmut.set_time(next_time);
                    let mut update_properties = Vec::new();

                    update_properties.append(&mut (dbus_menu_refmut.set_phase(current_phase)?));
                    if let Phase::Working = &current_phase {
                        rap_counter += 1;
                        update_properties.append(&mut (dbus_menu_refmut.set_rap(rap_counter)?));
                    }

                    dbus_menu_refmut
                        .emit_items_properties_updated(
                            dbus_menu_iface_ref.signal_emitter(),
                            update_properties,
                        )
                        .await?;
                }

                // アイコン画像の変更
                let bitmap = DoubleDigit::from(next_time).bitmap_flatten();
                let (font_color, bg_color) = match &current_phase {
                    Phase::Working => (
                        current_app_config.work_font_color,
                        current_app_config.work_bg_color,
                    ),
                    Phase::ShortBreak => (
                        current_app_config.short_break_font_color,
                        current_app_config.short_break_bg_color,
                    ),
                    Phase::LongBreak => (
                        current_app_config.long_break_font_color,
                        current_app_config.long_break_bg_color,
                    ),
                };
                {
                    let sni_iface_ref = conn
                        .object_server()
                        .interface::<_, StatusNotifierItem>(SNI_OBJECT_PATH)
                        .await?;
                    let mut sni_refmut = sni_iface_ref.get_mut().await;
                    sni_refmut.bitmap = bitmap;
                    sni_refmut.font_color = font_color;
                    sni_refmut.bg_color = bg_color;
                    sni_refmut
                        .emit_new_icon(sni_iface_ref.signal_emitter())
                        .await?;
                    sni_refmut
                        .icon_pixmap_changed(sni_iface_ref.signal_emitter())
                        .await?;
                }
            }
            AppInfo::ReceivedConfigure(configure) => {
                current_app_config = configure;

                // DbusMenuに対する設定の変更(リセット)
                let next_time = current_app_config.work_time;
                current_phase = Phase::Working;
                rap_counter = 1;

                {
                    let dbus_menu_iface_ref = conn
                        .object_server()
                        .interface::<_, DbusMenu>(DBUS_MENU_OBJECT_PATH)
                        .await?;
                    let mut dbus_menu_refmut = dbus_menu_iface_ref.get_mut().await;

                    dbus_menu_refmut.set_time(next_time);
                    let mut update_properties = Vec::new();

                    update_properties.append(&mut (dbus_menu_refmut.set_phase(current_phase)?));
                    update_properties.append(&mut (dbus_menu_refmut.set_rap(rap_counter)?));

                    dbus_menu_refmut
                        .emit_items_properties_updated(
                            dbus_menu_iface_ref.signal_emitter(),
                            update_properties,
                        )
                        .await?;
                }

                // SNIに対する設定の変更
                let bitmap = DoubleDigit::from(next_time).bitmap_flatten();
                let (font_color, bg_color) = (
                    current_app_config.work_font_color,
                    current_app_config.work_bg_color,
                );
                {
                    let sni_iface_ref = conn
                        .object_server()
                        .interface::<_, StatusNotifierItem>(SNI_OBJECT_PATH)
                        .await?;
                    let mut sni_refmut = sni_iface_ref.get_mut().await;
                    sni_refmut.font_color = font_color;
                    sni_refmut.bg_color = bg_color;
                    sni_refmut.bitmap = bitmap;

                    sni_refmut
                        .emit_new_icon(sni_iface_ref.signal_emitter())
                        .await?;
                    sni_refmut
                        .icon_pixmap_changed(sni_iface_ref.signal_emitter())
                        .await?;
                }

                // 音声に対する設定の変更
                if let StringOpt::Value(sound_file) = current_app_config.sound_break_to_work.clone()
                {
                    audio_cmd_sender
                        .try_send(AudioCommand::ChangeFile {
                            file: sound_file,
                            sound_file_type: SoundFileType::Break2Work,
                        })
                        .map_err(|e| {
                            try_send_error_to_app_error(
                                e,
                                "run_main_loop: audio_cmd_sender".to_owned(),
                            )
                        })?;
                }
                if let StringOpt::Value(sound_file) = current_app_config.sound_work_to_break.clone()
                {
                    audio_cmd_sender
                        .try_send(AudioCommand::ChangeFile {
                            file: sound_file,
                            sound_file_type: SoundFileType::Work2Break,
                        })
                        .map_err(|e| {
                            try_send_error_to_app_error(
                                e,
                                "run_main_loop: audio_cmd_sender".to_owned(),
                            )
                        })?;
                }
                audio_cmd_sender
                    .try_send(AudioCommand::ChangeVolume(current_app_config.sound_volume))
                    .map_err(|e| {
                        try_send_error_to_app_error(e, "run_main_loop: audio_cmd_sender".to_owned())
                    })?;
            }
            AppInfo::ReceivedExternalStart => {
                // リセット&スタート
                current_phase = Phase::Working;
                let next_time = current_app_config.work_time;
                rap_counter = 1;

                {
                    let dbus_menu_iface_ref = conn
                        .object_server()
                        .interface::<_, DbusMenu>(DBUS_MENU_OBJECT_PATH)
                        .await?;
                    let mut dbus_menu_refmut = dbus_menu_iface_ref.get_mut().await;

                    dbus_menu_refmut.set_time(next_time);
                    let mut update_properties = Vec::new();

                    update_properties.append(&mut (dbus_menu_refmut.set_phase(current_phase)?));
                    update_properties.append(&mut (dbus_menu_refmut.set_rap(rap_counter)?));

                    dbus_menu_refmut.pending()?;
                    update_properties.append(&mut (dbus_menu_refmut.start()?));

                    dbus_menu_refmut
                        .emit_items_properties_updated(
                            dbus_menu_iface_ref.signal_emitter(),
                            update_properties,
                        )
                        .await?;
                }
            }
            AppInfo::TimerTerminated => {
                return Ok(());
            }
            _ => {}
        }
    }

    Ok(())
}
