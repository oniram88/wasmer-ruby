//! The `Instance` WebAssembly class.

use crate::memory::{Memory, RubyMemory, MEMORY_WRAPPER};
use lazy_static::lazy_static;
use rutie::{
    class, methods,
    rubysys::{class, value::ValueType},
    types::{Argc, Value},
    util::str_to_cstring,
    wrappable_struct, AnyObject, Array, Class, Fixnum, Float, Object, RString, Symbol,
};
use std::{mem, rc::Rc};
use wasmer_runtime::{self as runtime, imports, Export};
use wasmer_runtime_core::types::Type;

/// The `ExportedFunctions` Ruby class.
pub struct ExportedFunctions {
    /// The WebAssembly runtime.
    instance: Rc<runtime::Instance>,
}

impl ExportedFunctions {
    /// Create a new instance of the `ExportedFunctions` Ruby class.
    pub fn new(instance: Rc<runtime::Instance>) -> Self {
        Self { instance }
    }

    /// Call an exported function on the given WebAssembly instance.
    pub fn method_missing(&self, method_name: &str, arguments: Array) -> AnyObject {
        let function = self.instance.dyn_func(method_name).unwrap();
        let signature = function.signature();
        let parameters = signature.params();
        let number_of_parameters = parameters.len() as isize;
        let number_of_arguments = arguments.length() as isize;
        let diff: isize = number_of_parameters - number_of_arguments;

        if diff > 0 {
            panic!("Missing arguments");
        } else if diff < 0 {
            panic!("Too much arguments");
        }

        let mut function_arguments =
            Vec::<runtime::Value>::with_capacity(number_of_parameters as usize);

        for (parameter, argument) in parameters.iter().zip(arguments.into_iter()) {
            let value = match (parameter, argument.ty()) {
                (Type::I32, ValueType::Fixnum) => {
                    runtime::Value::I32(argument.try_convert_to::<Fixnum>().unwrap().to_i32())
                }
                (Type::I64, ValueType::Fixnum) => {
                    runtime::Value::I64(argument.try_convert_to::<Fixnum>().unwrap().to_i64())
                }
                (Type::F32, ValueType::Float) => {
                    runtime::Value::F32(argument.try_convert_to::<Float>().unwrap().to_f64() as f32)
                }
                (Type::F64, ValueType::Float) => {
                    runtime::Value::F64(argument.try_convert_to::<Float>().unwrap().to_f64())
                }
                _ => panic!("aaahhh"),
            };

            function_arguments.push(value);
        }

        let results = function.call(function_arguments.as_slice()).unwrap();

        match results[0] {
            runtime::Value::I32(result) => Fixnum::new(result as i64).into(),
            runtime::Value::I64(result) => Fixnum::new(result).into(),
            runtime::Value::F32(result) => Float::new(result as f64).into(),
            runtime::Value::F64(result) => Float::new(result).into(),
        }
    }
}

wrappable_struct!(
    ExportedFunctions,
    ExportedFunctionsWrapper,
    EXPORTED_FUNCTIONS_WRAPPER
);

class!(RubyExportedFunctions);

/// Glue code to call the `ExportedFunctions.method_missing` method.
pub extern "C" fn ruby_exported_functions_method_missing(
    argc: Argc,
    argv: *const AnyObject,
    itself: RubyExportedFunctions,
) -> AnyObject {
    let arguments = Value::from(0);

    unsafe {
        let argv_pointer: *const Value = mem::transmute(argv);

        class::rb_scan_args(argc, argv_pointer, str_to_cstring("*").as_ptr(), &arguments)
    };

    let mut arguments = Array::from(arguments);
    let method_name = unsafe { arguments.shift().to::<Symbol>() };
    let method_name = method_name.to_str();

    itself
        .get_data(&*EXPORTED_FUNCTIONS_WRAPPER)
        .method_missing(method_name, arguments)
}

/// The `Instance` Ruby class.
pub struct Instance {
    /// The WebAssembly instance.
    instance: Rc<runtime::Instance>,
}

impl Instance {
    /// Create a new instance of the `Instance` Ruby class.
    /// The constructor receives bytes from a string.
    pub fn new(bytes: &[u8]) -> Self {
        let import_object = imports! {};
        let instance = Rc::new(runtime::instantiate(bytes, &import_object).unwrap());

        Self { instance }
    }
}

wrappable_struct!(Instance, InstanceWrapper, INSTANCE_WRAPPER);

class!(RubyInstance);

#[rustfmt::skip]
methods!(
    RubyInstance,
    _itself,

    // Glue code to call the `Instance.new` method.
    fn ruby_instance_new(bytes: RString) -> AnyObject {
        let instance = Instance::new(bytes.unwrap().to_bytes_unchecked());
        let exported_functions = ExportedFunctions::new(instance.instance.clone());

        let memory = instance
            .instance
            .exports()
            .find_map(|(_, export)| match export {
                Export::Memory(memory) => Some(Memory::new(Rc::new(memory))),
                _ => None,
            })
            .ok_or_else(|| panic!("ahhhhh"))
            .unwrap();

        let mut ruby_instance: AnyObject =
            Class::from_existing("Instance").wrap_data(instance, &*INSTANCE_WRAPPER);

        let ruby_exported_functions: RubyExportedFunctions =
            Class::from_existing("ExportedFunctions")
                .wrap_data(exported_functions, &*EXPORTED_FUNCTIONS_WRAPPER);

        ruby_instance.instance_variable_set("@exports", ruby_exported_functions);

        let ruby_memory: RubyMemory =
            Class::from_existing("Memory").wrap_data(memory, &*MEMORY_WRAPPER);

        ruby_instance.instance_variable_set("@memory", ruby_memory);

        ruby_instance
    }

    // Glue code to call the `Instance.exports` getter method.
    fn ruby_instance_exported_functions() -> RubyExportedFunctions {
        unsafe {
            _itself
                .instance_variable_get("@exports")
                .to::<RubyExportedFunctions>()
        }
    }

    // Glue code to call the `Instance.memory` getter method.
    fn ruby_instance_memory() -> RubyMemory {
        unsafe {
            _itself
                .instance_variable_get("@memory")
                .to::<RubyMemory>()
        }
    }
);