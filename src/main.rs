#![warn(clippy::all, clippy::pedantic)]
// TODO re-enable this later and review all occurrences
#![allow(clippy::cast_precision_loss)]

// TODO enable hand-picked clippy lints from the `restriction` group

//mod application_state;
mod framework;
mod logging;
mod scene;

use std::{fs::read_to_string, path::Path, thread};

use log::{error, info};
use logging::init_logger;
use rustpython_vm::{self as vm, Settings};

fn python_runner(source_path: &(impl AsRef<Path> + ToString)) {
    let source = read_to_string(source_path).unwrap();
    let path_string = source_path.as_ref().display().to_string();

    let settings = Settings::default().with_path(path_string.clone());

    let interpreter = vm::Interpreter::with_init(settings, |_vm| {
        // TODO add native modules
        // vm.add_native_module(name, module)
    });

    interpreter.enter(|vm| {
        let scope = vm.new_scope_with_builtins();
        let compile = vm.compile(&source, vm::compiler::Mode::Exec, path_string);

        match compile {
            Ok(py_code) => match vm.run_code_obj(py_code, scope) {
                Ok(code_result) => {
                    info!("Success: {code_result:?}");
                }
                Err(exception) => {
                    let mut output = String::new();
                    vm.write_exception(&mut output, &exception).unwrap();
                    error!("Syntax error: {output}");
                }
            },
            Err(err) => {
                let exception = vm.new_syntax_error(&err, Some(&source));
                let mut output = String::new();
                vm.write_exception(&mut output, &exception).unwrap();
                error!("Runtime error: {output}");
            }
        }
    });
}

fn main() {
    init_logger();

    let source_path = "src/test.py";
    let python_tread = thread::spawn(move || python_runner(&source_path));

    pollster::block_on(framework::start("demo scene".into()));
    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)

    python_tread.join().unwrap();
}
