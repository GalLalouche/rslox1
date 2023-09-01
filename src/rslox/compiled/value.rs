use std::borrow::BorrowMut;
use std::convert::TryFrom;
use std::ops::Deref;

use crate::rslox::common::utils::RcRc;
use crate::rslox::compiled::chunk::{Chunk, InternedString};
use crate::rslox::compiled::gc::{GcWeak, GcWeakMut};
use crate::rslox::compiled::op_code::StackLocation;
use crate::rslox::compiled::tests::DeepEq;

#[derive(Debug, Clone)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Nil,
    String(InternedString),
    Closure(GcWeak<Function>, RcRc<Vec<GcWeakMut<Value>>>),
    UpvaluePtr(GcWeakMut<Value>),
    OpenUpvalue(RcRc<Value>),
}


#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: InternedString,
    pub arity: usize,
    pub chunk: Chunk,
}

#[derive(Debug, Clone, PartialEq, Eq, Copy, Hash)]
pub struct Upvalue {
    pub index: StackLocation,
    pub is_local: bool,
}

impl Function {
    pub fn stringify(&self) -> String { format!("<fn {}>", self.name.to_owned()) }
}

impl DeepEq for Function {
    fn deep_eq(&self, other: &Self) -> bool {
        self.name.to_owned() == other.name.to_owned()
            && self.arity == other.arity
            && self.chunk.deep_eq(&other.chunk)
    }
}

impl PartialEq<Self> for Value {
    fn eq(&self, other: &Self) -> bool {
        match (&self, &other) {
            (Value::Number(n1), Value::Number(n2)) => n1 == n2,
            (Value::Bool(b1), Value::Bool(b2)) => b1 == b2,
            (Value::Nil, Value::Nil) => true,
            (Value::String(s1), Value::String(s2)) => s1 == s2,
            _ => false,
        }
    }
}

impl Value {
    pub fn is_string(&self) -> bool {
        match &self {
            Value::String(_) => true,
            _ => false,
        }
    }
    pub fn is_function(&self) -> bool {
        match &self {
            Value::Closure(..) => true,
            _ => false,
        }
    }

    pub fn stringify(&self) -> String {
        match self {
            Value::Number(f) => f.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Nil => "nil".to_owned(),
            Value::String(s) => s.unwrap_upgrade().to_string(),
            Value::Closure(f, _) => f.unwrap_upgrade().stringify(),
            Value::UpvaluePtr(value) => value.unwrap_upgrade().borrow().stringify(),
            Value::OpenUpvalue(value) => value.borrow().stringify(),
        }
    }
    pub fn is_truthy(&self) -> bool {
        !self.is_falsey()
    }
    pub fn is_falsey(&self) -> bool {
        match &self {
            Value::Nil => true,
            Value::Bool(false) => true,
            _ => false,
        }
    }
    pub fn is_upvalue_ptr(&self) -> bool {
        match self {
            Value::UpvaluePtr(_) => true,
            _ => false,
        }
    }
    pub fn upvalue_ptr(value: GcWeakMut<Value>) -> Self {
        assert!(!value.unwrap_upgrade().borrow().is_upvalue_ptr());
        Value::UpvaluePtr(value)
    }
    /** Returns true if succeeded. */
    #[must_use]
    pub fn update_number(&mut self, n: f64) -> bool {
        match self {
            v @ Value::Number(_) => {
                let _ = std::mem::replace(v, Value::Number(n));
                true
            }
            Value::UpvaluePtr(v) =>
                v.unwrap_upgrade().deref().borrow_mut().update_number(n),
            _ => false
        }
    }
}

impl TryFrom<&Value> for f64 {
    type Error = String;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match &value {
            Value::Number(f) => Ok(*f),
            Value::UpvaluePtr(v) => Self::try_from(v.unwrap_upgrade().borrow().deref()),
            e => Err(format!("Expected Value::Number, but found {:?}", e)),
        }
    }
}

impl<'a> TryFrom<&'a mut Value> for &'a mut bool {
    type Error = String;

    fn try_from(value: &'a mut Value) -> Result<Self, Self::Error> {
        match value.borrow_mut() {
            Value::Bool(b) => Ok(b),
            e => Err(format!("Expected Value::Bool, but found {:?}", e)),
        }
    }
}

impl<'a> TryFrom<&'a Value> for &'a bool {
    type Error = String;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match &value {
            Value::Bool(b) => Ok(&b),
            e => Err(format!("Expected Value::Bool, but found {:?}", e)),
        }
    }
}

impl<'a> TryFrom<&'a Value> for InternedString {
    type Error = String;

    fn try_from(value: &'a Value) -> Result<Self, Self::Error> {
        match &value {
            Value::String(s) => Ok(s.clone()),
            e => Err(format!("Expected Value::String, but found {:?}", e)),
        }
    }
}

impl InternedString {
    pub fn to_owned(&self) -> String { self.unwrap_upgrade().deref().clone() }
}
