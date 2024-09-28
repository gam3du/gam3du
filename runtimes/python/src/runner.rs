use crate::api_client::{insert_api_client, py_api_client};
use gam3du_framework::{
    api::{ApiClientEndpoint, Identifier},
    module::Module,
};
use log::{debug, error, info};
use rustpython_vm::{
    builtins::PyStrInterned,
    signal::{user_signal_channel, UserSignal, UserSignalReceiver, UserSignalSender},
    Interpreter, Settings,
};
use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    thread::{self, JoinHandle},
};

/// This indirection is necessary because we can't pass `rustpython_vm::stdlib::StdlibInitFunc`
/// to a new thread (`std::thread::spawn` requires `Send`).
/// Instead of passing it directly to the new python interpreter thread,
/// we can pass a function pointer (which is always `Send`).
type StdlibInitFunc = fn() -> rustpython_vm::stdlib::StdlibInitFunc;

static VM_ID: AtomicU64 = AtomicU64::new(0);

pub struct PythonRuntimeBuilder {
    sys_path: String,
    main_module_name: String,
    user_signal_receiver: Option<UserSignalReceiver>,

    api_clients: HashMap<Identifier, ApiClientEndpoint>,
    native_modules: HashMap<String, StdlibInitFunc>,
}

impl PythonRuntimeBuilder {
    #[must_use]
    pub fn new(sys_path: String, main_module_name: String) -> Self {
        Self {
            sys_path,
            main_module_name,
            user_signal_receiver: None,
            api_clients: HashMap::new(),
            native_modules: HashMap::new(),
        }
    }

    pub fn add_api_client(&mut self, api_client: ApiClientEndpoint) {
        assert!(
            self.api_clients
                .insert(api_client.api().name.clone(), api_client)
                .is_none(),
            "duplicate api name"
        );
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "passing a reference would make it impossible to pass a `String` without cloning"
    )]
    pub fn add_native_module(&mut self, name: impl ToString, init_fn: StdlibInitFunc) {
        assert!(
            self.native_modules
                .insert(name.to_string(), init_fn)
                .is_none(),
            "duplicate module name"
        );
    }

    pub fn enable_user_signals(&mut self) -> UserSignalSender {
        let (user_signal_sender, user_signal_receiver) = user_signal_channel();
        assert!(
            self.user_signal_receiver
                .replace(user_signal_receiver)
                .is_none(),
            "only one user channel can be set"
        );
        user_signal_sender
    }

    pub fn build(mut self) -> PythonRuntime {
        let id = VM_ID.fetch_add(1, Ordering::Relaxed).to_string();
        let has_api_clients = !self.api_clients.is_empty();

        let id_clone = id.clone();
        let interpreter = rustpython::InterpreterConfig::new()
            .settings({
                let mut settings = Settings::default();
                settings.install_signal_handlers = false;
                settings
            })
            .init_stdlib()
            .init_hook(Box::new(move |vm| {
                // TODO find a better way to identify this VM than abusing this field
                vm.wasm_id = Some(id_clone);

                if let Some(user_signal_receiver) = self.user_signal_receiver {
                    vm.set_user_signal_channel(user_signal_receiver);
                }

                if has_api_clients {
                    vm.add_native_module(
                        "robot_api_internal".to_owned(),
                        Box::new(py_api_client::make_module),
                    );
                }

                for (name, init) in self.native_modules {
                    vm.add_native_module(name, init());
                }
            }))
            .interpreter();

        let mut interned_main_module_name = None;
        interpreter.enter(|vm| {
            vm.insert_sys_path(vm.new_pyobj(self.sys_path))
                .expect("failed to add {sys_path} to python vm");

            interned_main_module_name = Some(vm.ctx.intern_str(self.main_module_name));

            let api_module = "robot_api_internal";
            let api = self
                .api_clients
                .remove(&Identifier("robot".into()))
                .unwrap();
            insert_api_client(vm, api_module, api);
        });

        PythonRuntime {
            main_module_name: interned_main_module_name.unwrap(),
            interpreter,
        }
    }

    #[must_use]
    pub fn build_runner_thread(self, user_signal_sender: UserSignalSender) -> PythonRunnerThread {
        let handle = thread::Builder::new()
            .stack_size(10 * 1024 * 1024)
            .spawn(|| {
                debug!("thread[python]: start interpreter");
                let runtime = self.build();
                runtime.enter_main();
            })
            .unwrap();

        PythonRunnerThread::new(handle, user_signal_sender)
    }
}

pub struct PythonRuntime {
    main_module_name: &'static PyStrInterned,
    interpreter: Interpreter,
}

impl Module for PythonRuntime {
    fn enter_main(&self) {
        self.interpreter.enter(|vm| {
            match vm.import(self.main_module_name, 0) {
                Ok(_module) => {
                    info!("Python thread completed successfully");
                }
                Err(exc) => {
                    let mut msg = String::new();
                    vm.write_exception(&mut msg, &exc).unwrap();
                    error!("Python thread exited with exception: {msg}");
                }
            }

            debug!("thread[python]: exit");
        });
    }

    fn wake(&self) {
        todo!("query all API endpoints for updates and notify the VM");
    }
}

pub struct PythonRunnerThread {
    join_handle: JoinHandle<()>,
    user_signal_sender: UserSignalSender,
}

impl PythonRunnerThread {
    #[must_use]
    fn new(join_handle: JoinHandle<()>, user_signal_sender: UserSignalSender) -> Self {
        Self {
            join_handle,
            user_signal_sender,
        }
    }

    pub fn stop(&self) {
        let make_interrupt: UserSignal = Box::new(|vm| {
            // Copied from rustpython_vm::stdlib::signal::_signal::default_int_handler
            let exec_type = vm.ctx.exceptions.keyboard_interrupt.to_owned();
            Err(vm.new_exception_empty(exec_type))
        });
        self.user_signal_sender.send(make_interrupt).unwrap();
    }

    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.join_handle.is_finished()
    }

    pub fn join(self) -> thread::Result<()> {
        self.join_handle.join()
    }
}
