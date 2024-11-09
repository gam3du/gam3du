use crate::api_client::py_api_client::{MaybeFulfilled, RequestHandle};
use gam3du_framework_common::{
    api::{ApiClientEndpoint, ApiDescriptor, Identifier, TypeDescriptor, Value},
    message::{ErrorResponseMessage, ResponseMessage, ServerToClientMessage},
};
use rustpython_vm::{builtins::PyBaseExceptionRef, convert::IntoObject};
use rustpython_vm::{
    builtins::PyStr, convert::ToPyObject, function::PosArgs, pyclass, pymodule, PyObject,
    PyObjectRef, PyPayload, PyRef, PyResult, TryFromBorrowedObject, VirtualMachine,
};
use tracing::{error, trace};

pub(crate) fn insert_api_client(vm: &VirtualMachine, api_module: &str, api: ApiClientEndpoint) {
    let api_module = vm.ctx.intern_str(api_module);
    let module = vm
        .import(api_module, 0)
        .expect("Expect robot api must be imported");

    module
        .set_attr("_private_api", PrivateApi::wrap(api).into_py(vm), vm)
        .expect("Set private api client");
}

fn get_api_client(vm: &VirtualMachine, api_module: &str) -> PyRef<PrivateApi> {
    // let api_module = vm.ctx.intern_str(api_module);

    let sys_modules = vm.sys_module.get_attr("modules", vm).unwrap();
    let module = match sys_modules.get_item(api_module, vm) {
        Ok(module) => module,
        Err(exception) => {
            vm.print_exception(exception);
            panic!("could not find module {api_module}");
        }
    };

    // let module = vm
    //     .import(api_module, 0)
    //     .expect("Expect robot api must be imported");

    let object = module
        .get_attr("_private_api", vm)
        .expect("Private api must be present");

    object.downcast().expect("Private api must be intact")
}

#[pyclass(name = "PrivateApi", module = false)]
struct PrivateApi {
    api: ApiClientEndpoint,
}

impl PrivateApi {
    fn wrap(api: ApiClientEndpoint) -> Self {
        Self { api }
    }

    fn into_py(self, vm: &VirtualMachine) -> PyObjectRef {
        vm.new_pyobj(self)
    }
}

impl std::fmt::Debug for PrivateApi {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "PrivateApi {{ /* private */ }}")
    }
}

impl PyPayload for PrivateApi {
    fn class(
        ctx: &rustpython_vm::Context,
    ) -> &'static rustpython_vm::Py<rustpython_vm::builtins::PyType> {
        ctx.types.object_type
    }
}

//type VmApiClients = HashMap<String, HashMap<Identifier, Box<dyn ApiClient>>>;
//
//thread_local! {
//    pub(super) static API_CLIENTS: RefCell<VmApiClients> = RefCell::default();
//}

#[pymodule]
pub(crate) mod py_api_client {

    use super::{FunctionNameConverter, PyResult, VirtualMachine};
    use gam3du_framework_common::message::RequestId;
    use rustpython_vm::{
        builtins::PyBaseExceptionRef, function::PosArgs, pyclass, PyObjectRef, PyPayload,
        TryFromObject,
    };

    #[pyfunction]
    fn message(
        name: FunctionNameConverter,
        args: PosArgs,
        // kwargs: KwArgs,
        vm: &VirtualMachine,
    ) -> PyResult<RequestHandle> {
        // just forward to a location outside of this macro so that the IDE can assist us
        super::message(name, args, vm)
    }

    #[pyfunction]
    fn poll(
        request: RequestHandle,
        vm: &VirtualMachine,
    ) -> Result<MaybeFulfilled, PyBaseExceptionRef> {
        // just forward to a location outside of this macro so that the IDE can assist us
        super::poll(request, vm)
    }

    #[pyclass(name, module = "py_api_client", no_attr)]
    #[derive(Copy, Clone, PyPayload)]
    pub(super) struct RequestHandle {
        id: RequestId,
    }

    #[pyclass]
    impl RequestHandle {}

