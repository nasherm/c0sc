use std::vec::Vec;
use crate::value::{ValueArray, Value};

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum OpCode{
    Byte(u8), // The byte here represents the index in the ValueArray
    OpConstant,
    OpReturn,
    OpNegate,
    OpAdd,
    OpSub,
    OpMult,
    OpDiv,
}

// Chunks define our bytecode to execute in a
// VM. This wraps a certain state within the context
// of execution
#[derive(Clone, Debug)]
pub struct Chunk {
    code: Vec<OpCode>,
    value_array: ValueArray,
    lines: Vec<u32>,
}

impl Chunk {
    pub fn init() -> Self {
        Chunk{
            code: Vec::new(),
            value_array: ValueArray::init(),
            lines: Vec::new(),
        }
    }

    pub fn write_chunk(&mut self, op: OpCode, line: u32) -> () {
        self.code.push(op);
        self.lines.push(line);
    }

    pub fn compare_code(&self, expected: &Vec<OpCode>, expected_value_array: &[f64]) -> (){
        assert_eq!(expected.len(), self.code.len());

        for i in 0..expected.len() {
            assert_eq!(expected[i], self.code[i]);
            match expected[i] {
                OpCode::Byte(value_array_index) => {
                    assert!(value_array_index < self.value_array.count() as u8);
                    assert_eq!(
                        self.value_array[value_array_index as usize].as_num(),
                        expected_value_array[value_array_index as usize]);
                    },
                    _ => (),
            }
        }
    }

    pub fn count(&self) -> usize {
        self.code.len()
    }

    pub fn get_instr(&self, offset: usize) -> OpCode {
        return self.code[offset]
    }

    pub fn add_constant(&mut self, value: Value) -> usize{
        self.value_array.write_value(value);
        println!("{:?}", self.value_array);
        return self.value_array.count() - 1;
    }

    pub fn get_constant_val(&self, opcode: OpCode) -> Value {
        let index: u8 = match opcode {
            OpCode::Byte(x) => x,
            _ => {
                // Trigger exception
                assert_eq!(true, false);
                0xff
            }
        };
        self.value_array.get_val(index as usize)
    }

    pub fn get_line(&self, offset: usize) -> u32 {
        self.lines[offset]
    }
}
