use std::io::{self, Write};

use crate::api::{Api, FunctionDescriptor, Identifier, ParameterDescriptor, TypeDescriptor};

pub fn generate(out: &mut impl Write, api: &Api) -> io::Result<()> {
    // TODO add documentation comments for api

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
        ref returns,
    } = *function;

    // TODO add documentation comments for function and parameters

    write!(out, "def {name}(", name = identifier(name))?;

    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            write!(out, ", ")?;
        }
        generate_parameter(out, parameter)?;
    }

    write!(out, ")")?;

    if let Some(ref returns) = *returns {
        write!(out, " -> {typ}", typ = typ(&returns.typ))?;
    }
    writeln!(out, ":")?;

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

#[must_use]
pub fn typ(descriptor: &TypeDescriptor) -> String {
    match *descriptor {
        TypeDescriptor::Integer(_) => "int".into(),
        TypeDescriptor::Float => "float".into(),
        TypeDescriptor::Boolean => "bool".into(),
        TypeDescriptor::String => "str".into(),
        TypeDescriptor::List(ref element_type) => format!("list[{}]", typ(element_type)),
    }
}
