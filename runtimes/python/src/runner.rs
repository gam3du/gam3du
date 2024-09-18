use std::{
    sync::mpsc::Sender,
    thread::{self, JoinHandle},
};

use log::{debug, error, info};
use runtimes::event::EngineEvent;
use rustpython_vm::{
    pymodule,
    signal::{UserSignal, UserSignalSender},
    PyObject, PyResult, Settings, TryFromBorrowedObject, VirtualMachine,
};

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

pub fn run(
    sys_path: String,
    main_module_name: String,
    sender: Sender<EngineEvent>,
) -> PythonThread {
    rust_py_module::COMMAND_QUEUE
        .lock()
        .unwrap()
        .replace(sender);

    let (user_signal_sender, user_signal_receiver) = rustpython_vm::signal::user_signal_channel();

    let join_handle = thread::spawn(|| {
        debug!("thread[python]: start interpreter");

        let interpreter = rustpython::InterpreterConfig::new()
            .settings({
                let mut settings = Settings::default();
                settings.install_signal_handlers = false;
                settings
            })
            .init_stdlib()
            .init_hook(Box::new(|vm| {
                vm.set_user_signal_channel(user_signal_receiver);

                vm.add_native_module(
                    "robot_api_internal".to_owned(),
                    Box::new(rust_py_module::make_module),
                );
            }))
            .interpreter();

        interpreter.enter(|vm| {
            vm.insert_sys_path(vm.new_pyobj(sys_path))
                .expect("add path");

            let main_module_name = vm.ctx.intern_str(main_module_name);

            match vm.import(main_module_name, 0) {
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
    });

    PythonThread {
        join_handle,
        user_signal_sender,
    }
}

#[pymodule]
#[expect(
    clippy::unnecessary_wraps,
    clippy::unused_self,
    reason = "those are required by the Python API"
)]
mod rust_py_module {
    use std::{
        sync::{mpsc::Sender, Mutex},
        thread,
        time::Duration,
    };

    use runtimes::{api, event::EngineEvent};

    use super::{PyObject, PyResult, TryFromBorrowedObject, VirtualMachine};
    use rustpython_vm::{
        builtins::PyStr,
        function::{KwArgs, PosArgs},
    };

    pub(super) static COMMAND_QUEUE: Mutex<Option<Sender<EngineEvent>>> = Mutex::new(None);

    //     #[pyfunction]
    //     fn rust_function(
    //         num: i32,
    //         str: String,
    //         python_person: PythonPerson,
    //         _vm: &VirtualMachine,
    //     ) -> PyResult<RustStruct> {
    //         println!(
    //             "Calling standalone rust function from python passing args:
    // num: {},
    // string: {},
    // python_person.name: {}",
    //             num, str, python_person.name
    //         );
    //         Ok(RustStruct {
    //             numbers: NumVec(vec![1, 2, 3, 4]),
    //         })
    //     }

    // #[pyfunction]
    // fn move_forward() {
    //     COMMAND_QUEUE
    //         .lock()
    //         .unwrap()
    //         .as_mut()
    //         .unwrap()
    //         .send(EngineEvent::ApiCall {
    //             api: Identifier("robot".into()),
    //             command: Identifier("move forward".into()),
    //         })
    //         .unwrap();
    //     thread::sleep(Duration::from_millis(1000));
    // }

    // #[pyfunction]
    // fn turn_left() {
    //     COMMAND_QUEUE
    //         .lock()
    //         .unwrap()
    //         .as_mut()
    //         .unwrap()
    //         .send(EngineEvent::ApiCall {
    //             api: Identifier("robot".into()),
    //             command: Identifier("turn left".into()),
    //         })
    //         .unwrap();
    //     thread::sleep(Duration::from_millis(1000));
    // }

    // #[pyfunction]
    // fn turn_right() {
    //     COMMAND_QUEUE
    //         .lock()
    //         .unwrap()
    //         .as_mut()
    //         .unwrap()
    //         .send(EngineEvent::ApiCall {
    //             api: Identifier("robot".into()),
    //             command: Identifier("turn right".into()),
    //         })
    //         .unwrap();
    //     thread::sleep(Duration::from_millis(1000));
    // }

    // #[derive(Debug, Clone)]
    // struct NumVec(Vec<i32>);

    // impl ToPyObject for NumVec {
    //     fn to_pyobject(self, vm: &VirtualMachine) -> PyObjectRef {
    //         let list = self.0.into_iter().map(|item| vm.new_pyobj(item)).collect();
    //         PyList::new_ref(list, vm.as_ref()).to_pyobject(vm)
    //     }
    // }

    // #[pyattr]
    // #[pyclass(module = "rust_py_module", name = "RustStruct")]
    // #[derive(Debug, PyPayload)]
    // struct RustStruct {
    //     numbers: NumVec,
    // }

    // #[pyclass]
    // impl RustStruct {
    //     #[pygetset]
    //     fn numbers(&self) -> NumVec {
    //         self.numbers.clone()
    //     }

    //     #[pymethod]
    //     fn print_in_rust_from_python(&self) {
    //         println!("Calling a rust method from python");
    //     }
    // }

    // struct PythonPerson {
    //     name: String,
    // }

    // impl<'obj> TryFromBorrowedObject<'obj> for PythonPerson {
    //     fn try_from_borrowed_object(vm: &VirtualMachine, obj: &'obj PyObject) -> PyResult<Self> {
    //         let name = obj.get_attr("name", vm)?.try_into_value::<String>(vm)?;
    //         Ok(PythonPerson { name })
    //     }
    // }

    // TODO: Can we use this to store a reference to the real api struct?
    // Maybe as a singleton inside the python module?
    struct Api;

    impl Api {
        fn check_identifier(&self, name: &str) -> bool {
            name == "move forward" || name == "turn left" || name == "turn right"
        }
    }

    struct IdentifierConverter(Option<String>);

    impl IdentifierConverter {
        fn convert(self, vm: &VirtualMachine, api: &Api) -> PyResult<api::Identifier> {
            match self.0 {
                Some(name) if api.check_identifier(&name) => Ok(api::Identifier(name)),
                Some(name) => {
                    Err(vm.new_value_error(format!("{name:?} is not an identifier name")))
                }
                None => Err(vm.new_type_error("Identifier name must be a string".to_owned())),
            }
        }
    }

    impl<'obj> TryFromBorrowedObject<'obj> for IdentifierConverter {
        fn try_from_borrowed_object(_: &VirtualMachine, obj: &'obj PyObject) -> PyResult<Self> {
            let identifier: Option<&PyStr> = obj.payload();
            let identifier = identifier.map(|pystr| pystr.as_ref().to_owned());
            Ok(Self(identifier))
        }
    }

    struct ParameterConverter(PosArgs, KwArgs);

    impl ParameterConverter {
        fn new(pos_args: PosArgs, kwargs: KwArgs) -> Self {
            Self(pos_args, kwargs)
        }

        fn convert(
            self,
            _vm: &VirtualMachine,
            _api: &Api,
            _command: &api::Identifier,
        ) -> PyResult<Vec<api::Value>> {
            // TODO: Extract parameters
            std::hint::black_box(&self.0);
            std::hint::black_box(&self.1);
            // let robot_event = api.find(command);
            // let actuals = vec![];
            // for parameter in robot_event.parameters() {
            //     if let Some(actual) = find_positional_match(parameter, self.0) {
            //         actuals.push(actual);
            //         continue;
            //     }
            //     if let Some(actual) = find_keyword_match(parameter, self.1) {
            //         actuals.push(actual);
            //         continue;
            //     }
            // }
            // return Ok(actuals)
            Ok(vec![])
        }
    }

    #[pyfunction]
    fn message(
        name: IdentifierConverter,
        args: PosArgs,
        kwargs: KwArgs,
        vm: &VirtualMachine,
    ) -> PyResult<()> {
        let api = Api {};
        let command = name.convert(vm, &api)?;
        let parameters = ParameterConverter::new(args, kwargs).convert(vm, &api, &command)?;
        let result =
            COMMAND_QUEUE
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .send(EngineEvent::RobotEvent {
                    command,
                    parameters,
                });

        // If sending the message fails, the application
        // is probably already exiting.
        match result {
            Ok(()) => (),
            Err(_) => return Ok(()),
        }

        thread::sleep(Duration::from_millis(1000));
        Ok(())
    }
}
