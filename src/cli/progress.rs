use std::{borrow::Cow, sync::LazyLock};

use indicatif::{MultiProgress, ProgressBar};
use std::time::Duration;

pub static CLI_MULTI_PROGRESS: LazyLock<MultiProgress> = LazyLock::new(|| MultiProgress::new());

pub fn new_progress(progress_bar: ProgressBar) -> ProgressBar {
    let progress_bar = CLI_MULTI_PROGRESS.add(progress_bar);
    progress_bar.enable_steady_tick(Duration::from_millis(10));
    progress_bar
}

pub fn finish_progress(progress_bar: &ProgressBar) {
    progress_bar.finish();
    CLI_MULTI_PROGRESS.remove(progress_bar);
}

pub trait ProgressBarExt {
    fn trace(&self, msg: impl Into<Cow<'static, str>>);
    fn info(&self, msg: impl Into<Cow<'static, str>>);
    fn error(&self, msg: impl Into<Cow<'static, str>>);
}

impl ProgressBarExt for ProgressBar {
    fn trace(&self, msg: impl Into<Cow<'static, str>>) {
        let msg_cow: Cow<'_, str> = msg.into();
        let msg_str: &str = msg_cow.as_ref();

        log::trace!("{}", msg_str);
        self.set_message(msg_cow);
    }

    fn info(&self, msg: impl Into<Cow<'static, str>>) {
        let msg_cow: Cow<'_, str> = msg.into();
        let msg_str: &str = msg_cow.as_ref();

        log::info!("{}", msg_str);
        self.set_message(msg_cow);
    }

    fn error(&self, msg: impl Into<Cow<'static, str>>) {
        let msg_cow: Cow<'_, str> = msg.into();
        let msg_str: &str = msg_cow.as_ref();

        log::error!("{}", msg_str);
        self.set_message(msg_cow);
    }
}
