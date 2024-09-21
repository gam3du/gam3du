use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    thread::{self, JoinHandle},
};

use gam3du_framework::module::Module;
use log::{debug, error, info};
use runtimes::api::ApiClient;
use rustpython_vm::{
    builtins::PyStrInterned,
    signal::{UserSignal, UserSignalReceiver, UserSignalSender},
    stdlib::StdlibInitFunc,
    Interpreter, Settings,
};

use crate::api_client::{py_api_client, API_CLIENTS};

static VM_ID: AtomicU64 = AtomicU64::new(0);

pub struct PythonRunner {
    id: String,
    main_module_name: &'static PyStrInterned,
    interpreter: Interpreter,
}

impl PythonRunner {
    fn new(
        sys_path: String,
        main_module_name: String,
        user_signal_receiver: Option<UserSignalReceiver>,
        more_native_modules: Vec<(String, StdlibInitFunc)>,
    ) -> Self {
        let id = VM_ID.fetch_add(1, Ordering::Relaxed).to_string();

        let id_clone = id.clone();
        let interpreter = rustpython::InterpreterConfig::new()
            .settings({
                let mut settings = Settings::default();
                settings.install_signal_handlers = false;
                settings
            })
            .init_stdlib()
            .init_hook(Box::new(|vm| {
                // TODO find a better way to identify this VM than abusing this field
                vm.wasm_id = Some(id_clone);

                if let Some(user_signal_receiver) = user_signal_receiver {
                    vm.set_user_signal_channel(user_signal_receiver);
                }

                vm.add_native_module(
                    "robot_api_internal".to_owned(),
                    Box::new(py_api_client::make_module),
                );

                for (name, init) in more_native_modules {
                    vm.add_native_module(name, init);
                }
            }))
            .interpreter();

        let mut interned_main_module_name = None;
        interpreter.enter(|vm| {
            vm.insert_sys_path(vm.new_pyobj(sys_path))
                .expect("failed to add {sys_path} to python vm");

            interned_main_module_name = Some(vm.ctx.intern_str(main_module_name));
        });

        API_CLIENTS.with_borrow_mut(|clients| {
            clients.insert(id.clone(), HashMap::default());
        });

        Self {
            interpreter,
            main_module_name: interned_main_module_name.unwrap(),
            id,
        }
    }
}

impl Module for PythonRunner {
    fn add_api_client(&mut self, api_client: Box<dyn ApiClient>) {
        API_CLIENTS.with_borrow_mut(|clients| {
            let api_clients = clients.get_mut(&self.id).unwrap();
            api_clients.insert(api_client.api_name().clone(), api_client);
        });
    }

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

    fn wake(&self) {}
}

pub struct ThreadBuilder {
    sys_path: String,
    main_module_name: String,
    api_clients: Vec<Box<dyn ApiClient>>,
    more_native_modules: Vec<(String, StdlibInitFunc)>,
}

impl ThreadBuilder {
    #[must_use]
    pub fn new(
        sys_path: String,
        main_module_name: String,
        more_native_modules: Vec<(String, StdlibInitFunc)>,
    ) -> Self {
        Self {
            sys_path,
            main_module_name,
            api_clients: Vec::new(),
            more_native_modules,
        }
    }

    pub fn add_api_client(&mut self, api_client: Box<dyn ApiClient>) {
        self.api_clients.push(api_client);
    }

    #[must_use]
    pub fn build_and_run(self) -> PythonThread {
        // let mut api_clients = API_CLIENTS.lock().unwrap();
        // api_clients.extend(self.api_clients);

        let (user_signal_sender, user_signal_receiver) =
            rustpython_vm::signal::user_signal_channel();

        let join_handle = thread::spawn(|| {
            debug!("thread[python]: start interpreter");

            let mut runner = PythonRunner::new(
                self.sys_path,
                self.main_module_name,
                Some(user_signal_receiver),
                self.more_native_modules,
            );

            for client in self.api_clients {
                runner.add_api_client(client);
            }

            runner.enter_main();
        });

        PythonThread {
            join_handle,
            user_signal_sender,
        }
    }
}

pub struct PythonThread {
    join_handle: JoinHandle<()>,
    user_signal_sender: UserSignalSender,
}

impl PythonThread {
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
