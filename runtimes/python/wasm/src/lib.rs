//! TODO
#![allow(missing_docs, reason = "TODO")]
#![expect(clippy::print_stdout, reason = "just a demo")]

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::js_sys;
use web_sys::console::log_1;

#[wasm_bindgen]
extern "C" {
    // #[wasm_bindgen(js_namespace = window)]
    fn callback(s: &str) -> u32;
}

type Task = Pin<Box<dyn Future<Output = Result<JsValue, JsValue>> + 'static>>;

#[wasm_bindgen]
pub struct RustPromise {
    task: Task,
    result: Option<JsValue>,
}

#[wasm_bindgen]
impl RustPromise {
    fn new(future: impl Future<Output = Result<JsValue, JsValue>> + 'static) -> Self {
        let task = Box::pin(future);
        Self { task, result: None }
    }

    pub fn poll(&mut self) -> Result<JsValue, JsValue> {
        fn noop_waker_ref() -> &'static Waker {
            const NOOP: RawWaker = {
                const VTABLE: RawWakerVTable = RawWakerVTable::new(
                    // Cloning just returns a new no-op raw waker
                    |_| NOOP,
                    // `wake` does nothing
                    |_| {},
                    // `wake_by_ref` does nothing
                    |_| {},
                    // Dropping does nothing as we don't allocate anything
                    |_| {},
                );
                RawWaker::new(&(), &VTABLE)
            };

            static NOOP_WAKER: Waker = unsafe { Waker::from_raw(NOOP) };

            &NOOP_WAKER
        }

        /// Create a Js object conforming to the `iterator result interface`
        /// from the [`iterator protocol`](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Iteration_protocols#the_iterator_protocol).
        /// TODO: Optimize this with js_sys::Object::create?
        fn iterator_result_interface(done: bool, value: Option<&JsValue>) -> JsValue {
            let undefined = JsValue::UNDEFINED;
            let done_key = "done".into();
            let done_value = done.into();
            let value_key = "value".into();
            let value_value = value.unwrap_or_else(|| &undefined);
            let entries = JsValue::from(js_sys::Array::of2(
                &js_sys::Array::of2(&done_key, &done_value),
                &js_sys::Array::of2(&value_key, &value_value),
            ));
            let object = js_sys::Object::from_entries(&entries);
            // Given the arguments are statically enforced to be valid, this can't fail.
            let object = object.unwrap();
            JsValue::from(object)
        }

        if let Some(value) = &self.result {
            return Ok(iterator_result_interface(true, Some(value)));
        }

        let mut context = Context::from_waker(noop_waker_ref());
        match self.task.as_mut().poll(&mut context) {
            Poll::Ready(Ok(value)) => {
                self.result = Some(value);
                if let Some(value) = &self.result {
                    return Ok(iterator_result_interface(true, Some(value)));
                }
                unreachable!()
            }
            Poll::Ready(Err(error)) => Err(error),
            Poll::Pending => Ok(iterator_result_interface(false, None)),
        }
    }
}

#[wasm_bindgen]
pub fn greet(name: String) -> RustPromise {
    let mut i = 0;
    log_1(&"in greet".into());
    RustPromise::new(async move {
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

fn yield_point() -> impl Future {
    enum YieldOnce {
        Init,
        Done,
    }

    impl Future for YieldOnce {
        type Output = ();

        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            match &*self {
                Self::Init => {
                    self.set(Self::Done);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
                Self::Done => Poll::Ready(()),
            }
        }
    }

    YieldOnce::Init
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
