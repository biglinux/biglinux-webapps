//! Helper to run a blocking job on a worker thread and resume the UI thread
//! with its result, *without* polling. Built on top of
//! [`glib::MainContext::spawn_local`] + a one-shot `glib::Sender` channel so
//! the main loop only wakes when the worker actually has data.

use std::cell::RefCell;
use std::rc::Rc;
use std::thread;

use gtk4::glib;

/// Run `job` on a background thread and invoke `on_result` on the UI thread
/// with the value the job returned. `on_result` always fires exactly once,
/// even if the worker thread is killed before sending — in that case the
/// callback receives `None`.
pub fn run_with_result<R, Job, OnResult>(job: Job, on_result: OnResult)
where
    R: Send + 'static,
    Job: FnOnce() -> R + Send + 'static,
    OnResult: FnOnce(R) + 'static,
{
    run_with_result_or_default(job, move |maybe_result| {
        if let Some(value) = maybe_result {
            on_result(value);
        } else {
            log::warn!("Background job did not produce a result (worker exited early)");
        }
    });
}

/// Like [`run_with_result`] but always invokes the callback, with `Some` when
/// the worker produced a value and `None` when it was dropped (panic, OS kill).
/// Use this when the UI must clear a "loading" state regardless of outcome.
pub fn run_with_result_or_default<R, Job, OnResult>(job: Job, on_result: OnResult)
where
    R: Send + 'static,
    Job: FnOnce() -> R + Send + 'static,
    OnResult: FnOnce(Option<R>) + 'static,
{
    let (sender, receiver) = async_channel::bounded::<R>(1);
    thread::spawn(move || {
        let value = job();
        // best-effort send: receiver may have been dropped if the dialog closed
        let _ = sender.send_blocking(value);
    });

    let on_result = Rc::new(RefCell::new(Some(on_result)));
    glib::MainContext::default().spawn_local(async move {
        let value = receiver.recv().await.ok();
        if let Some(callback) = on_result.borrow_mut().take() {
            callback(value);
        }
    });
}
