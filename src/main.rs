use std::fs::read_to_string;

use rustpython_vm as vm;

fn main() -> vm::PyResult<()> {
    let source_path = "src/test.py";
    let source = read_to_string(source_path).unwrap();

    vm::Interpreter::without_stdlib(Default::default()).enter(|vm| {
        let scope = vm.new_scope_with_builtins();
        let code_obj = vm
            .compile(&source, vm::compiler::Mode::Exec, source_path.into())
            .map_err(|err| vm.new_syntax_error(&err, Some(&source)))?;

        vm.run_code_obj(code_obj, scope)?;

        Ok(())
    })
}
