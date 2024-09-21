use std::{cell::RefCell, collections::HashMap, thread, time::Duration};

use log::{error, trace};
use runtimes::{
    api::{ApiClient, ApiDescriptor, Identifier, TypeDescriptor, Value},
    message::{ErrorResponseMessage, ResponseMessage, ServerToClientMessage},
};
use rustpython_vm::{
    builtins::PyStr,
    function::{KwArgs, PosArgs},
    pymodule, PyObject, PyResult, TryFromBorrowedObject, VirtualMachine,
};

type VmApiClients = HashMap<String, HashMap<Identifier, Box<dyn ApiClient>>>;

thread_local! {
    pub(super) static API_CLIENTS: RefCell<VmApiClients> = RefCell::default();
}

#[pymodule]
pub(crate) mod py_api_client {
    use super::{FunctionNameConverter, PyResult, VirtualMachine};
    use rustpython_vm::function::{KwArgs, PosArgs};

    #[pyfunction]
    fn message(
        name: FunctionNameConverter,
        args: PosArgs,
        kwargs: KwArgs,
        vm: &VirtualMachine,
    ) -> PyResult<()> {
        // just forward to a location outside of this macro so that the IDE can assist us
        super::message(name, args, kwargs, vm)
    }
}

fn message(
    // api_name: Identifier,
    name: FunctionNameConverter,
    args: PosArgs,
    kwargs: KwArgs,
    vm: &VirtualMachine,
) -> PyResult<()> {
    // TODO move the api selection into the caller
    let api_name = Identifier("robot".into());
    let vm_id = vm.wasm_id.as_ref().unwrap();

    API_CLIENTS.with_borrow_mut(|api_clients| {
        let api_clients = api_clients.get_mut(vm_id).unwrap();
        let api_client = api_clients.get_mut(&api_name).unwrap();
        let api = api_client.api();

        let command = name.convert(vm, api)?;

        let function = api.functions.get(&command).expect("unknown command");
        // TODO check that the number of given arguments matches the expected count
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

struct FunctionNameConverter(Option<String>);

impl FunctionNameConverter {
    fn convert(self, vm: &VirtualMachine, api: &ApiDescriptor) -> PyResult<Identifier> {
        let Some(name) = self.0 else {
            return Err(vm.new_type_error("Identifier name must be a string".to_owned()));
        };

        let identifier = Identifier(name.into());
        if api.functions.contains_key(&identifier) {
            Ok(identifier)
        } else {
            Err(vm.new_value_error(format!("{identifier} is not a known function name")))
        }
    }
}

impl<'obj> TryFromBorrowedObject<'obj> for FunctionNameConverter {
    fn try_from_borrowed_object(_: &VirtualMachine, obj: &'obj PyObject) -> PyResult<Self> {
        let identifier: Option<&PyStr> = obj.payload();
        let identifier = identifier.map(|pystr| pystr.as_ref().to_owned());
        Ok(Self(identifier))
    }
}

// struct ParameterConverter(PosArgs, KwArgs);

// impl ParameterConverter {
//     fn new(pos_args: PosArgs, kwargs: KwArgs) -> Self {
//         Self(pos_args, kwargs)
//     }

//     fn convert(
//         self,
//         _vm: &VirtualMachine,
//         _api: &ApiDescriptor,
//         _command: &Identifier,
//     ) -> PyResult<Vec<Value>> {
//         // TODO: Extract parameters
//         std::hint::black_box(&self.0);
//         std::hint::black_box(&self.1);
//         // let robot_event = api.find(command);
//         // let actuals = vec![];
//         // for parameter in robot_event.parameters() {
//         //     if let Some(actual) = find_positional_match(parameter, self.0) {
//         //         actuals.push(actual);
//         //         continue;
//         //     }
//         //     if let Some(actual) = find_keyword_match(parameter, self.1) {
//         //         actuals.push(actual);
//         //         continue;
//         //     }
//         // }
//         // return Ok(actuals)
//         Ok(vec![])
//     }
// }
