use crate::{AppError, SoundFilePath, SoundVolume};

use async_channel::Receiver;
use rodio::{Decoder, DeviceSinkBuilder, Player};

use std::fs::File;
use std::io::{BufReader, Cursor};

#[derive(Debug, Clone, Copy)]
pub enum SoundFileType {
    Work2Break,
    Break2Work,
}

#[derive(Debug, Clone)]
pub enum AudioCommand {
    Play {
        sound_file_type: SoundFileType,
    },
    ChangeFile {
        file: SoundFilePath,
        sound_file_type: SoundFileType,
    },
    ChangeVolume(SoundVolume),
    Terminate,
}

/// オーディオループの開始．
pub fn run_audio_loop(cmd_receiver: Receiver<AudioCommand>) -> Result<(), AppError> {
    let mut sink_handle = DeviceSinkBuilder::open_default_sink()?;
    sink_handle.log_on_drop(false);
    let player = Player::connect_new(sink_handle.mixer());

    let default_work2break_bytes = include_bytes!("../assets/sounds/work2break.wav");
    let default_break2work_bytes = include_bytes!("../assets/sounds/break2work.wav");

    let mut current_work2break_path = Option::<SoundFilePath>::None;
    let mut current_break2work_path = Option::<SoundFilePath>::None;

    while let Ok(cmd) = cmd_receiver.recv_blocking() {
        match cmd {
            AudioCommand::Play { sound_file_type } => {
                match sound_file_type {
                    SoundFileType::Break2Work => match current_break2work_path.as_ref() {
                        Some(sound_file_path) => {
                            let source = Decoder::try_from(BufReader::new(File::open(
                                sound_file_path.path(),
                            )?))?;
                            player.append(source);
                        }
                        None => {
                            let source = Decoder::try_from(Cursor::new(default_break2work_bytes))?;
                            player.append(source);
                        }
                    },
                    SoundFileType::Work2Break => match current_work2break_path.as_ref() {
                        Some(sound_file_path) => {
                            let source = Decoder::try_from(BufReader::new(File::open(
                                sound_file_path.path(),
                            )?))?;
                            player.append(source);
                        }
                        None => {
                            let source = Decoder::try_from(Cursor::new(default_work2break_bytes))?;
                            player.append(source);
                        }
                    },
                }

                player.sleep_until_end();
            }
            AudioCommand::ChangeFile {
                file,
                sound_file_type,
            } => match sound_file_type {
                SoundFileType::Break2Work => {
                    current_break2work_path = Some(file);
                }
                SoundFileType::Work2Break => {
                    current_work2break_path = Some(file);
                }
            },
            AudioCommand::ChangeVolume(volume) => {
                player.set_volume(volume.as_float());
            }
            AudioCommand::Terminate => {
                break;
            }
        }
    }

    Ok(())
}
