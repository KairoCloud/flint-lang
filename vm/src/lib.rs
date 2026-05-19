pub mod bytecode;
pub mod compiler;
pub mod vm;
pub mod gc;
pub mod value;
pub mod frame;

pub use bytecode::{Instruction, Opcode, BytecodeModule};
pub use compiler::compile;
pub use vm::VM;
pub use gc::GarbageCollector;
pub use value::Value;
pub use frame::Frame;