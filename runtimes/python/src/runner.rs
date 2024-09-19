use std::{
    collections::HashMap,
    sync::{LazyLock, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use log::{debug, error, info, trace};
use runtimes::{
    api::{ApiClient, Identifier, Value},
    message::{ErrorResponseMessage, ResponseMessage, ServerToClientMessage},
};
use rustpython_vm::{
    builtins::PyStr,
    function::{KwArgs, PosArgs},
    pymodule,
    signal::{UserSignal, UserSignalSender},
    PyObject, PyResult, Settings, TryFromBorrowedObject, VirtualMachine,
};

// FIXME this should be one lookup table per Python VM; workaround: one table per python thread might be enough
pub(super) static API_CLIENTS: LazyLock<Mutex<HashMap<Identifier, Box<dyn ApiClient>>>> =
    LazyLock::new(Mutex::default);

pub struct RunnerBuilder {
    sys_path: String,
    main_module_name: String,
    api_clients: HashMap<Identifier, Box<dyn ApiClient>>,
}

impl RunnerBuilder {
    #[must_use]
    pub fn new(sys_path: String, main_module_name: String) -> Self {
        Self {
            sys_path,
            main_module_name,
            api_clients: HashMap::new(),
        }
    }

    pub fn add_api_client(&mut self, api_client: Box<dyn ApiClient>) {
        self.api_clients
            .insert(api_client.api_name().clone(), api_client);
    }

    pub fn build_and_run(self) -> PythonThread {
        let mut api_clients = API_CLIENTS.lock().unwrap();
        api_clients.extend(self.api_clients);

        let (user_signal_sender, user_signal_receiver) =
            rustpython_vm::signal::user_signal_channel();

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
                vm.insert_sys_path(vm.new_pyobj(self.sys_path))
                    .expect("add path");

                let main_module_name = vm.ctx.intern_str(self.main_module_name);

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

#[pymodule]
mod rust_py_module {
    use super::{IdentifierConverter, PyResult, VirtualMachine};
    use rustpython_vm::function::{KwArgs, PosArgs};

    #[pyfunction]
    fn message(
        name: IdentifierConverter,
        args: PosArgs,
        kwargs: KwArgs,
        vm: &VirtualMachine,
    ) -> PyResult<()> {
        // just forward to a location outside of this macro so that the IDE can assist us
        super::message(name, args, kwargs, vm)
    }

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
}

fn message(
    // api_name: Identifier,
    name: IdentifierConverter,
    args: PosArgs,
    kwargs: KwArgs,
    vm: &VirtualMachine,
) -> PyResult<()> {
    // TODO move the api selection into the caller
    let api_name = Identifier("robot".into());
    let api = Api {};
    let command = name.convert(vm, &api)?;
    let parameters = ParameterConverter::new(args, kwargs).convert(vm, &api, &command)?;

    let mut api_clients = API_CLIENTS.lock().unwrap();
    let api_client = api_clients.get_mut(&api_name).unwrap();
    let message_id = api_client.send_command(command, parameters);

    // TODO move this polling into the python bindgen layer to enable user scripts to perform async calls rather than blocking
    let response = loop {
        match api_client.poll_response() {
            Some(response) => break response,
            None => thread::sleep(Duration::from_millis(10)),
        }
    };

    match response {
        ServerToClientMessage::Response(ResponseMessage { id, result }) => {
            assert_eq!(message_id, id, "request-response id mismatch");
            trace!("command successfully returned: {result}");
        }
        ServerToClientMessage::ErrorResponse(ErrorResponseMessage { id, message }) => {
            assert_eq!(message_id, id, "request-response id mismatch");
            error!("command returned an error: {message}");
            let error = vm.new_runtime_error(message);
            return Err(error);
        }
        ServerToClientMessage::Event(_) => todo!(),
    }

    Ok(())
}

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
    fn convert(self, vm: &VirtualMachine, api: &Api) -> PyResult<Identifier> {
        match self.0 {
            Some(name) if api.check_identifier(&name) => Ok(Identifier(name.into())),
            Some(name) => Err(vm.new_value_error(format!("{name:?} is not an identifier name"))),
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
        _command: &Identifier,
    ) -> PyResult<Vec<Value>> {
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
