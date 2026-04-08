use std::collections::HashMap;

use crate::{
    instruction::Instruction,
    object::{IntoNyaType, Nil, NyaHeapObject, NyaHeapType, NyaPrimitiveType},
};

/// Convert relative index `idx` into absolute index using `len` as reference
fn calc_idx(len: usize, idx: isize) -> usize {
    (if idx < 0 { len as isize + idx } else { idx } as usize)
}

/// This type holds the state of the virtual machine
pub struct NyaState {
    stack: Vec<NyaPrimitiveType>,
    heap: Vec<NyaHeapObject>,
    globals: HashMap<String, NyaPrimitiveType>,
}

impl NyaState {
    /// Create a new NyaState
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            heap: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn run_instructions(&mut self, instructions: &[Instruction]) {
        let mut pc: usize = 0;
        'exec: while pc < instructions.len() {
            match instructions[pc] {
                Instruction::PushInt(int) => self.push_value(int),
                Instruction::PushFloat(float) => self.push_value(float),
                // yk somewhat funny, i never actually used pop instruction in any of my languages
                Instruction::Pop => self.pop_stack(1),
                Instruction::Add => {
                    if let Some(a) = self.pop_stack_and_take() {
                        if let Some(b) = self.pop_stack_and_take() {
                            match a {
                                NyaPrimitiveType::Number(a) => match b {
                                    NyaPrimitiveType::Number(b) => {
                                        self.pop_stack(2);
                                        self.push_value(a + b);
                                    }
                                    _ => panic!("invalid second value for addition"),
                                },
                                NyaPrimitiveType::Int(a) => match b {
                                    NyaPrimitiveType::Int(b) => {
                                        self.pop_stack(2);
                                        self.push_value(a + b);
                                    }
                                    _ => panic!("invalid second value for addition"),
                                },
                                _ => panic!("invalid type for addition"),
                            }
                        } else {
                            panic!("Not enough values on stack");
                        }
                    } else {
                        panic!("Not enough values on stack");
                    }
                }
                // this is relative to make functions more portable, but the actual approach could vary
                Instruction::Jump(offset) => pc += offset,
                Instruction::SetGlobal => {
                    if let Some(name) = self.pop_stack_and_take() {
                        if let Some(val) = self.pop_stack_and_take() {
                            let var_name = match name {
                                // this is where you would get the string value from heap object but idk how
                                // or if you have decided how to do strings
                                NyaPrimitiveType::HeapRef(_obj) => "cool_var",
                                _ => panic!("expected string on stack for global variable name"),
                            };
                            self.set_global(var_name, val);
                        }
                    }
                }
                Instruction::GetGlobal => {
                    if let Some(name) = self.pop_stack_and_take() {
                        let var_name = match name {
                            // this is where you would get the string value from heap object but idk how
                            // or if you have decided how to do strings
                            NyaPrimitiveType::HeapRef(_obj) => "cool_var",
                            _ => panic!("expected string on stack for global variable name"),
                        };
                        self.get_global(var_name)
                    }
                }
                Instruction::Halt => break 'exec,
                Instruction::Print => {
                    if let Some(val) = self.pop_stack_and_take() {
                        match val {
                            NyaPrimitiveType::HeapRef(_nya_heap_object) => panic!(
                                "objects would need to either implement 'to string' or print addr"
                            ),
                            NyaPrimitiveType::Number(v) => println!("{}", v),
                            NyaPrimitiveType::Int(v) => println!("{}", v),
                            NyaPrimitiveType::Nil => print!("nil"),
                        }
                    }
                }
                Instruction::CollectGarbage => self.garbage_collect(),
                Instruction::Equal => {
                    if let Some(a) = self.pop_stack_and_take() {
                        if let Some(b) = self.pop_stack_and_take() {
                            match a {
                                NyaPrimitiveType::Number(a) => match b {
                                    NyaPrimitiveType::Number(b) => {
                                        self.push_value(a == b);
                                    }
                                    _ => panic!("invalid second value for comparison"),
                                },
                                NyaPrimitiveType::Int(a) => match b {
                                    NyaPrimitiveType::Int(b) => {
                                        self.push_value(a == b);
                                    }
                                    _ => panic!("invalid second value for comparison"),
                                },
                                _ => panic!("invalid type for comparison"),
                            }
                        } else {
                            panic!("Not enough values on stack");
                        }
                    } else {
                        panic!("Not enough values on stack");
                    }
                }
                Instruction::Not => {
                    if let Some(a) = self.pop_stack_and_take() {
                        match a {
                            NyaPrimitiveType::Int(a) => {
                                self.push_value(if a == 0 { 1 } else { 0 });
                            }
                            _ => panic!("invalid type for boolean not"),
                        }
                    } else {
                        panic!("Not enough values on stack");
                    }
                }
                Instruction::JumpIf(offset) => {
                    if let Some(a) = self.pop_stack_and_take() {
                        match a {
                            NyaPrimitiveType::Int(a) => {
                                if a != 0 {
                                    pc += offset
                                }
                            }
                            _ => panic!("invalid type for condition check"),
                        }
                    } else {
                        panic!("Not enough values on stack");
                    }
                }
            }
            pc += 1
        }
    }

    // fetching data

    pub fn get_number(&self, idx: isize) -> Option<f64> {
        if let Some(NyaPrimitiveType::Number(number)) = self.get_stack(idx) {
            Some(*number)
        } else {
            None
        }
    }

    pub fn get_number_mut(&mut self, idx: isize) -> Option<&mut f64> {
        if let Some(NyaPrimitiveType::Number(number)) = self.get_stack_mut(idx) {
            Some(number)
        } else {
            None
        }
    }

    pub fn get_int(&self, idx: isize) -> Option<i64> {
        if let Some(NyaPrimitiveType::Int(i)) = self.get_stack(idx) {
            Some(*i)
        } else {
            None
        }
    }

    pub fn get_int_mut(&mut self, idx: isize) -> Option<&mut i64> {
        if let Some(NyaPrimitiveType::Int(i)) = self.get_stack_mut(idx) {
            Some(i)
        } else {
            None
        }
    }

    pub fn get_string(&self, idx: isize) -> Option<&str> {
        if let Some(NyaPrimitiveType::HeapRef(heap_obj)) = self.get_stack(idx)
            && let NyaHeapType::String(s) = &***heap_obj
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn get_string_mut(&mut self, idx: isize) -> Option<&mut String> {
        if let Some(NyaPrimitiveType::HeapRef(heap_obj)) = self.get_stack_mut(idx)
            && let NyaHeapType::String(s) = &mut ***heap_obj
        {
            Some(s)
        } else {
            None
        }
    }

    pub fn get_index(&mut self, stack_idx: isize, idx: isize) {
        if let Some(NyaPrimitiveType::HeapRef(heap_obj)) = self.get_stack(stack_idx)
            && let NyaHeapType::Array(array) = &***heap_obj
            && let Some(obj) = array.get(calc_idx(array.len(), idx))
        {
            self.push_stack_object(*obj);
        } else {
            self.push_value(Nil);
        }
    }

    // pub fn set_index(&mut self, stack_idx: isize, idx: isize) {
    //     if let Some(NyaPrimativeType::HeapRef(heap_obj)) = self.get_stack_mut(stack_idx)
    //         && let NyaHeapType::Array(array) = &mut ***heap_obj
    //     {
    //         if let Some(obj) = self.pop_stack_and_take() {
    //             array.push(obj);
    //         } else {
    //             array.push(Nil.into());
    //         }
    //     }
    // }

    // memory

    /// Allocate an object on the gc heap. If it is not in a root
    pub fn alloc_heap_object(&mut self, obj: NyaHeapType) -> NyaHeapObject {
        let heap_obj = unsafe { NyaHeapObject::new(obj) };
        self.heap.push(heap_obj);
        heap_obj
    }

    /// Get value at relative index `idx` where negative index is subtracted from length of the array
    fn get_stack(&self, idx: isize) -> Option<&NyaPrimitiveType> {
        self.stack.get(calc_idx(self.stack.len(), idx))
    }

    fn get_stack_mut(&mut self, idx: isize) -> Option<&mut NyaPrimitiveType> {
        let idx = calc_idx(self.stack.len(), idx);
        self.stack.get_mut(idx)
    }

    fn push_stack_object(&mut self, obj: NyaPrimitiveType) {
        self.stack.push(obj);
    }

    pub fn push_value<T>(&mut self, object: T)
    where
        T: IntoNyaType,
    {
        let obj = object.into_nya_object(self);
        self.push_stack_object(obj);
    }

    /// Pop value from top of the stack and return it
    fn pop_stack_and_take(&mut self) -> Option<NyaPrimitiveType> {
        self.stack.pop()
    }

    /// Pop `n`` values from top of the stack and discard them
    pub fn pop_stack(&mut self, n: usize) {
        for _ in 0..n {
            self.pop_stack_and_take();
        }
    }

    pub fn set_global<T>(&mut self, name: &str, object: T)
    where
        T: IntoNyaType,
    {
        let obj = object.into_nya_object(self);
        self.globals.insert(name.to_string(), obj);
    }

    pub fn remove_global(&mut self, name: &str) {
        self.globals.remove(name);
    }

    pub fn get_global(&mut self, name: &str) {
        self.push_stack_object(
            self.globals
                .get(name)
                .map_or(NyaPrimitiveType::Nil, |obj| *obj),
        );
    }

    pub fn pop_global(&mut self, name: &str) {
        let obj = self
            .pop_stack_and_take()
            .map_or(NyaPrimitiveType::Nil, |obj| obj);
        self.set_global(name, obj);
    }

    pub fn garbage_collect(&mut self) {
        for obj in &mut self.heap {
            obj.marked = false;
        }

        for obj in &mut self.stack {
            if let NyaPrimitiveType::HeapRef(obj) = obj {
                obj.marked = true;
                obj.mark_children();
            }
        }

        for obj in self.globals.values_mut() {
            if let NyaPrimitiveType::HeapRef(obj) = obj {
                obj.marked = true;
                obj.mark_children();
            }
        }

        for i in (0..self.heap.len()).rev() {
            if !self.heap[i].marked {
                let obj = self.heap.swap_remove(i);
                println!("freed {:?}", **obj);
                unsafe {
                    obj.free();
                }
            }
        }
    }
}

impl Drop for NyaState {
    fn drop(&mut self) {
        for obj in &self.heap {
            unsafe {
                obj.free();
            }
        }
    }
}
