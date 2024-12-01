use crate::{
    api_client::{insert_api_client, py_api_client},
    api_server::{insert_api_server, py_api_server},
};
use gam3du_framework_common::{
    api::{Identifier, Value},
    api_channel::{ApiClientEndpoint, ApiServerEndpoint},
    message::{ClientToServerMessage, RequestMessage},
    module::Module,
};
use runtime_python_bindgen::PyIdentifier;
use rustpython_vm::{
    builtins::PyStrInterned,
    convert::IntoObject,
    function::FuncArgs,
    signal::{user_signal_channel, UserSignal, UserSignalReceiver, UserSignalSender},
    Interpreter, PyObjectRef, Settings,
};
use std::{
    collections::HashMap,
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};
use tracing::{debug, error, info};

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

    api_clients: HashMap<Identifier, Box<dyn ApiClientEndpoint>>,
    api_servers: HashMap<Identifier, Arc<Mutex<dyn ApiServerEndpoint>>>,
    native_modules: HashMap<String, StdlibInitFunc>,
}

impl PythonRuntimeBuilder {
    #[must_use]
    pub fn new(sys_path: &Path, main_module_name: impl Into<String>) -> Self {
        Self {
            sys_path: sys_path.to_string_lossy().into_owned(),
            main_module_name: main_module_name.into(),
            user_signal_receiver: None,
            api_clients: HashMap::new(),
            api_servers: HashMap::new(),
            native_modules: HashMap::new(),
        }
    }

    pub fn add_api_client(&mut self, api_client: Box<dyn ApiClientEndpoint + 'static>) {
        assert!(
            self.api_clients
                .insert(api_client.api().name.clone(), api_client)
                .is_none(),
            "duplicate api name"
        );
    }

    pub fn add_api_server(&mut self, api_server: impl ApiServerEndpoint + 'static) {
        assert!(
            self.api_servers
                .insert(
                    api_server.api().name.clone(),
                    Arc::new(Mutex::new(api_server))
                )
                .is_none(),
            "duplicate api name"
        );
    }

    pub fn add_native_module(&mut self, name: impl Into<String>, init_fn: StdlibInitFunc) {
        assert!(
            self.native_modules.insert(name.into(), init_fn).is_none(),
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

    pub fn build(self) -> PythonRuntime {
        let Self {
            sys_path,
            main_module_name,
            user_signal_receiver,
            api_clients,
            api_servers,
            native_modules,
        } = self;

        let id = VM_ID.fetch_add(1, Ordering::Relaxed).to_string();
        let has_api_clients = !api_clients.is_empty();
        let has_api_servers = !api_servers.is_empty();

        let id_clone = id.clone();
        let api_clients_clone = api_clients.keys().cloned().collect::<Vec<_>>();
        let api_servers_clone = api_servers.keys().cloned().collect::<Vec<_>>();
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

                if let Some(user_signal_receiver) = user_signal_receiver {
                    vm.set_user_signal_channel(user_signal_receiver);
                }

                if has_api_clients {
                    vm.add_native_module(
                        "api_client".to_owned(),
                        Box::new(py_api_client::make_module),
                    );

                    for api_name in api_clients_clone {
                        let api_module = format!("{}_api_internal", api_name.module());
                        debug!("adding native module {api_module}");
                        vm.add_native_module(api_module, Box::new(py_api_client::make_module));
                    }
                }

                if has_api_servers {
                    vm.add_native_module(
                        "api_server".to_owned(),
                        Box::new(py_api_server::make_module),
                    );

                    for api_name in api_servers_clone {
                        let api_module = format!("{}_api_internal", api_name.module());
                        debug!("adding native module {api_module}");
                        vm.add_native_module(api_module, Box::new(py_api_server::make_module));
                    }
                }

                for (name, init) in native_modules {
                    vm.add_native_module(name, init());
                }
            }))
            .interpreter();

