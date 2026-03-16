use crate::{AppError, AppInfo, TimerMinute, try_send_error_to_app_error};

use async_channel::{Receiver, Sender, TryRecvError};
use chrono::{
    TimeDelta,
    prelude::{DateTime as ChronoDateTime, Local},
};

type DateTime = ChronoDateTime<Local>;
pub type Duration = TimeDelta;

fn get_current_datetime() -> DateTime {
    Local::now()
}

#[derive(Debug, Clone, Copy)]
enum TimerStatus {
    Running,
    Paused { pause_start_time: DateTime },
}

/// 残り時間を計算するだけのタイマー．
struct Timer {
    rest_time: Duration,
    last_calc_time: DateTime,
    timer_status: TimerStatus,
}

impl Timer {
    /// タイマーの開始
    fn start(duration: Duration) -> Self {
        Self {
            rest_time: duration,
            last_calc_time: get_current_datetime(),
            timer_status: TimerStatus::Running,
        }
    }
    fn status(&self) -> &TimerStatus {
        &self.timer_status
    }

    /// 残りの時間を計算し，返す．
    fn calc_rest_time(&mut self) -> Duration {
        match self.timer_status {
            TimerStatus::Running => {
                let current_time = get_current_datetime();
                self.rest_time -= current_time - self.last_calc_time;
                self.last_calc_time = current_time;

                self.rest_time
            }
            TimerStatus::Paused { pause_start_time } => {
                let current_time = get_current_datetime();
                if pause_start_time > self.last_calc_time {
                    self.rest_time -= pause_start_time - self.last_calc_time;
                }
                self.last_calc_time = current_time;

                self.rest_time
            }
        }
    }
    /// ポーズ
    fn pause(&mut self) {
        self.calc_rest_time();
        if let TimerStatus::Running = self.timer_status {
            self.timer_status = TimerStatus::Paused {
                pause_start_time: get_current_datetime(),
            };
        }
    }
    /// 再開
    fn resume(&mut self) {
        self.calc_rest_time();
        if let TimerStatus::Paused {
            pause_start_time: _,
        } = self.timer_status
        {
            self.timer_status = TimerStatus::Running;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum TimerCommand {
    /// タイマーを開始する
    Start(TimerMinute),
    /// タイマーを一時停止する．
    Pause,
    /// 一時停止したタイマーを再開する．
    Resume,
    /// タイマーのリセット
    Reset,
    /// タイマーのスキップ
    Skip,
    /// タイマースレッドを終了する．
    Terminate,
}

/// タイマーループ内では基本的にコマンドの処理，インフォの送信，スリープのみ行い，ブロッキングする処理は行わない．
/// info_senderは最低3つ以上のキャパシティを持つチャネルとする．
pub fn run_timer_loop(
    cmd_receiver: Receiver<TimerCommand>,
    info_sender: Sender<AppInfo>,
    internal_duration: Duration,
) -> Result<(), AppError> {
    let mut timer = Option::<Timer>::None;
    let mut seconds_counter = Option::<u32>::None;
    let mut minute_counter = Option::<TimerMinute>::None;
    let internal_duration_std = internal_duration.to_std().unwrap(); // おそらく失敗しない

    loop {
        match cmd_receiver.try_recv() {
            Ok(cmd) => match cmd {
                TimerCommand::Start(minutes) => {
                    timer = Some(Timer::start(Duration::minutes(minutes.as_u8().into())));
                    info_sender
                        .try_send(AppInfo::TimerStarted(minutes))
                        .map_err(|e| {
                            try_send_error_to_app_error(e, "timer info_sender".to_owned())
                        })?;
                    seconds_counter = Some(minutes.as_secs());
                    minute_counter = Some(minutes);
                }
                TimerCommand::Pause => {
                    if let Some(timer) = timer.as_mut() {
                        timer.pause();
                        info_sender.try_send(AppInfo::TimerPaused).map_err(|e| {
                            try_send_error_to_app_error(e, "timer info_sender".to_owned())
                        })?;
                    }
                }
                TimerCommand::Resume => {
                    if let Some(timer) = timer.as_mut() {
                        timer.resume();
                        info_sender.try_send(AppInfo::TimerResumed).map_err(|e| {
                            try_send_error_to_app_error(e, "timer info_sender".to_owned())
                        })?;
                    }
                }
                TimerCommand::Reset => {
                    timer = None;
                    seconds_counter = None;
                    minute_counter = None;

                    info_sender.try_send(AppInfo::TimerReset).map_err(|e| {
                        try_send_error_to_app_error(e, "timer info_sender".to_owned())
                    })?;
                }
                TimerCommand::Skip => {
                    timer = None;
                    seconds_counter = None;
                    minute_counter = None;

                    info_sender.try_send(AppInfo::TimerSkipped).map_err(|e| {
                        try_send_error_to_app_error(e, "timer info_sender".to_owned())
                    })?;
                }
                TimerCommand::Terminate => {
                    info_sender
                        .try_send(AppInfo::TimerTerminated)
                        .map_err(|e| {
                            try_send_error_to_app_error(e, "timer info_sender".to_owned())
                        })?;
                    break;
                }
            },
            Err(e) => {
                if let TryRecvError::Closed = e {
                    return Err(AppError::ReceiverError {
                        name: "timer cmd_receiver".to_owned(),
                    });
                }
            }
        }

        if let Some(timer) = timer.as_mut()
            && let TimerStatus::Running = timer.status()
            && let Some(seconds_counter) = seconds_counter.as_mut()
            && *seconds_counter != 0
            && let Some(minute_counter) = minute_counter.as_mut()
        {
            let rest_time = timer.calc_rest_time(); // 残り時間

            let rest_time_to_next_second = rest_time - Duration::seconds(rest_time.num_seconds()); // 次に秒カウントが下がるまでのタイマー時間

            // 次に秒カウントが下がるまでのタイマー時間が規定スリープ時間より小さい場合
            if rest_time_to_next_second <= internal_duration {
                let rest_time_to_next_second_std = rest_time_to_next_second.to_std().unwrap();
                std::thread::sleep(rest_time_to_next_second_std);
                *seconds_counter -= 1;

                info_sender
                    .try_send(AppInfo::Seconds(*seconds_counter))
                    .map_err(|e| try_send_error_to_app_error(e, "timer info_sender".to_owned()))?;
                if seconds_counter.is_multiple_of(60) {
                    minute_counter.decrement();

                    info_sender
                        .try_send(AppInfo::Minutes(*minute_counter))
                        .map_err(|e| {
                            try_send_error_to_app_error(e, "timer info_sender".to_owned())
                        })?;
                }

                if *seconds_counter == 0 {
                    info_sender.try_send(AppInfo::TimerFinished).map_err(|e| {
                        try_send_error_to_app_error(e, "timer info_sender".to_owned())
                    })?;
                }
            } else {
                std::thread::sleep(internal_duration_std);
            }
        } else {
            std::thread::sleep(internal_duration_std);
        }

        // 終了したタイマーの削除
        if let Some(0) = seconds_counter {
            timer = None;
            seconds_counter = None;
            minute_counter = None;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use super::Duration;

    fn duration_approximately_eq(x: Duration, y: Duration, span: Duration) -> bool {
        if x < y { y - x < span } else { x - y < span }
    }

    #[test]
    fn test_approximately_eq() {
        assert!(duration_approximately_eq(
            Duration::milliseconds(1000),
            Duration::milliseconds(1100),
            Duration::milliseconds(500)
        ));

        assert!(!duration_approximately_eq(
            Duration::milliseconds(1600),
            Duration::milliseconds(1000),
            Duration::milliseconds(500)
        ));
    }

    #[cfg(feature = "test-timer")]
    #[test]
    fn test_timer() {
        use super::Timer;

        let mut timer = Timer::start(Duration::seconds(8));

        std::thread::sleep(std::time::Duration::from_secs(3));
        assert!(duration_approximately_eq(
            timer.calc_rest_time(),
            Duration::seconds(5),
            Duration::milliseconds(50)
        ));

        timer.pause();
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(duration_approximately_eq(
            timer.calc_rest_time(),
            Duration::seconds(5),
            Duration::milliseconds(50)
        ));

        timer.resume();
        std::thread::sleep(std::time::Duration::from_secs(2));
        assert!(duration_approximately_eq(
            timer.calc_rest_time(),
            Duration::seconds(3),
            Duration::milliseconds(50)
        ));

        std::thread::sleep(std::time::Duration::from_secs(3));
        assert!(duration_approximately_eq(
            timer.calc_rest_time(),
            Duration::seconds(0),
            Duration::milliseconds(50)
        ));
    }
}
