use dice_core::value::Value;
use std::ops::Range;

// NOTE: Allocate 1MB of stack space, this is 65,536 values when sizeof(Value) == 16
const MAX_STACK_SIZE: usize = (1024 * 1024) / std::mem::size_of::<Value>();

#[derive(Debug)]
pub struct Stack {
    values: Vec<Value>,
    stack_ptr: usize,
}

// TODO: Enforce stack overflows and underflows.
impl Stack {
    #[inline]
    pub fn push(&mut self, value: Value) {
        self.values[self.stack_ptr] = value;
        self.stack_ptr += 1;
    }

    pub fn push_slice(&mut self, values: &[Value]) {
        // TODO: Replace this with a more efficient multi-push.
        for value in values {
            self.push(value.clone());
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Value {
        self.stack_ptr -= 1;
        std::mem::replace(&mut self.values[self.stack_ptr], Value::Null)
    }

    pub fn pop_count(&mut self, count: usize) -> Vec<Value> {
        let mut result = vec![Value::Null; count];
        let items_to_pop = &mut self.values[self.stack_ptr - count..self.stack_ptr];
        self.stack_ptr -= count;

        for index in (0..items_to_pop.len()).rev() {
            std::mem::swap(&mut items_to_pop[index], &mut result[index])
        }

        result
    }

    pub fn reserve_slots(&mut self, count: usize) -> Range<usize> {
        let start = self.stack_ptr;
        let new_stack_ptr = self.stack_ptr + count;

        self.stack_ptr = new_stack_ptr;
        assert!(self.stack_ptr < MAX_STACK_SIZE, "Stack Overflowed");

        start..new_stack_ptr
    }

    pub fn release_slots(&mut self, count: usize) {
        let new_stack_ptr = self.stack_ptr - count;
        for value in &mut self.values[new_stack_ptr..self.stack_ptr] {
            *value = Value::Null;
        }

        self.stack_ptr = new_stack_ptr;

        // NOTE: If the stack ptr is greater than the stack size, the stack ptr underflowed.
        assert!(self.stack_ptr < MAX_STACK_SIZE, "Stack Underflowed")
    }

    #[inline]
    pub fn slots(&mut self, slots: Range<usize>) -> &mut [Value] {
        &mut self.values[slots]
    }

    #[inline]
    pub fn slot(&mut self, slot: usize) -> &mut Value {
        &mut self.values[slot]
    }

    // NOTE: Returns the value offset from the top of the stack.
    #[inline]
    pub fn peek_mut(&mut self, offset: usize) -> &mut Value {
        &mut self.values[self.stack_ptr - offset - 1]
    }

    #[inline]
    pub fn peek(&self, offset: usize) -> &Value {
        &self.values[self.stack_ptr - offset - 1]
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.stack_ptr
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self {
            values: vec![Value::Null; MAX_STACK_SIZE],
            stack_ptr: 0,
        }
    }
}
