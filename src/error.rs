#[allow(clippy::enum_variant_names)]
#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("SenderError: Receiver was dropped: {name}")]
    SenderError { name: String },

    #[error("ReceiverError: Sender was dropped: {name}")]
    ReceiverError { name: String },

    #[error("ChannelFullError: channel is full: {name}")]
    ChannelFullError { name: String },

    #[error("TypeError: {0}")]
    TypeError(String),

    #[error("ZbusError: {0}")]
    ZbusError(String),

    #[error("AudioError: {0}")]
    AudioError(String),

    #[error("IoError: {0}")]
    IoError(String),

    #[error("CustomError: {0}")]
    CustomError(String),

    #[error("DialoguerError: {0}")]
    DialoguerError(String),

    #[error("ArgError: {0}")]
    ArgError(String),
}

impl From<zbus::Error> for AppError {
    fn from(value: zbus::Error) -> Self {
        AppError::ZbusError(value.to_string())
    }
}

pub fn try_send_error_to_app_error<T>(
    try_send_error: async_channel::TrySendError<T>,
    sender_name: String,
) -> AppError {
    use async_channel::TrySendError;

    match try_send_error {
        TrySendError::Closed(_) => AppError::SenderError { name: sender_name },
        TrySendError::Full(_) => AppError::ChannelFullError { name: sender_name },
    }
}

impl From<rodio::DeviceSinkError> for AppError {
    fn from(value: rodio::DeviceSinkError) -> Self {
        AppError::AudioError(value.to_string())
    }
}

impl From<rodio::decoder::DecoderError> for AppError {
    fn from(value: rodio::decoder::DecoderError) -> Self {
        AppError::AudioError(value.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::IoError(value.to_string())
    }
}

impl From<dialoguer::Error> for AppError {
    fn from(value: dialoguer::Error) -> Self {
        AppError::DialoguerError(value.to_string())
    }
}

impl From<lexopt::Error> for AppError {
    fn from(value: lexopt::Error) -> Self {
        AppError::ArgError(value.to_string())
    }
}
