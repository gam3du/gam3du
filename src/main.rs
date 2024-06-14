#![warn(clippy::all, clippy::pedantic)]
// TODO re-enable this later and review all occurrences
#![allow(clippy::cast_precision_loss)]

// TODO enable hand-picked clippy lints from the `restriction` group

mod cube;
mod framework;

use std::{fs::read_to_string, thread};

use cube::RotatingCube;
use rustpython_vm::{self as vm, Settings};

fn main() {
    let python_tread = thread::spawn(|| {
        let source_path = "src/test.py";
        let source = read_to_string(source_path).unwrap();
        let py_result: vm::PyResult<()> = vm::Interpreter::without_stdlib(Settings::default())
            .enter(|vm| {
                let scope = vm.new_scope_with_builtins();
                let code_obj = vm
                    .compile(&source, vm::compiler::Mode::Exec, source_path.into())
                    .map_err(|err| vm.new_syntax_error(&err, Some(&source)))?;

                vm.run_code_obj(code_obj, scope)?;

                Ok(())
            });

        py_result.unwrap();
    });

    framework::run::<RotatingCube>("cube");

    python_tread.join().unwrap();
}
