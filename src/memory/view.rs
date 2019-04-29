use lazy_static::lazy_static;
use rutie::{class, methods, wrappable_struct, Fixnum, Integer, NilClass, Object};
use std::{mem::size_of, rc::Rc};
use wasmer_runtime as runtime;

pub struct MemoryView {
    memory: Rc<runtime::memory::Memory>,
    offset: usize,
}

impl MemoryView {
    pub fn new(memory: Rc<runtime::Memory>, offset: usize) -> Self {
        Self { memory, offset }
    }

    pub fn len(&self) -> usize {
        self.memory.view::<u8>()[self.offset..].len() / size_of::<u8>()
    }

    pub fn set(&self, index: isize, value: u8) -> Result<(), &str> {
        let offset = self.offset;
        let view = self.memory.view::<u8>();

        if index < 0 {
            return Err("foo");
        }

        let index = index as usize;

        if view.len() <= offset + index {
            Err("bar")
        } else {
            view[offset + index].set(value);

            Ok(())
        }
    }

    pub fn get(&self, index: isize) -> Result<u8, &str> {
        let offset = self.offset;
        let view = self.memory.view::<u8>();

        if index < 0 {
            return Err("foo");
        }

        let index = index as usize;

        if view.len() <= offset + index {
            Err("bar")
        } else {
            Ok(view[offset + index].get())
        }
    }
}

wrappable_struct!(MemoryView, MemoryViewWrapper, MEMORY_VIEW_WRAPPER);

class!(RubyMemoryView);

#[rustfmt::skip]
methods!(
    RubyMemoryView,
    itself,

    fn ruby_memory_view_length() -> Fixnum {
        Fixnum::new(itself.get_data(&*MEMORY_VIEW_WRAPPER).len() as i64)
    }

    fn ruby_memory_view_set(index: Integer, value: Integer) -> NilClass {
        let memory_view = itself.get_data(&*MEMORY_VIEW_WRAPPER);
        memory_view.set(index.unwrap().to_i32() as isize, value.unwrap().to_i32() as u8).unwrap();

        NilClass::new()
    }

    fn ruby_memory_view_get(index: Integer) -> Fixnum {
        let memory_view = itself.get_data(&*MEMORY_VIEW_WRAPPER);

        Fixnum::new(memory_view.get(index.unwrap().to_i32() as isize).unwrap() as i64)
    }
);