use crate::try_send_error_to_app_error;
use crate::{AppConfig, AppInfo};

use async_channel::Sender;
use log::error;
use zbus::interface;

/// 外部操作用のインターフェース
pub struct PomodoroSniController {
    app_config: AppConfig,
    controller_sender: Sender<AppInfo>,
}

impl PomodoroSniController {
    pub fn new(app_config: AppConfig, sender: Sender<AppInfo>) -> Self {
        Self {
            app_config,
            controller_sender: sender,
        }
    }
}

#[interface(
    name = "org.zbus.PomodoroSniController",
    proxy(
        default_service = "org.zbus.PomodoroSni",
        default_path = "/PomodoroSniController"
    )
)]
impl PomodoroSniController {
    /// 現在のconfigを取得する．
    pub fn get_config(&self) -> AppConfig {
        self.app_config.clone()
    }

    /// configをdbusサーバーに送る．
    pub fn set_config(&mut self, config: AppConfig) {
        self.app_config = config.clone();

        if let Err(e) = self
            .controller_sender
            .try_send(AppInfo::ReceivedConfigure(config))
            .map_err(|e| {
                try_send_error_to_app_error(
                    e,
                    "PomodoroSniController: controller_sender".to_owned(),
                )
            })
        {
            error!("{e}");
        }
    }
    /// 外部からのスタート
    pub fn external_start(&self) {
        if let Err(e) = self
            .controller_sender
            .try_send(AppInfo::ReceivedExternalStart)
            .map_err(|e| {
                try_send_error_to_app_error(
                    e,
                    "PomodoroSniController: controller_sender".to_owned(),
                )
            })
        {
            error!("{e}");
        }
    }
}
