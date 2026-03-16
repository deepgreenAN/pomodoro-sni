pub mod menu_tree;

use crate::timer::TimerCommand;
use crate::{AppError, TimerMinute, menu_properties, try_send_error_to_app_error};
use menu_tree::{MenuNode, MenuProperties, MenuTree};

use std::collections::HashMap;

use async_channel::Sender;
use log::{debug, error};
use zbus::zvariant::OwnedValue;
use zbus::{Error as ZbusError, interface, object_server::SignalEmitter};

const ROOT_ID: i32 = 0;
const STATUS_ID: i32 = 11;
const PHASE_ID: i32 = 12;
const RAP_ID: i32 = 13;
const FIRST_SEPARATOR_ID: i32 = 2;
const START_ID: i32 = 21;
const RESUME_ID: i32 = 22;
const PAUSE_ID: i32 = 23;
const SKIP_ID: i32 = 24;
const RESET_ID: i32 = 25;
const SECOND_SEPARATOR_ID: i32 = 3;
const SETTINGS_ID: i32 = 31;
const THIRD_SEPARATOR_ID: i32 = 4;
const EXIT_ID: i32 = 41;

fn default_menu_tree() -> MenuTree {
    let mut menu_tree = MenuTree::new();

    menu_tree.insert_node(MenuNode {
        id: ROOT_ID,
        properties: menu_properties!("children-display" => "submenu"),
        children: vec![
            STATUS_ID,
            PHASE_ID,
            RAP_ID,
            FIRST_SEPARATOR_ID,
            START_ID,
            RESUME_ID,
            PAUSE_ID,
            SKIP_ID,
            RESET_ID,
            SECOND_SEPARATOR_ID,
            SETTINGS_ID,
            THIRD_SEPARATOR_ID,
            EXIT_ID,
        ],
    });

    menu_tree.insert_node(MenuNode {
        id: STATUS_ID,
        properties: menu_properties!(
            "label" => "Status: Pending",
            "enabled" => false
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: RAP_ID,
        properties: menu_properties!(
            "label" => "Rap: 1",
            "enabled" => false
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: PHASE_ID,
        properties: menu_properties!(
            "label" => "Phase: Working",
            "enabled" => false
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: FIRST_SEPARATOR_ID,
        properties: menu_properties!(
            "type" => "separator",
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: START_ID,
        properties: menu_properties!(
            "label" => "Start",
            "icon-name" => "media-playback-start",
            "visible" => true
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: RESUME_ID,
        properties: menu_properties!(
            "label" => "Resume",
            "icon-name" => "media-playback-start",
            "visible" => false
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: PAUSE_ID,
        properties: menu_properties!(
            "label" => "Pause",
            "icon-name" => "media-playback-pause",
            "visible" => false
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: SKIP_ID,
        properties: menu_properties!(
            "label" => "Skip",
            "icon-name" => "media-skip-forward",
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: RESET_ID,
        properties: menu_properties!(
            "label" => "Reset",
            "icon-name" => "edit-undo"
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: SECOND_SEPARATOR_ID,
        properties: menu_properties!(
            "type" => "separator",
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: SETTINGS_ID,
        properties: menu_properties!(
            "label" => "Settings",
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: THIRD_SEPARATOR_ID,
        properties: menu_properties!(
            "type" => "separator",
        ),
        children: Vec::new(),
    });

    menu_tree.insert_node(MenuNode {
        id: EXIT_ID,
        properties: menu_properties!(
            "label" => "Exit",
            "icon-name" => "process-stop",
        ),
        children: Vec::new(),
    });

    menu_tree
}

/// タイマーコントローラーの状態
#[derive(Debug, Clone, Copy)]
enum TimerControllerStatus {
    /// スタート前，リセットした状態，またはダイアログでキャンセルを選び停止した状態．
    Pending,
    /// タイマーが動いている状態
    Running,
    /// タイマーがポーズしている状態
    Paused,
}

/// 実質的なタイマーのGUI．タイマーのコントローラーも内包する．
pub struct DbusMenu {
    menu_tree: MenuTree,
    revision_counter: u32,
    timer_cmd_sender: Sender<TimerCommand>,
    timer_controller_status: TimerControllerStatus,
    time: TimerMinute,
}

pub type UpdateProperties = Vec<(i32, HashMap<String, OwnedValue>)>;

impl DbusMenu {
    pub fn new(timer_cmd_sender: Sender<TimerCommand>, time: TimerMinute) -> Self {
        Self {
            menu_tree: default_menu_tree(),
            revision_counter: 1,
            timer_cmd_sender,
            timer_controller_status: TimerControllerStatus::Pending,
            time,
        }
    }
    pub fn set_time(&mut self, time: TimerMinute) {
        self.time = time;
    }
    pub async fn emit_items_properties_updated<'a>(
        &mut self,
        emitter: &SignalEmitter<'a>,
        update_properties: Vec<(i32, HashMap<String, OwnedValue>)>,
    ) -> Result<(), AppError> {
        // プロパティ変更の通知&レイアウト変更の通知(実装依存のため)
        self.revision_counter += 1;
        emitter
            .items_properties_updated(update_properties, Vec::new())
            .await?;

        emitter.layout_updated(self.revision_counter, 0).await?;

        Ok(())
    }
    pub fn set_rap(&mut self, rap: u32) -> Result<UpdateProperties, AppError> {
        let mut update_properties = UpdateProperties::new();

        let rap_text = format!("Rap: {rap}");

        self.menu_tree
            .get_mut(RAP_ID)
            .unwrap()
            .properties
            .insert_value("label".to_owned(), rap_text.clone());
        update_properties.push((
            RAP_ID,
            menu_properties!("label" => rap_text).inner().clone(),
        ));

        Ok(update_properties)
    }
    pub fn set_phase<T: Into<String>>(&mut self, phase: T) -> Result<UpdateProperties, AppError> {
        let mut update_properties = UpdateProperties::new();

        let phase_text = format!("Phase: {}", Into::<String>::into(phase));

        self.menu_tree
            .get_mut(PHASE_ID)
            .unwrap()
            .properties
            .insert_value("label".to_owned(), phase_text.clone());
        update_properties.push((
            PHASE_ID,
            menu_properties!("label" => phase_text).inner().clone(),
        ));

        Ok(update_properties)
    }

    pub fn start(&mut self) -> Result<UpdateProperties, AppError> {
        if let TimerControllerStatus::Pending = self.timer_controller_status {
            self.timer_controller_status = TimerControllerStatus::Running;

            let mut update_properties = UpdateProperties::new();

            // ステータスの変更
            self.menu_tree
                .get_mut(STATUS_ID)
                .unwrap()
                .properties
                .insert_value("label".to_owned(), "Status: Running".to_owned());
            update_properties.push((
                STATUS_ID,
                menu_properties!("label" => "Status: Running")
                    .inner()
                    .clone(),
            ));

            // Startの不可視化
            self.menu_tree
                .get_mut(START_ID)
                .unwrap()
                .properties
                .insert_value("visible".to_owned(), false);
            update_properties.push((
                START_ID,
                menu_properties!("visible" => false).inner().clone(),
            ));

            // Pauseの可視化
            self.menu_tree
                .get_mut(PAUSE_ID)
                .unwrap()
                .properties
                .insert_value("visible".to_owned(), true);
            update_properties.push((
                PAUSE_ID,
                menu_properties!("visible" => true).inner().clone(),
            ));

            // Resumeの不可視化
            self.menu_tree
                .get_mut(RESUME_ID)
                .unwrap()
                .properties
                .insert_value("visible".to_owned(), false);
            update_properties.push((
                PAUSE_ID,
                menu_properties!("visible" => false).inner().clone(),
            ));

            // タイマーへのコマンド送信
            self.timer_cmd_sender
                .try_send(TimerCommand::Start(self.time))
                .map_err(|e| {
                    try_send_error_to_app_error(e, "DbusMenu: timer_cmd_sender".to_owned())
                })?;

            Ok(update_properties)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn pause(&mut self) -> Result<UpdateProperties, AppError> {
        if let TimerControllerStatus::Running = self.timer_controller_status {
            self.timer_controller_status = TimerControllerStatus::Paused;

            let mut update_properties = UpdateProperties::new();

            // ステータスの変更
            self.menu_tree
                .get_mut(STATUS_ID)
                .unwrap()
                .properties
                .insert_value("label".to_owned(), "Status: Paused".to_owned());
            update_properties.push((
                STATUS_ID,
                menu_properties!("label" => "Status: Paused")
                    .inner()
                    .clone(),
            ));

            // Startの不可視化
            self.menu_tree
                .get_mut(START_ID)
                .unwrap()
                .properties
                .insert_value("visible".to_owned(), false);
            update_properties.push((
                START_ID,
                menu_properties!("visible" => false).inner().clone(),
            ));

            // Pauseの不可視化
            self.menu_tree
                .get_mut(PAUSE_ID)
                .unwrap()
                .properties
                .insert_value("visible".to_owned(), false);
            update_properties.push((
                PAUSE_ID,
                menu_properties!("visible" => false).inner().clone(),
            ));

            // Resumeの可視化
            self.menu_tree
                .get_mut(RESUME_ID)
                .unwrap()
                .properties
                .insert_value("visible".to_owned(), true);
            update_properties.push((
                PAUSE_ID,
                menu_properties!("visible" => true).inner().clone(),
            ));

            // タイマーへのコマンド送信
            self.timer_cmd_sender
                .try_send(TimerCommand::Pause)
                .map_err(|e| {
                    try_send_error_to_app_error(e, "DbusMenu: timer_cmd_sender".to_owned())
                })?;

            Ok(update_properties)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn resume(&mut self) -> Result<UpdateProperties, AppError> {
        if let TimerControllerStatus::Paused = self.timer_controller_status {
            self.timer_controller_status = TimerControllerStatus::Running;

            let mut update_properties = UpdateProperties::new();

            // ステータスの変更
            self.menu_tree
                .get_mut(STATUS_ID)
                .unwrap()
                .properties
                .insert_value("label".to_owned(), "Status: Running".to_owned());
            update_properties.push((
                STATUS_ID,
                menu_properties!("label" => "Status: Running")
                    .inner()
                    .clone(),
            ));

            // Startの不可視化
            self.menu_tree
                .get_mut(START_ID)
                .unwrap()
                .properties
                .insert_value("visible".to_owned(), false);
            update_properties.push((
                START_ID,
                menu_properties!("visible" => false).inner().clone(),
            ));

            // Pauseの可視化
            self.menu_tree
                .get_mut(PAUSE_ID)
                .unwrap()
                .properties
                .insert_value("visible".to_owned(), true);
            update_properties.push((
                PAUSE_ID,
                menu_properties!("visible" => true).inner().clone(),
            ));

            // Resumeの不可視化
            self.menu_tree
                .get_mut(RESUME_ID)
                .unwrap()
                .properties
                .insert_value("visible".to_owned(), false);
            update_properties.push((
                PAUSE_ID,
                menu_properties!("visible" => false).inner().clone(),
            ));

            // タイマーへのコマンド送信
            self.timer_cmd_sender
                .try_send(TimerCommand::Resume)
                .map_err(|e| {
                    try_send_error_to_app_error(e, "DbusMenu: timer_cmd_sender".to_owned())
                })?;

            Ok(update_properties)
        } else {
            Ok(Vec::new())
        }
    }

    /// 状態をPendingにする．コマンドは送信しない．
    pub fn pending(&mut self) -> Result<UpdateProperties, AppError> {
        self.timer_controller_status = TimerControllerStatus::Pending;

        let mut update_properties = UpdateProperties::new();

        // ステータスの変更
        self.menu_tree
            .get_mut(STATUS_ID)
            .unwrap()
            .properties
            .insert_value("label".to_owned(), "Status: Pending".to_owned());
        update_properties.push((
            STATUS_ID,
            menu_properties!("label" => "Status: Pending")
                .inner()
                .clone(),
        ));

        // Startの可視化
        self.menu_tree
            .get_mut(START_ID)
            .unwrap()
            .properties
            .insert_value("visible".to_owned(), true);
        update_properties.push((
            PAUSE_ID,
            menu_properties!("visible" => true).inner().clone(),
        ));

        // Pauseの不可視化
        self.menu_tree
            .get_mut(PAUSE_ID)
            .unwrap()
            .properties
            .insert_value("visible".to_owned(), false);
        update_properties.push((
            PAUSE_ID,
            menu_properties!("visible" => false).inner().clone(),
        ));

        // Resumeの不可視化
        self.menu_tree
            .get_mut(RESUME_ID)
            .unwrap()
            .properties
            .insert_value("visible".to_owned(), false);
        update_properties.push((
            PAUSE_ID,
            menu_properties!("visible" => false).inner().clone(),
        ));

        Ok(update_properties)
    }

    pub fn reset(&mut self) -> Result<UpdateProperties, AppError> {
        let update_properties = self.pending()?;

        // タイマーへのコマンド送信
        self.timer_cmd_sender
            .try_send(TimerCommand::Reset)
            .map_err(|e| try_send_error_to_app_error(e, "DbusMenu: timer_cmd_sender".to_owned()))?;

        Ok(update_properties)
    }
    pub fn skip(&mut self) -> Result<UpdateProperties, AppError> {
        let update_properties = self.pending()?;

        // タイマーへのコマンド送信
        self.timer_cmd_sender
            .try_send(TimerCommand::Skip)
            .map_err(|e| try_send_error_to_app_error(e, "DbusMenu: timer_cmd_sender".to_owned()))?;

        Ok(update_properties)
    }
    pub fn spawn_settings(&mut self) -> Result<(), AppError> {
        use crate::terminal::get_available_terminal_cmd;

        let current_exe =
            std::env::current_exe().map_err(|e| AppError::CustomError(e.to_string()))?;
        let _child = get_available_terminal_cmd()?
            .arg(current_exe)
            .arg("configure")
            .spawn()?;

        Ok(())
    }
    pub fn exit(&self) -> Result<(), AppError> {
        self.timer_cmd_sender
            .try_send(TimerCommand::Terminate)
            .map_err(|e| try_send_error_to_app_error(e, "DbusMenu: timer_cmd_sender".to_owned()))
    }
}

#[interface(name = "com.canonical.dbusmenu")]
impl DbusMenu {
    #[zbus(property)]
    pub async fn version(&self) -> u32 {
        2
    }
    #[zbus(property)]
    pub async fn status(&self) -> String {
        "normal".to_string()
    }

    pub async fn get_layout(
        &self,
        parent_id: i32,
        recursion_depth: i32,
        property_names: Vec<String>,
    ) -> (u32, (i32, HashMap<String, OwnedValue>, Vec<OwnedValue>)) {
        (
            self.revision_counter,
            self.menu_tree
                .to_tree(parent_id, recursion_depth, &property_names),
        )
    }

    pub async fn get_group_properties(
        &self,
        ids: Vec<i32>,
        property_names: Vec<String>,
    ) -> Vec<(i32, HashMap<String, OwnedValue>)> {
        self.menu_tree.get_group_properties(&ids, &property_names)
    }

    pub async fn get_property(&self, id: i32, name: String) -> OwnedValue {
        self.menu_tree.get_property(id, &name)
    }

    pub async fn event(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        id: i32,
        event_id: String,
        data: OwnedValue,
        timestamp: u32,
    ) {
        debug!("id: {id}, event_id: {event_id}, data: {data:?}, timestamp: {timestamp}");

        match id {
            ROOT_ID => {}
            START_ID => match self.start() {
                Ok(update_properties) => {
                    if !update_properties.is_empty()
                        && let Err(e) = self
                            .emit_items_properties_updated(&emitter, update_properties)
                            .await
                    {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            PAUSE_ID => match self.pause() {
                Ok(update_properties) => {
                    if !update_properties.is_empty()
                        && let Err(e) = self
                            .emit_items_properties_updated(&emitter, update_properties)
                            .await
                    {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            RESUME_ID => match self.resume() {
                Ok(update_properties) => {
                    if !update_properties.is_empty()
                        && let Err(e) = self
                            .emit_items_properties_updated(&emitter, update_properties)
                            .await
                    {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            RESET_ID => match self.reset() {
                Ok(update_properties) => {
                    if !update_properties.is_empty()
                        && let Err(e) = self
                            .emit_items_properties_updated(&emitter, update_properties)
                            .await
                    {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            SKIP_ID => match self.skip() {
                Ok(update_properties) => {
                    if !update_properties.is_empty()
                        && let Err(e) = self
                            .emit_items_properties_updated(&emitter, update_properties)
                            .await
                    {
                        error!("{e}");
                    }
                }
                Err(e) => {
                    error!("{e}");
                }
            },
            SETTINGS_ID => {
                if let Err(e) = self.spawn_settings() {
                    error!("{e}");
                }
            }
            EXIT_ID => {
                if let Err(e) = self.exit() {
                    error!("{e}");
                }
            }
            _ => {}
        }
    }

    pub async fn about_to_show(&self, _id: i32) -> bool {
        true
    }

    #[zbus(signal)]
    pub async fn items_properties_updated(
        emitter: &SignalEmitter<'_>,
        update_props: Vec<(i32, HashMap<String, OwnedValue>)>,
        removed_props: Vec<(i32, Vec<String>)>,
    ) -> Result<(), ZbusError>;

    #[zbus(signal)]
    pub async fn layout_updated(
        emitter: &SignalEmitter<'_>,
        revision: u32,
        id: i32,
    ) -> Result<(), ZbusError>;

    #[zbus(signal)]
    pub async fn item_activation_requested(
        emitter: &SignalEmitter<'_>,
        id: i32,
        timestamp: u32,
    ) -> Result<(), ZbusError>;
}