        let mut interned_main_module_name = None;
        interpreter.enter(|vm| {
            vm.insert_sys_path(vm.new_pyobj(sys_path))
                .expect("failed to add {sys_path} to python vm");

            interned_main_module_name = Some(vm.ctx.intern_str(main_module_name));

            for (api_name, api_client) in api_clients {
                // let api_module = "robot_api_internal";
                let api_module = format!("{}_api_internal", api_name.module());
                // let api = self
                //     .api_clients
                //     .remove(&Identifier("robot".into()))
                //     .unwrap();
                insert_api_client(vm, &api_module, api_client);
            }

            for (api_name, api_server) in &api_servers {
                // let api_module = "robot_api_internal";
                let api_module = format!("{}_api_internal", api_name.module());
                // let api = self
                //     .api_clients
                //     .remove(&Identifier("robot".into()))
                //     .unwrap();
                insert_api_server(vm, &api_module, Arc::clone(api_server));
            }
        });

        PythonRuntime {
            main_module_name: interned_main_module_name.unwrap(),
            interpreter,
            api_server_endpoints: api_servers.into_values().collect(),
            module: None,
        }
    }

    // FIXME requires the api servers to be `Send` which is not possible or necessary on WASM
    // #[must_use]
    // pub fn build_runner_thread(self) -> JoinHandle<()> {
    //     thread::Builder::new()
    //         // .stack_size(10 * 1024 * 1024)
    //         .spawn(|| {
    //             debug!("thread[python]: start interpreter");
    //             let mut runtime = self.build();
    //             runtime.enter_main();
    //         })
    //         .unwrap()
    // }
}

pub struct PythonRuntime {
    main_module_name: &'static PyStrInterned,
    pub interpreter: Interpreter,
    api_server_endpoints: Vec<Arc<Mutex<dyn ApiServerEndpoint>>>,
    pub module: Option<PyObjectRef>,
}

impl Module for PythonRuntime {
    fn enter_main(&mut self) {
        self.interpreter.enter(|vm| {
            match vm.import(self.main_module_name, 0) {
                Ok(module) => {
                    self.module = Some(module);
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

    fn wake(&mut self) {
        for api_server_endpoint in &mut self.api_server_endpoints {
            'next_message: loop {
                let incoming_message = {
                    let Some(message) = api_server_endpoint.lock().unwrap().poll_request() else {
                        break 'next_message;
                    };
                    message
                };
                // }

                // while let Some(incoming_message) = api_server_endpoint.lock().unwrap().poll_request() {
                let ClientToServerMessage::Request(request) = incoming_message;

                let RequestMessage {
                    id,
                    command,
                    arguments,
                } = request;

                let module = self.module.as_mut().expect("cannot wake() before init()");
                self.interpreter.enter(|vm| {
                    let py_id = vm.ctx.new_int(id.0.get()).into_object();
                    let mut args = vec![py_id];
                    args.extend(arguments.into_iter().map(|argument| match argument {
                        Value::Unit => todo!(),
                        Value::Integer(value) => vm.ctx.new_int(value).into_object(),
                        Value::Float(value) => vm.ctx.new_float(f64::from(value)).into_object(),
                        Value::Boolean(value) => vm.ctx.new_bool(value).into_object(),
                        Value::String(value) => vm.ctx.intern_str(value).to_object(),
                        Value::List(_value) => todo!(),
                    }));

                    let args = FuncArgs::from(args);
                    let handler_function_name =
                        vm.ctx.intern_str(format!("on_{}", command.function()));
                    let callback = match module.get_attr(handler_function_name.as_str(), vm) {
                        Ok(callback) => callback,
                        Err(exception) => {
                            vm.print_exception(exception);
                            panic!("missing callback function");
                        }
                    };
                    match callback.call(args, vm) {
                        Ok(_result) => {}
                        Err(exception) => {
                            vm.print_exception(exception);
                            panic!("missing callback function");
                        }
                    }
                });
            }
        }
    }
}

pub struct PythonRunnerThread {
    join_handle: JoinHandle<()>,
    user_signal_sender: UserSignalSender,
}

impl PythonRunnerThread {
    // #[must_use]
    // fn new(join_handle: JoinHandle<()>, user_signal_sender: UserSignalSender) -> Self {
    //     Self {
    //         join_handle,
    //         user_signal_sender,
    //     }
    // }

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
