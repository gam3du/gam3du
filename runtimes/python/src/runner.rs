use std::{
    cell::RefCell,
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
    thread::{self, JoinHandle},
    time::Duration,
};

use gam3du_framework::module::Module;
use log::{debug, error, info, trace};
use runtimes::{
    api::{ApiClient, Identifier, TypeDescriptor, Value},
    message::{ErrorResponseMessage, ResponseMessage, ServerToClientMessage},
};
use rustpython_vm::{
    builtins::{PyStr, PyStrInterned},
    function::{KwArgs, PosArgs},
    pymodule,
    signal::{UserSignal, UserSignalReceiver, UserSignalSender},
    Interpreter, PyObject, PyResult, Settings, TryFromBorrowedObject, VirtualMachine,
};

type VmApiClients = HashMap<String, HashMap<Identifier, Box<dyn ApiClient>>>;

thread_local! {
    pub(super) static API_CLIENTS: RefCell<VmApiClients> = RefCell::default();
}

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
                    Box::new(rust_py_module::make_module),
                );
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
}

impl ThreadBuilder {
    #[must_use]
    pub fn new(sys_path: String, main_module_name: String) -> Self {
        Self {
            sys_path,
            main_module_name,
            api_clients: Vec::new(),
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
    // let mut parameters = Vec::new(); // ParameterConverter::new(args, kwargs).convert(vm, &api, &command)?;

    let vm_id = vm.wasm_id.as_ref().unwrap();

    // for arg in kwargs {
    //     info!("##############Arg: {arg:?}");
    // }

    API_CLIENTS.with_borrow_mut(|api_clients| {
        let api_clients = api_clients.get_mut(vm_id).unwrap();
        let api_client = api_clients.get_mut(&api_name).unwrap();
        let api = api_client.api();

        let function = api.functions.get(&command).expect("unknown command");
        let arguments = function
            .parameters
            .iter()
            .zip(args)
            .map(|(param, arg)| match &param.typ {
                TypeDescriptor::Integer(range) => {
                    let int = arg.try_int(vm).unwrap();
                    let primitive = int.try_to_primitive::<i64>(vm).unwrap();
                    assert!(primitive >= range.start, "integer parameter out of range");
                    assert!(primitive <= range.end, "integer parameter out of range");
                    Value::Integer(primitive)
                }
                TypeDescriptor::Float => {
                    let int = arg.try_float(vm).unwrap();
                    let primitive = int.to_f64() as f32;
                    Value::Float(primitive)
                }
                TypeDescriptor::Boolean => todo!(),
                TypeDescriptor::String => todo!(),
                TypeDescriptor::List(type_descriptor) => todo!(),
            })
            .collect();
        let message_id = api_client.send_command(command, arguments);

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
    })
}

// TODO: Can we use this to store a reference to the real api struct?
// Maybe as a singleton inside the python module?
struct Api;

impl Api {
    fn check_identifier(&self, name: &str) -> bool {
        true
        //name == "move forward" || name == "turn left" || name == "turn right"
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
