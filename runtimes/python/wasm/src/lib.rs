//! TODO
#![allow(missing_docs, reason = "TODO")]
#![expect(clippy::print_stdout, reason = "just a demo")]

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::task::{Context, Poll};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys::Promise;
use wasm_bindgen_futures::{js_sys, JsFuture};
use web_sys::console::log_1;

#[wasm_bindgen]
extern "C" {
    // #[wasm_bindgen(js_namespace = window)]
    fn callback(s: &str) -> u32;
}

#[wasm_bindgen]
pub fn greet(name: String) -> Promise {
    let mut i = 0;
    log_1(&"in greet".into());
    wasm_bindgen_futures::future_to_promise(async move {
        log_1(&"in future 1".into());
        yield_point().await;
        log_1(&"in future 2".into());
        loop {
            let callback = callback(&format!("Hello, {name}!"));
            log_1(&format!("{i} callback returned {callback}").into());
            if callback > 0 {
                log_1(&"in future 3".into());
                break;
            }
            let time = web_time::Instant::now();
            while time.elapsed() < web_time::Duration::from_secs(1) {
                // burn cycles
            }
            i += 1;
            yield_point().await;
        }
        log_1(&"in future 4".into());
        Ok(JsValue::NULL)
    })
}

async fn yield_point() {
    let future: JsFuture =
        wasm_bindgen_futures::future_to_promise(async { Ok(JsValue::NULL) }).into();
    future.await.ok();
}

struct TimeoutFuture {
    done: Arc<AtomicBool>,
    id: i32,
}

impl TimeoutFuture {
    fn new() -> Self {
        let window = web_sys::window().unwrap();
        let done = Arc::new(AtomicBool::new(false));
        let f = {
            let done = Arc::clone(&done);
            move || done.store(true, std::sync::atomic::Ordering::SeqCst)
        };
        let id = window
            .set_timeout_with_callback(
                &Closure::once(f)
                    .as_ref()
                    .unchecked_ref::<js_sys::Function>(),
            )
            .unwrap();
        Self { done, id }
    }
}

impl Drop for TimeoutFuture {
    fn drop(&mut self) {
        if let Some(window) = web_sys::window() {
            window.clear_timeout_with_handle(self.id);
        }
    }
}

impl Future for TimeoutFuture {
    type Output = Result<JsValue, JsValue>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.done.load(std::sync::atomic::Ordering::SeqCst) {
            Poll::Ready(Ok(JsValue::NULL))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}
