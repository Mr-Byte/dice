mod helper;

use crate::{module::ModuleLoader, runtime::Runtime, stack::CallFrame};
use dice_core::{
    bytecode::{instruction::Instruction, Bytecode, BytecodeCursor},
    protocol::operator::{
        ADD, DICE_ROLL, DIE_ROLL, DIV, EQ, GT, GTE, LT, LTE, MUL, NEQ, RANGE_EXCLUSIVE, RANGE_INCLUSIVE, REM, SUB,
    },
    upvalue::{Upvalue, UpvalueState},
    value::{Class, FnClosure, Object, Value, ValueKind},
};
use dice_error::runtime_error::RuntimeError;
use std::collections::hash_map::Entry;

impl<L> Runtime<L>
where
    L: ModuleLoader,
{
    pub(super) fn execute_bytecode(
        &mut self,
        bytecode: &Bytecode,
        call_frame: CallFrame,
        parent_upvalues: Option<&[Upvalue]>,
    ) -> Result<Value, RuntimeError> {
        let initial_stack_depth = self.stack.len();
        let mut cursor = bytecode.cursor();

        while let Some(instruction) = cursor.read_instruction() {
            match instruction {
                Instruction::PUSH_NULL => self.stack.push(Value::Null),
                Instruction::PUSH_UNIT => self.stack.push(Value::Unit),
                Instruction::PUSH_FALSE => self.stack.push(Value::Bool(false)),
                Instruction::PUSH_TRUE => self.stack.push(Value::Bool(true)),
                Instruction::PUSH_I0 => self.stack.push(Value::Int(0)),
                Instruction::PUSH_I1 => self.stack.push(Value::Int(1)),
                Instruction::PUSH_F0 => self.stack.push(Value::Float(0.0)),
                Instruction::PUSH_F1 => self.stack.push(Value::Float(1.0)),
                Instruction::PUSH_CONST => self.push_const(bytecode, &mut cursor),
                Instruction::POP => std::mem::drop(self.stack.pop()),
                Instruction::SWAP => self.stack.swap(),
                Instruction::DUP => self.dup(&mut cursor),
                Instruction::CREATE_ARRAY => self.create_list(&mut cursor),
                Instruction::CREATE_OBJECT => self.create_object(),
                Instruction::CREATE_CLASS => self.create_class(&bytecode, &mut cursor)?,
                Instruction::INHERIT_CLASS => self.inherit_class(&bytecode, &mut cursor)?,
                Instruction::CREATE_CLOSURE => {
                    self.create_closure(bytecode, call_frame, parent_upvalues, &mut cursor)?
                }
                Instruction::NEG => self.neg()?,
                Instruction::NOT => self.not()?,
                Instruction::DIE_ROLL => self.die_roll()?,
                Instruction::MUL => self.mul()?,
                Instruction::DIV => self.div()?,
                Instruction::REM => self.rem()?,
                Instruction::ADD => self.add()?,
                Instruction::SUB => self.sub()?,
                Instruction::GT => self.gt()?,
                Instruction::GTE => self.gte()?,
                Instruction::LT => self.lt()?,
                Instruction::LTE => self.lte()?,
                Instruction::EQ => self.eq()?,
                Instruction::NEQ => self.neq()?,
                Instruction::IS => self.is()?,
                Instruction::DICE_ROLL => self.dice_roll()?,
                Instruction::RANGE_EXCLUSIVE => self.range_exclusive()?,
                Instruction::RANGE_INCLUSIVE => self.range_inclusive()?,
                Instruction::JUMP => self.jump(&mut cursor),
                Instruction::JUMP_IF_FALSE => self.jump_if_false(&mut cursor)?,
                Instruction::JUMP_IF_TRUE => self.jump_if_true(&mut cursor)?,
                Instruction::LOAD_LOCAL => self.load_local(call_frame, &mut cursor),
                Instruction::STORE_LOCAL => self.store_local(call_frame, &mut cursor),
                Instruction::ASSIGN_LOCAL => self.assign_local(call_frame, &mut cursor),
                Instruction::LOAD_UPVALUE => self.load_upvalue(parent_upvalues, &mut cursor),
                Instruction::STORE_UPVALUE => self.store_upvalue(parent_upvalues, &mut cursor),
                Instruction::ASSIGN_UPVALUE => self.assign_upvalue(parent_upvalues, &mut cursor),
                Instruction::CLOSE_UPVALUE => self.close_upvalue(call_frame, &mut cursor),
                Instruction::LOAD_GLOBAL => self.load_global(bytecode, &mut cursor)?,
                Instruction::STORE_GLOBAL => self.store_global(bytecode, &mut cursor)?,
                Instruction::LOAD_FIELD => self.load_field(bytecode, &mut cursor)?,
                Instruction::STORE_FIELD => self.store_field(bytecode, &mut cursor)?,
                Instruction::ASSIGN_FIELD => self.assign_field(bytecode, &mut cursor)?,
                Instruction::LOAD_INDEX => self.load_index()?,
                Instruction::STORE_INDEX => self.store_index()?,
                Instruction::ASSIGN_INDEX => self.assign_index()?,
                Instruction::STORE_METHOD => self.store_method(bytecode, &mut cursor)?,
                Instruction::LOAD_FIELD_TO_LOCAL => self.load_field_to_local(bytecode, call_frame, &mut cursor)?,
                Instruction::CALL => self.call(&mut cursor)?,
                Instruction::ASSERT_BOOL => self.assert_bool()?,
                Instruction::ASSERT_TYPE_FOR_LOCAL => self.assert_type_for_local(call_frame, &mut cursor)?,
                Instruction::ASSERT_TYPE_OR_NULL_FOR_LOCAL => {
                    self.assert_type_or_null_for_local(call_frame, &mut cursor)?
                }
                Instruction::ASSERT_TYPE_AND_RETURN => {
                    self.assert_type_and_return()?;
                    break;
                }
                Instruction::ASSERT_TYPE_OR_NULL_AND_RETURN => {
                    self.assert_type_or_null_and_return()?;
                    break;
                }
                Instruction::LOAD_MODULE => self.load_module(&bytecode, &mut cursor)?,
                Instruction::RETURN => break,
                unknown => return Err(RuntimeError::UnknownInstruction(unknown.value())),
            }
        }

        // NOTE: subtract 1 to compensate for the last item of the stack not yet being popped.
        let final_stack_depth = self.stack.len() - 1;

        debug_assert_eq!(
            initial_stack_depth, final_stack_depth,
            "Stack was left in a bad state. Initial depth {}, final depth {}",
            initial_stack_depth, final_stack_depth
        );

        Ok(self.stack.pop())
    }

    fn jump(&mut self, cursor: &mut BytecodeCursor) {
        let offset = cursor.read_offset();
        cursor.offset_position(offset);
    }

    fn dup(&mut self, cursor: &mut BytecodeCursor) {
        let value = self.stack.peek_mut(cursor.read_u8() as usize).clone();
        self.stack.push(value);
    }

    fn assert_bool(&mut self) -> Result<(), RuntimeError> {
        if self.stack.peek_mut(0).kind() != ValueKind::Bool {
            return Err(RuntimeError::Aborted(String::from("Value must evaluate to a boolean.")));
        }

        Ok(())
    }

    fn assert_type_for_local(
        &mut self,
        call_frame: CallFrame,
        cursor: &mut BytecodeCursor,
    ) -> Result<(), RuntimeError> {
        let class = self.stack.pop();
        let class = class.as_class()?;
        let value = &self.stack[call_frame][cursor.read_u8() as usize];

        if *value == Value::Null {
            return Err(RuntimeError::Aborted(String::from("Value cannot be null.")));
        }

        let is_type = value
            .as_object()
            .ok()
            .and_then(|object| object.class())
            .or_else(|| self.value_class_mapping.get(&value.kind()).cloned())
            .map_or(false, |local_class| local_class.is_class(&class));

        if is_type {
            Ok(())
        } else {
            // TODO: Create a more contextual error.
            Err(RuntimeError::Aborted(String::from("Types did not match.")))
        }
    }

    fn assert_type_or_null_for_local(
        &mut self,
        call_frame: CallFrame,
        cursor: &mut BytecodeCursor,
    ) -> Result<(), RuntimeError> {
        let class = self.stack.pop();
        let class = class.as_class()?;
        let value = &self.stack[call_frame][cursor.read_u8() as usize];

        if *value == Value::Null {
            return Ok(());
        }

        let is_type = value
            .as_object()
            .ok()
            .and_then(|object| object.class())
            .or_else(|| self.value_class_mapping.get(&value.kind()).cloned())
            .map_or(false, |local_class| local_class.is_class(&class));

        if is_type {
            Ok(())
        } else {
            // TODO: Create a more contextual error.
            Err(RuntimeError::Aborted(String::from("Types did not match.")))
        }
    }

    fn assert_type_and_return(&mut self) -> Result<(), RuntimeError> {
        let class = self.stack.pop();
        let class = class.as_class()?;
        let value = self.stack.peek(0);

        if *value == Value::Null {
            return Err(RuntimeError::Aborted(String::from("Value cannot be null.")));
        }

        let is_type = value
            .as_object()
            .ok()
            .and_then(|object| object.class())
            .or_else(|| self.value_class_mapping.get(&value.kind()).cloned())
            .map_or(false, |return_class| return_class.is_class(&class));

        if is_type {
            Ok(())
        } else {
            // TODO: Create a more contextual error.
            Err(RuntimeError::Aborted(String::from("Types did not match.")))
        }
    }

    fn assert_type_or_null_and_return(&mut self) -> Result<(), RuntimeError> {
        let class = self.stack.pop();
        let class = class.as_class()?;
        let value = self.stack.peek(0);

        if *value == Value::Null {
            return Ok(());
        }

        let is_type = value
            .as_object()
            .ok()
            .and_then(|object| object.class())
            .or_else(|| self.value_class_mapping.get(&value.kind()).cloned())
            .map_or(false, |return_class| return_class.is_class(&class));

        if is_type {
            Ok(())
        } else {
            // TODO: Create a more contextual error.
            Err(RuntimeError::Aborted(String::from("Types did not match.")))
        }
    }

    fn not(&mut self) -> Result<(), RuntimeError> {
        match self.stack.peek_mut(0) {
            Value::Bool(value) => *value = !*value,
            _ => return Err(RuntimeError::Aborted(String::from("Value must be a boolean."))),
        }

        Ok(())
    }

    fn die_roll(&mut self) -> Result<(), RuntimeError> {
        self.call_unary_op(&DIE_ROLL)
    }

    fn neg(&mut self) -> Result<(), RuntimeError> {
        match self.stack.peek_mut(0) {
            Value::Int(value) => *value = -*value,
            Value::Float(value) => *value = -*value,
            _ => {
                return Err(RuntimeError::Aborted(String::from(
                    "Can only negate an integer or float.",
                )))
            }
        }

        Ok(())
    }

    fn mul(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Int(rhs), Value::Int(lhs)) => *lhs *= rhs,
            (Value::Float(rhs), Value::Float(lhs)) => *lhs *= rhs,
            (rhs, _) => self.call_binary_op(&MUL, rhs)?,
        }

        Ok(())
    }

    fn div(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Int(rhs), Value::Int(lhs)) => {
                if rhs == 0 {
                    return Err(RuntimeError::DivideByZero);
                }

                *lhs /= rhs;
            }
            (Value::Float(rhs), Value::Float(lhs)) => *lhs /= rhs,
            (rhs, _) => self.call_binary_op(&DIV, rhs)?,
        }

        Ok(())
    }

    fn rem(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Int(rhs), Value::Int(lhs)) => {
                if rhs == 0 {
                    return Err(RuntimeError::DivideByZero);
                }

                *lhs %= rhs;
            }
            (Value::Float(rhs), Value::Float(lhs)) => *lhs %= rhs,
            (rhs, _) => self.call_binary_op(&REM, rhs)?,
        }

        Ok(())
    }

    fn add(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Int(rhs), Value::Int(lhs)) => *lhs += rhs,
            (Value::Float(rhs), Value::Float(lhs)) => *lhs += rhs,
            (rhs, _) => self.call_binary_op(&ADD, rhs)?,
        }

        Ok(())
    }

    fn gt(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Bool(rhs), Value::Bool(lhs)) => *lhs &= !rhs,
            (Value::Int(rhs), Value::Int(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs > rhs),
            (Value::Float(rhs), Value::Float(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs > rhs),
            (rhs, _) => self.call_binary_op(&GT, rhs)?,
        }

        Ok(())
    }

    fn gte(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Bool(rhs), Value::Bool(lhs)) => *lhs = *lhs >= rhs,
            (Value::Int(rhs), Value::Int(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs >= rhs),
            (Value::Float(rhs), Value::Float(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs >= rhs),
            (rhs, _) => self.call_binary_op(&GTE, rhs)?,
        }

        Ok(())
    }

    fn lt(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Bool(rhs), Value::Bool(lhs)) => *lhs = !(*lhs) & rhs,
            (Value::Int(rhs), Value::Int(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs < rhs),
            (Value::Float(rhs), Value::Float(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs < rhs),
            (rhs, _) => self.call_binary_op(&LT, rhs)?,
        }

        Ok(())
    }

    fn lte(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Bool(rhs), Value::Bool(lhs)) => *lhs = *lhs <= rhs,
            (Value::Int(rhs), Value::Int(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs <= rhs),
            (Value::Float(rhs), Value::Float(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs <= rhs),
            (rhs, _) => self.call_binary_op(&LTE, rhs)?,
        }

        Ok(())
    }

    fn sub(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Int(rhs), Value::Int(lhs)) => *lhs -= rhs,
            (Value::Float(rhs), Value::Float(lhs)) => *lhs -= rhs,
            (rhs, _) => self.call_binary_op(&SUB, rhs)?,
        }

        Ok(())
    }

    fn eq(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Null, Value::Null) => *self.stack.peek_mut(0) = Value::Bool(true),
            (Value::Null, _) => *self.stack.peek_mut(0) = Value::Bool(false),
            (_, Value::Null) => *self.stack.peek_mut(0) = Value::Bool(false),
            (Value::Unit, Value::Unit) => *self.stack.peek_mut(0) = Value::Bool(true),
            (Value::Unit, _) => *self.stack.peek_mut(0) = Value::Bool(false),
            (_, Value::Unit) => *self.stack.peek_mut(0) = Value::Bool(false),
            (Value::Bool(rhs), Value::Bool(lhs)) => *lhs = *lhs == rhs,
            (Value::Int(rhs), Value::Int(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs == rhs),
            (Value::Float(rhs), Value::Float(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs == rhs),
            (rhs, _) => self.call_binary_op(&EQ, rhs)?,
        }

        Ok(())
    }

    fn neq(&mut self) -> Result<(), RuntimeError> {
        match (self.stack.pop(), self.stack.peek_mut(0)) {
            (Value::Null, Value::Null) => *self.stack.peek_mut(0) = Value::Bool(false),
            (Value::Null, _) => *self.stack.peek_mut(0) = Value::Bool(true),
            (_, Value::Null) => *self.stack.peek_mut(0) = Value::Bool(true),
            (Value::Unit, Value::Unit) => *self.stack.peek_mut(0) = Value::Bool(false),
            (Value::Unit, _) => *self.stack.peek_mut(0) = Value::Bool(true),
            (_, Value::Unit) => *self.stack.peek_mut(0) = Value::Bool(true),
            (Value::Bool(rhs), Value::Bool(lhs)) => *lhs = *lhs != rhs,
            (Value::Int(rhs), Value::Int(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs != rhs),
            (Value::Float(rhs), Value::Float(lhs)) => *self.stack.peek_mut(0) = Value::Bool(*lhs != rhs),
            (rhs, _) => self.call_binary_op(&NEQ, rhs)?,
        }

        Ok(())
    }

    fn dice_roll(&mut self) -> Result<(), RuntimeError> {
        let rhs = self.stack.pop();
        self.call_binary_op(&DICE_ROLL, rhs)
    }

    fn range_inclusive(&mut self) -> Result<(), RuntimeError> {
        let rhs = self.stack.pop();
        self.call_binary_op(&RANGE_INCLUSIVE, rhs)
    }

    fn range_exclusive(&mut self) -> Result<(), RuntimeError> {
        let rhs = self.stack.pop();
        self.call_binary_op(&RANGE_EXCLUSIVE, rhs)
    }

    fn is(&mut self) -> Result<(), RuntimeError> {
        let class = self.stack.pop();
        let class = class.as_class()?;
        let instance = self.stack.peek(0);
        let is_type = instance
            .as_object()
            .ok()
            .and_then(|object| object.class())
            .or_else(|| self.value_class_mapping.get(&instance.kind()).cloned())
            .map_or(false, |instance_class| instance_class.is_class(&class));

        *self.stack.peek_mut(0) = Value::Bool(is_type);

        Ok(())
    }

    fn create_list(&mut self, cursor: &mut BytecodeCursor) {
        let count = cursor.read_u8() as usize;
        let items = self.stack.pop_count(count);

        self.stack.push(Value::Array(items.to_vec().into()));
    }

    fn create_object(&mut self) {
        let object = Object::new(self.any_class.clone());

        self.stack.push(Value::Object(object));
    }

    fn create_class(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let name_slot = cursor.read_u8() as usize;
        let name = bytecode.constants()[name_slot].as_symbol()?;
        let class = Class::with_base(name, self.any_class.clone());

        self.stack.push(Value::Class(class));

        Ok(())
    }

    fn inherit_class(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let name_slot = cursor.read_u8() as usize;
        let name = bytecode.constants()[name_slot].as_symbol()?;
        let class = Class::with_base(name, self.stack.pop().as_class()?);

        self.stack.push(Value::Class(class));

        Ok(())
    }

    fn push_const(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) {
        let const_pos = cursor.read_u8() as usize;
        let value = bytecode.constants()[const_pos].clone();
        self.stack.push(value);
    }

    fn jump_if_false(&mut self, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let offset = cursor.read_offset();
        let value = self.stack.pop().as_bool()?;

        if !value {
            cursor.offset_position(offset)
        }

        Ok(())
    }

    fn jump_if_true(&mut self, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let offset = cursor.read_offset();
        let value = self.stack.pop().as_bool()?;

        if value {
            cursor.offset_position(offset)
        }

        Ok(())
    }

    fn load_local(&mut self, call_frame: CallFrame, cursor: &mut BytecodeCursor) {
        // TODO Bounds check the slot?
        let slot = cursor.read_u8() as usize;
        let frame = &self.stack[call_frame];
        let value = frame[slot].clone();
        self.stack.push(value);
    }

    fn store_local(&mut self, call_frame: CallFrame, cursor: &mut BytecodeCursor) {
        let value = self.stack.pop();
        let slot = cursor.read_u8() as usize;

        self.stack[call_frame][slot] = value.clone();
        self.stack.push(value);
    }

    fn assign_local(&mut self, call_frame: CallFrame, cursor: &mut BytecodeCursor) {
        let value = self.stack.pop();
        let slot = cursor.read_u8() as usize;

        self.stack[call_frame][slot] = value;
        self.stack.push(Value::Unit);
    }

    fn load_upvalue(&mut self, parent_upvalues: Option<&[Upvalue]>, cursor: &mut BytecodeCursor) {
        if let Some(parent_upvalues) = parent_upvalues {
            let upvalue_slot = cursor.read_u8() as usize;
            let upvalue = parent_upvalues[upvalue_slot].clone();
            let value = match &*upvalue.state_mut() {
                UpvalueState::Open(slot) => self.stack[*slot].clone(),
                UpvalueState::Closed(value) => value.clone(),
            };

            self.stack.push(value);
        } else {
            unreachable!("LOAD_UPVALUE used in non-closure context.")
        }
    }

    fn store_upvalue(&mut self, parent_upvalues: Option<&[Upvalue]>, cursor: &mut BytecodeCursor) {
        if let Some(parent_upvalues) = parent_upvalues {
            let upvalue_slot = cursor.read_u8() as usize;
            let upvalue = parent_upvalues[upvalue_slot].clone();
            let value = self.stack.pop();
            let result = match &mut *upvalue.state_mut() {
                UpvalueState::Open(slot) => {
                    self.stack[*slot] = value.clone();
                    value
                }
                UpvalueState::Closed(closed_value) => {
                    *closed_value = value.clone();
                    value
                }
            };

            self.stack.push(result)
        } else {
            unreachable!("STORE_UPVALUE used in non-closure context.")
        }
    }

    fn assign_upvalue(&mut self, parent_upvalues: Option<&[Upvalue]>, cursor: &mut BytecodeCursor) {
        if let Some(parent_upvalues) = parent_upvalues {
            let upvalue_slot = cursor.read_u8() as usize;
            let upvalue = parent_upvalues[upvalue_slot].clone();
            let value = self.stack.pop();
            match &mut *upvalue.state_mut() {
                UpvalueState::Open(slot) => self.stack[*slot] = value,
                UpvalueState::Closed(closed_value) => *closed_value = value,
            };

            self.stack.push(Value::Unit)
        } else {
            unreachable!("ASSIGN_UPVALUE used in non-closure context.")
        }
    }

    fn close_upvalue(&mut self, call_frame: CallFrame, cursor: &mut BytecodeCursor) {
        let offset = cursor.read_u8() as usize;
        let value = std::mem::replace(&mut self.stack[call_frame][offset], Value::Null);
        let offset = call_frame.start() + offset;
        let found_upvalue = self.find_open_upvalue(offset);

        if let Some((index, _)) = found_upvalue {
            if let Some(upvalue) = self.open_upvalues.remove(index) {
                upvalue.close(value);
            }
        }
    }

    fn store_global(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let const_pos = cursor.read_u8() as usize;
        let value = &bytecode.constants()[const_pos];
        let global_name = value.as_symbol()?;
        let global = self.stack.pop();

        match self.globals.entry(global_name) {
            Entry::Occupied(_) => todo!("Return error that global already exists."),
            Entry::Vacant(entry) => {
                entry.insert(global);
            }
        }

        Ok(())
    }

    fn load_global(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let const_pos = cursor.read_u8() as usize;
        let global = bytecode.constants()[const_pos].as_symbol()?;
        let value = self
            .globals
            .get(&global)
            .cloned()
            .ok_or_else(|| RuntimeError::VariableNotFound((&*global).to_owned()))?;

        self.stack.push(value);

        Ok(())
    }

    fn load_field(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let key_index = cursor.read_u8() as usize;
        let key = bytecode.constants()[key_index].as_symbol()?;

        let value = self.stack.pop();
        let value = self.get_field(&key, value)?;

        self.stack.push(value);

        Ok(())
    }

    fn store_field(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let key_index = cursor.read_u8() as usize;
        let key = bytecode.constants()[key_index].as_symbol()?;
        let value = self.stack.pop();
        let object = self.stack.pop();
        let object = object.as_object()?;

        object.set_field(key, value.clone());
        self.stack.push(value);

        Ok(())
    }

    fn assign_field(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let key_index = cursor.read_u8() as usize;
        let key = bytecode.constants()[key_index].as_symbol()?;
        let value = self.stack.pop();
        let object = self.stack.pop();
        let object = object.as_object()?;

        object.set_field(key, value);
        self.stack.push(Value::Unit);

        Ok(())
    }

    fn load_index(&mut self) -> Result<(), RuntimeError> {
        let index = self.stack.pop();
        let target = self.stack.peek(0);
        let result = match target {
            Value::Array(array) if index.kind() == ValueKind::Int => {
                let index = index.as_int()?;
                array.elements().get(index as usize).cloned().unwrap_or(Value::Null)
            }
            target => {
                let field = index.as_symbol()?;
                self.get_field(&field, target.clone())?
            }
        };

        *self.stack.peek_mut(0) = result;

        Ok(())
    }

    fn store_index(&mut self) -> Result<(), RuntimeError> {
        let value = self.stack.pop();
        let index = self.stack.pop();
        let target = self.stack.peek_mut(0);

        match target {
            Value::Array(array) if index.kind() == ValueKind::Int => {
                let index = index.as_int()?;
                array.elements_mut()[index as usize] = value.clone();
                *target = value;
            }
            target => {
                let object = target.as_object()?;
                let field = index.as_symbol()?;
                object.set_field(field, value.clone());
                *target = value;
            }
        };

        Ok(())
    }

    fn assign_index(&mut self) -> Result<(), RuntimeError> {
        let value = self.stack.pop();
        let index = self.stack.pop();
        let target = self.stack.peek_mut(0);

        match target {
            Value::Array(array) if index.kind() == ValueKind::Int => {
                let index = index.as_int()?;
                array.elements_mut()[index as usize] = value;
                *target = Value::Unit;
            }
            target => {
                let object = target.as_object()?;
                let field = index.as_symbol()?;
                object.set_field(field, value);
                *target = Value::Unit;
            }
        };

        Ok(())
    }

    fn store_method(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let key_index = cursor.read_u8() as usize;
        let key = bytecode.constants()[key_index].as_symbol()?;
        let value = self.stack.pop();
        let object = self.stack.pop();
        let class = object.as_class()?;

        class.set_method(key, value);

        Ok(())
    }

    fn load_field_to_local(
        &mut self,
        bytecode: &Bytecode,
        call_frame: CallFrame,
        cursor: &mut BytecodeCursor,
    ) -> Result<(), RuntimeError> {
        let key_index = cursor.read_u8() as usize;
        let local_slot = cursor.read_u8() as usize;
        let key = bytecode.constants()[key_index].as_symbol()?;
        let value = self.stack.pop();
        let value = self.get_field(&key, value)?;

        self.stack[call_frame][local_slot] = value.clone();
        self.stack.push(value);

        Ok(())
    }

    fn create_closure(
        &mut self,
        bytecode: &Bytecode,
        call_frame: CallFrame,
        parent_upvalues: Option<&[Upvalue]>,
        cursor: &mut BytecodeCursor,
    ) -> Result<(), RuntimeError> {
        let const_pos = cursor.read_u8() as usize;

        match bytecode.constants()[const_pos] {
            Value::FnScript(ref fn_script) => {
                let upvalue_count = fn_script.bytecode().upvalue_count();
                let mut upvalues = Vec::with_capacity(upvalue_count);

                for _ in 0..upvalue_count {
                    let is_parent_local = cursor.read_u8() == 1;
                    let index = cursor.read_u8() as usize;

                    if is_parent_local {
                        let offset = call_frame.start() + index;
                        match self.find_open_upvalue(offset) {
                            None => {
                                let upvalue = Upvalue::new_open(call_frame.start() + index);
                                self.open_upvalues.push_back(upvalue.clone());
                                upvalues.push(upvalue);
                            }
                            Some((_, upvalue)) => upvalues.push(upvalue),
                        };
                    } else if let Some(parent_upvalues) = parent_upvalues {
                        let upvalue = parent_upvalues[index].clone();
                        upvalues.push(upvalue);
                    } else {
                        // NOTE: Produce an unreachable here. This case should never execute, but this is a sanity check to ensure it doesn't.
                        unreachable!("No parent scope found.")
                    }
                }

                let closure = Value::FnClosure(FnClosure::new(fn_script.clone(), upvalues.into_boxed_slice()));
                self.stack.push(closure);
            }
            _ => return Err(RuntimeError::NotAFunction),
        }

        Ok(())
    }

    pub fn call(&mut self, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let arg_count = cursor.read_u8() as usize;
        self.call_fn(arg_count)
    }

    fn load_module(&mut self, bytecode: &Bytecode, cursor: &mut BytecodeCursor) -> Result<(), RuntimeError> {
        let module_slot = cursor.read_u8() as usize;
        let module_name = bytecode.constants()[module_slot].as_symbol()?;
        let module = match self.loaded_modules.entry(module_name.clone()) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let export = Value::Object(Object::new(self.module_class.clone()));
                entry.insert(export.clone());

                let module = self.module_loader.load_module(module_name)?;
                self.run_module(module.bytecode, export)?
            }
        };

        self.stack.push(module);

        Ok(())
    }
}
