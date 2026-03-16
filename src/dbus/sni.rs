use crate::Rgb;
use crate::error::AppError;

use log::debug;
use zbus::zvariant::{ObjectPath, OwnedObjectPath};
use zbus::{Error as ZbusError, interface, object_server::SignalEmitter, proxy};

pub struct StatusNotifierItem {
    pub bitmap: [bool; 256],
    pub font_color: Rgb,
    pub bg_color: Rgb,
}

impl StatusNotifierItem {
    pub fn new(bitmap: [bool; 256], font_color: Rgb, bg_color: Rgb) -> Self {
        Self {
            bitmap,
            font_color,
            bg_color,
        }
    }
    pub async fn emit_new_icon<'a>(&self, emitter: &SignalEmitter<'a>) -> Result<(), AppError> {
        Self::new_icon(emitter).await?;
        Ok(())
    }
}

#[interface(name = "org.kde.StatusNotifierItem")]
impl StatusNotifierItem {
    #[zbus(property)]
    pub async fn category(&self) -> String {
        "ApplicationStatus".to_owned()
    }

    #[zbus(property)]
    pub async fn id(&self) -> String {
        "pomodoro-sni".to_owned()
    }

    #[zbus(property)]
    pub async fn title(&self) -> String {
        "pomodoro timer which implements SNI.".to_owned()
    }

    #[zbus(property)]
    pub async fn status(&self) -> String {
        "Active".to_owned()
    }

    #[zbus(property)]
    pub async fn window_id(&self) -> u32 {
        0
    }

    #[zbus(property)]
    pub async fn icon_name(&self) -> String {
        String::new() // 空白文字列
    }

    #[zbus(property)]
    pub async fn icon_pixmap(&self) -> Vec<(i32, i32, Vec<u8>)> {
        let font_color = self.font_color;
        let bg_color = self.bg_color;

        let img = self
            .bitmap
            .into_iter()
            .flat_map(|bit| {
                let color = match bit {
                    true => font_color,
                    false => bg_color,
                };
                [255_u8, color.r, color.g, color.b] // [a, r, g, b]
            })
            .collect::<Vec<_>>();

        vec![(16, 16, img)]
    }

    #[zbus(property)]
    pub async fn overlay_icon_name(&self) -> String {
        String::new()
    }

    #[zbus(property)]
    pub async fn overlay_icon_pixmap(&self) -> Vec<(i32, i32, Vec<u8>)> {
        Vec::new()
    }

    #[zbus(property)]
    pub async fn attention_icon_name(&self) -> String {
        String::new()
    }

    #[zbus(property)]
    pub async fn attention_icon_pixmap(&self) -> Vec<(i32, i32, Vec<u8>)> {
        Vec::new()
    }

    #[zbus(property)]
    pub async fn attention_movie_name(&self) -> String {
        String::new()
    }

    #[zbus(property)]
    pub async fn tool_tip(&self) -> (String, Vec<(i32, i32, Vec<u8>)>, String, String) {
        let icon_name = String::new();
        let icon_data = Vec::new();
        let title = "my sni tooltip".to_owned();
        let desc = "this is tooltip description".to_owned();

        (icon_name, icon_data, title, desc)
    }

    #[zbus(property)]
    pub async fn item_is_menu(&self) -> bool {
        true
    }

    #[zbus(property)]
    pub async fn menu(&self) -> OwnedObjectPath {
        ObjectPath::try_from("/DbusMenu").unwrap().into()
    }

    /// dbusmenuを使う場合は無視される？
    pub async fn context_menu(&self, _x: i32, _y: i32) {
        debug!("StatusNotifierItem::context_menu called");
    }

    /// 多くの場合左クリックで呼ばれる．
    pub async fn activate(&self, _x: i32, _y: i32) {
        debug!("StatusNotifierItem::activate called");
    }

    pub async fn secondary_active(&self, _x: i32, _y: i32) {}

    pub async fn scroll(&self, _delta: i32, _orientation: String) {}

    #[zbus(signal)]
    async fn new_title(emitter: &SignalEmitter<'_>) -> Result<(), ZbusError>;

    #[zbus(signal)]
    async fn new_icon(emitter: &SignalEmitter<'_>) -> Result<(), ZbusError>;

    #[zbus(signal)]
    async fn new_attention_icon(emitter: &SignalEmitter<'_>) -> Result<(), ZbusError>;

    #[zbus(signal)]
    async fn new_overlay_icon(emitter: &SignalEmitter<'_>) -> Result<(), ZbusError>;

    #[zbus(signal)]
    async fn new_tooltip(emitter: &SignalEmitter<'_>) -> Result<(), ZbusError>;

    #[zbus(signal)]
    async fn new_status(emitter: &SignalEmitter<'_>, status: String) -> Result<(), ZbusError>;
}

#[proxy(
    default_service = "org.kde.StatusNotifierWatcher",
    default_path = "/StatusNotifierWatcher",
    interface = "org.kde.StatusNotifierWatcher"
)]
pub trait StatusNotifierWatcher {
    fn register_status_notifier_item(&self, service: String) -> Result<(), ZbusError>;
    fn register_status_notifier_host(&self, service: String) -> Result<(), ZbusError>;

    #[zbus(property)]
    fn registered_status_notifier_items(&self) -> Result<Vec<String>, ZbusError>;

    #[zbus(property)]
    fn is_status_notifier_host_registered(&self) -> Result<bool, ZbusError>;

    #[zbus(property)]
    fn protocol_version(&self) -> Result<i32, ZbusError>;

    #[zbus(signal)]
    fn status_notifier_item_registered(&self, service: String) -> zbus::Result<()>;

    #[zbus(signal)]
    fn status_notifier_item_unregistered(&self, service: String) -> zbus::Result<()>;

    #[zbus(signal)]
    fn status_notifier_host_registered(&self) -> zbus::Result<()>;
}
