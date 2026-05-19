pub mod llvm;
pub mod wasm;

pub use llvm::{LLVMBackend, LLVMModule, generate_llvm_ir};
pub use wasm::{WasmBackend, compile_to_wasm, target_wasm32};