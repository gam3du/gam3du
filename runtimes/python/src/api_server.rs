use gam3du_framework_common::{
    api::{ApiServerEndpoint, Identifier, Value},
    message::RequestId,
};
use runtime_python_bindgen::PyIdentifier;
use rustpython_vm::{pyclass, pymodule, PyObjectRef, PyPayload, PyRef, VirtualMachine};
use std::{
    borrow::BorrowMut,
    sync::{Arc, Mutex},
};

pub(crate) fn insert_api_server(
    vm: &VirtualMachine,
    api_module: &str,
    api: Arc<Mutex<ApiServerEndpoint>>,
) {
    let api_module = vm.ctx.intern_str(api_module);
    let module = vm.import(api_module, 0).unwrap_or_else(|exception| {
        vm.print_exception(exception);
        panic!("Could not import api server module `{api_module}`");
    });

    module
        .set_attr("_private_api", PrivateApiServer::wrap(api).into_py(vm), vm)
        .expect("Set private api client");
}

fn get_api_server(vm: &VirtualMachine, api_module: &str) -> PyRef<PrivateApiServer> {
    let sys_modules = vm.sys_module.get_attr("modules", vm).unwrap();
    let module = sys_modules
        .get_item(api_module, vm)
        .unwrap_or_else(|exception| {
            vm.print_exception(exception);
            panic!("Could not find module `{api_module}`");
        });

    let object = module
        .get_attr("_private_api", vm)
        .unwrap_or_else(|exception| {
            vm.print_exception(exception);
            panic!("Could not find private api in module `{api_module}`");
        });

    object.downcast().expect("Private api must be intact")
}

#[pyclass(name = "PrivateApiServer", module = false)]
struct PrivateApiServer {
    api: Arc<Mutex<ApiServerEndpoint>>,
}

impl PrivateApiServer {
    fn wrap(api: Arc<Mutex<ApiServerEndpoint>>) -> Self {
        Self { api }
    }

    fn into_py(self, vm: &VirtualMachine) -> PyObjectRef {
        vm.new_pyobj(self)
    }
}

impl std::fmt::Debug for PrivateApiServer {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "PrivateApiServer {{ /* private */ }}")
    }
}

impl PyPayload for PrivateApiServer {
    fn class(
        ctx: &rustpython_vm::Context,
    ) -> &'static rustpython_vm::Py<rustpython_vm::builtins::PyType> {
        ctx.types.object_type
    }
}

#[pymodule]
pub(crate) mod py_api_server {
    use super::VirtualMachine;

    #[pyfunction]
    fn send_boolean_response(api_name: String, request_id: u128, value: bool, vm: &VirtualMachine) {
        // just forward to a location outside of this macro so that the IDE can assist us
        super::send_boolean_response(api_name, request_id, value, vm);
    }
}

fn send_boolean_response(api_name: String, request_id: u128, value: bool, vm: &VirtualMachine) {
    let api_server_module_name = format!(
        "{}_api_internal",
        Identifier::try_from(api_name).unwrap().file()
    );
    let mut binding = get_api_server(vm, &api_server_module_name);
    let private_api_server_module = binding.borrow_mut();

    let request_id = RequestId::try_from(request_id).unwrap();

    private_api_server_module
        .api
        .lock()
        .unwrap()
        .send_response(request_id, Value::Boolean(value));
}
