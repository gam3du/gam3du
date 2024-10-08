use crate::Config;
use gam3du_framework::api::{
    ApiDescriptor, FunctionDescriptor, Identifier, ParameterDescriptor, TypeDescriptor, Value,
};
use std::io::{self, Write};

pub fn generate(out: &mut impl Write, api: &ApiDescriptor, config: &Config) -> io::Result<()> {
    // TODO add documentation comments for api

    generate_module(out, api, config)?;

    Ok(())
}

fn generate_module(
    out: &mut impl Write,
    api: &ApiDescriptor,
    config: &Config,
) -> Result<(), io::Error> {
    if config.sync {
        writeln!(out, "import robot_api_async")?;
        writeln!(out, "import asyncio")?;
    } else {
        writeln!(out, "import asyncio")?;
        writeln!(out, "import robot_api_internal")?;
    }
    writeln!(out)?;
    api.functions
        .values()
        .try_for_each(|function| generate_function(out, function, config))?;
    Ok(())
}

pub fn generate_function(
    out: &mut impl Write,
    function: &FunctionDescriptor,
    config: &Config,
) -> io::Result<()> {
    let FunctionDescriptor {
        ref name,
        caption: _,
        description: _,
        ref parameters,
        ref returns,
    } = *function;

    // TODO add documentation comments for function and parameters

    if !config.sync {
        write!(out, "async ")?;
    }
    write!(out, "def {name}(", name = identifier(name))?;

    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            write!(out, ", ")?;
        }
        generate_parameter(out, parameter, false)?;
    }

    write!(out, ")")?;

    if let Some(ref returns) = *returns {
        write!(out, " -> {typ}", typ = typ(&returns.typ))?;
    }
    writeln!(out, ":")?;

    if config.sync {
        write!(
            out,
            "\tfuture = robot_api_async.{name}(",
            name = identifier(name)
        )?;
        let mut first = true;
        for parameter in parameters {
            if !first {
                write!(out, ", ")?;
            }
            first = false;
            generate_parameter(out, parameter, true)?;
        }
        writeln!(out, ")")?;
        writeln!(out, "\treturn asyncio.run(future)")?;
    } else {
        write!(out, "\thandle = robot_api_internal.message(\"{name}\"")?;
        for parameter in parameters {
            write!(out, ", ")?;
            generate_parameter(out, parameter, true)?;
        }
        writeln!(out, ")")?;
        writeln!(
            out,
            "\twhile True:
		result = robot_api_internal.poll(handle)
		if result.is_done():
			return result.get_value()
		await asyncio.sleep(0.01)
"
        )?;
    }

    writeln!(out)?;

    Ok(())
}

pub fn generate_parameter(
    out: &mut impl Write,
    parameter: &ParameterDescriptor,
    name_only: bool,
) -> io::Result<()> {
    let ParameterDescriptor {
        ref name,
        caption: _,
        description: _,
        ref typ,
        default: _,
    } = *parameter;

    write!(out, "{name}", name = identifier(name))?;

    // If the caller only needs the name of the parameter, we are done.
    if name_only {
        return Ok(());
    }

    let typ = self::typ(typ);
    write!(out, ": {typ}")?;

    if let Some(ref default) = parameter.default {
        match *default {
            Value::Unit => {}
            Value::Integer(default) => write!(out, " = {default}")?,
            Value::Float(default) => write!(out, " = {default}")?,
            Value::Boolean(true) => write!(out, " = True")?,
            Value::Boolean(false) => write!(out, " = False")?,
            Value::String(ref default) => write!(out, " = {default:?}")?,
            Value::List(ref default) => match **default {
                Value::Unit => {}
                Value::Integer(default) => write!(out, " = {default}")?,
                Value::Float(default) => write!(out, " = {default}")?,
                Value::Boolean(true) => write!(out, " = True")?,
                Value::Boolean(false) => write!(out, " = False")?,
                Value::String(ref default) => write!(out, " = {default:?}")?,
                Value::List(_) => unreachable!("3D lists are not supported"),
            },
        }
    }

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
