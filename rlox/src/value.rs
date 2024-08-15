use std::vec::Vec;
use std::ops::Index;
use std::fmt::{Debug, Formatter, Error};
use std::ops::{Add, Sub, Mul, Div, Neg};

#[derive(Debug, Copy, Clone, PartialEq)]
enum ValueType {
    Bool,
    Nil,
    Num,
}

#[derive(Clone, Copy)]
union ValueUnion {
    b: bool,
    f: f64,
}

impl Debug for ValueUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        unsafe {
            write!(f, "ValueUnion [ byte_format: {:#08x} b: {:?}, f: {:} ]", self.f as u64, self.b, self.f)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Value {
    _type: ValueType,
    _as: ValueUnion,
}

impl Add for Value {
    // This overload only makes sense for numbers
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        assert!(self._type == ValueType::Num
            && rhs._type == ValueType::Num);
        unsafe {
            Value {
                _type: ValueType::Num,
                _as: ValueUnion { f: self._as.f + rhs._as.f }
            }
        }
    }
}

impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        assert!(self._type == ValueType::Num
            && rhs._type == ValueType::Num);
        unsafe {
            Value {
                _type: ValueType::Num,
                _as: ValueUnion { f: self._as.f - rhs._as.f }
            }
        }
    }
}

impl Mul for Value {
    type Output = Value;
    fn mul(self, rhs: Self) -> Self::Output {
        assert!(self._type == ValueType::Num
            && rhs._type == ValueType::Num);
        unsafe {
            Value {
                _type: ValueType::Num,
                _as: ValueUnion { f: self._as.f * rhs._as.f }
            }
        }
    }
}

impl Div for Value {
    type Output = Value;
    fn div(self, rhs: Self) -> Self::Output {
        assert!(self._type == ValueType::Num
            && rhs._type == ValueType::Num);
        unsafe {
            Value {
                _type: ValueType::Num,
                _as: ValueUnion { f: self._as.f / rhs._as.f }
            }
        }
    }
}

impl Neg for Value {
    type Output = Value;
    fn neg(self) -> Self::Output {
        assert_eq!(self._type, ValueType::Num);
        unsafe {
            Value {
                _type: ValueType::Num,
                _as: ValueUnion { f : - self._as.f }
            }
        }
    }
}

impl Value {
    pub fn blank() -> Self{
        Value {
            _type: ValueType::Nil,
            _as: ValueUnion { f: 0.0 }
        }
    }
    pub fn bool_val(value: f64) -> Self  {
        Value {
            _type: ValueType::Bool,
            _as: ValueUnion { b: value > 0.0 }
        }
    }

    pub fn nil_val() -> Self {
        Value {
            _type: ValueType::Nil,
            _as: ValueUnion { f: 0.0 }
        }
    }

    pub fn num_val(val: f64) -> Self {
        Value {
            _type: ValueType::Num,
            _as: ValueUnion { f: val }
        }
    }

    pub fn as_bool(&self) -> bool {
        // TODO: is there a nicer way of doing this without unsafe
        unsafe {
            self._as.b
        }
    }

    pub fn as_num(&self) -> f64 {
        unsafe {
            self._as.f
        }
    }

    pub fn is_bool(&self) -> bool { self._type == ValueType::Bool }
    pub fn is_num(&self) -> bool { self._type == ValueType::Num }
    pub fn is_nil(&self) -> bool { self._type == ValueType::Nil }
}


pub fn print_value(value: Value) -> () {
    println!("{:?}", value)
}

// Constants are stored in a value array,
// and we access said constants by being aware
// of their index
#[derive(Clone, Debug)]
pub struct ValueArray{
    values: Vec<Value>,
}

impl ValueArray {
    pub fn init() -> Self {
        ValueArray {
            values: Vec::new(),
        }
    }

    pub fn write_value(&mut self, value: Value) -> ()  {
        self.values.push(value);
    }

    pub fn count(&self) -> usize {
        self.values.len()
    }

    pub fn get_val(&self, index: usize) -> Value{
        self.values[index]
    }

}

impl Index<usize> for ValueArray {
    type Output = Value;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}