    #[allow(
        clippy::multiple_inherent_impl,
        reason = "required as separation between macro and non-macro code"
    )]
    impl RequestHandle {
        pub(super) fn new(request_id: RequestId) -> Self {
            Self { id: request_id }
        }

        pub(super) fn inner(&self) -> RequestId {
            self.id
        }
    }

    impl std::fmt::Debug for RequestHandle {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter
                .debug_struct("RequestHandle")
                .field("request_id", &self.id)
                .finish()
        }
    }

    impl TryFromObject for RequestHandle {
        fn try_from_object(vm: &VirtualMachine, obj: PyObjectRef) -> PyResult<Self> {
            obj.payload()
                .copied()
                .ok_or_else(|| vm.new_value_error("invalid pending request".to_owned()))
        }
    }

    #[pyclass(name, module = "py_api_client", no_attr)]
    #[derive(PyPayload)]
    pub(super) struct MaybeFulfilled {
        id: RequestId,
        value: Option<PyObjectRef>,
    }

    impl MaybeFulfilled {
        pub(super) fn new(request_id: RequestId) -> Self {
            Self {
                id: request_id,
                value: None,
            }
        }

        pub(super) fn with_value(self, value: PyObjectRef) -> Self {
            Self {
                id: self.id,
                value: Some(value),
            }
        }
    }

    impl std::fmt::Debug for MaybeFulfilled {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter
                .debug_struct("MaybeFulfilled")
                .field("request_id", &self.id)
                .finish_non_exhaustive()
        }
    }

    /// Public Python API
    #[pyclass]
    #[allow(
        clippy::multiple_inherent_impl,
        reason = "required as separation between macro and non-macro code"
    )]
    impl MaybeFulfilled {
        #[pymethod]
        fn is_done(&self) -> bool {
            self.value.is_some()
        }

        #[pymethod]
        fn get_value(&self) -> PyObjectRef {
            self.value.clone().unwrap()
        }
    }
}

fn poll(request: RequestHandle, vm: &VirtualMachine) -> Result<MaybeFulfilled, PyBaseExceptionRef> {
    let api_module = "robot_control_api_internal";
    let private_api = get_api_client(vm, api_module);
    let message_id = request.inner();

    let response = private_api.api.poll_response();

    match response {
        None => Ok(MaybeFulfilled::new(message_id)),
        Some(response) => match response {
            ServerToClientMessage::Response(ResponseMessage { id, result }) => {
                assert_eq!(message_id, id, "request-response id mismatch");
                trace!("command successfully returned: {result:?}");
                let value = match result {
                    Value::Unit => vm.ctx.none(),
                    Value::Integer(_) => todo!(),
                    Value::Float(_) => todo!(),
                    Value::Boolean(value) => value.to_pyobject(vm),
                    Value::String(_) => todo!(),
                    Value::List(_) => todo!(),
                };
                Ok(MaybeFulfilled::new(message_id).with_value(value.into_object()))
            }
            ServerToClientMessage::ErrorResponse(ErrorResponseMessage { id, message }) => {
                assert_eq!(message_id, id, "request-response id mismatch");
                error!("command returned an error: {message}");
                let error = vm
                    .invoke_exception(
                        vm.ctx.exceptions.runtime_error.to_owned(),
                        vec![vm.ctx.new_str(message).to_pyobject(vm)],
                    )
                    .expect("Constructor of \"RuntimeError\" should not fail");
                vm.print_exception(error.clone());
                Err(error)
            }
        },
    }
}

fn message(
    // api_name: Identifier,
    name: FunctionNameConverter,
    args: PosArgs,
    // kwargs: KwArgs,
    vm: &VirtualMachine,
) -> PyResult<RequestHandle> {
    // TODO move the api selection into the caller
    // let api_name = Identifier("robot".into());
    // let vm_id = vm.wasm_id.as_ref().unwrap();

    let api_module = "robot_control_api_internal";
    let private_api = get_api_client(vm, api_module);
    let api = private_api.api.api();

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
                #[allow(
                    clippy::cast_possible_truncation,
                    reason = "the api only supports f32 at the moment"
                )]
                let primitive = int.to_f64() as f32;
                Value::Float(primitive)
            }
            TypeDescriptor::Boolean => todo!(),
            TypeDescriptor::String => todo!(),
            TypeDescriptor::List(_type_descriptor) => todo!(),
        })
        .collect();

    Ok(RequestHandle::new(
        private_api.api.send_command(command, arguments),
    ))
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
