//! 退出确认状态机模块。
//!
//! 管理「双击 Ctrl+C 退出」的状态转换与提示渲染。

use std::time::{Duration, Instant};

enum State {
    Idle,
    ConfirmOnce(Instant),
    Confirmed,
}

pub struct ExitHandler {
    state: State,
    timeout: Duration,
}

impl ExitHandler {
    pub fn new(timeout: Duration) -> Self {
        Self {
            state: State::Idle,
            timeout,
        }
    }

    pub fn press_ctrl_c(&mut self) {
        let now = Instant::now();
        self.state = match self.state {
            State::Idle => State::ConfirmOnce(now),
            State::ConfirmOnce(first) if now.duration_since(first) < self.timeout => {
                State::Confirmed
            }
            State::ConfirmOnce(_) => State::ConfirmOnce(now),
            State::Confirmed => State::Confirmed,
        };
    }

    /// 重置退出确认（按非退出按键时调用）。
    pub fn reset(&mut self) {
        self.state = State::Idle;
    }

    /// 检查 ConfirmOnce 是否超时，超时则转回 Idle。
    pub fn tick(&mut self) {
        if let State::ConfirmOnce(first) = self.state
            && first.elapsed() >= self.timeout
        {
            self.state = State::Idle;
        }
    }

    /// 退出是否已确认（按了两次 Ctrl+C）。
    pub fn should_exit(&self) -> bool {
        matches!(self.state, State::Confirmed)
    }

    /// 返回事件轮询的超时时间。
    ///
    /// - Idle / Confirmed：100ms 轮询
    /// - ConfirmOnce：剩余等待时间，以便及时处理第二次 Ctrl+C
    pub fn poll_timeout(&self) -> Duration {
        match self.state {
            State::Idle | State::Confirmed => Duration::from_millis(100),
            State::ConfirmOnce(first) => self.timeout.saturating_sub(first.elapsed()),
        }
    }

    pub fn hint_text(&self) -> Option<&'static str> {
        if matches!(self.state, State::ConfirmOnce(_)) {
            Some("再次按下 Ctrl+C 以退出")
        } else {
            None
        }
    }
}
