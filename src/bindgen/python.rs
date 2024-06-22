use std::io::{self, Write};

use crate::api::{Api, FunctionDescriptor, Identifier, ParameterDescriptor};

pub fn generate(out: &mut impl Write, api: &Api) -> io::Result<()> {
    api.functions
        .iter()
        .try_for_each(|function| generate_function(out, function))?;

    Ok(())
}

pub fn generate_function(out: &mut impl Write, function: &FunctionDescriptor) -> io::Result<()> {
    let FunctionDescriptor {
        ref name,
        caption: _,
        description: _,
        ref parameters,
        returns: _,
    } = *function;

    write!(out, "def {name}(", name = identifier(name))?;

    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            write!(out, ", ")?;
        }
        generate_parameter(out, parameter)?;
    }

    writeln!(out, "):")?;
    writeln!(out, "\tpass")?;
    writeln!(out)?;

    Ok(())
}

pub fn generate_parameter(out: &mut impl Write, parameter: &ParameterDescriptor) -> io::Result<()> {
    let ParameterDescriptor {
        ref name,
        caption: _,
        description: _,
        typ: _,
    } = *parameter;

    write!(out, "{name}", name = identifier(name))?;

    Ok(())
}

#[must_use]
pub fn identifier(identifier: &Identifier) -> String {
    // TODO add safeguards against reserved keywords
    identifier.0.replace(' ', "_")
}
