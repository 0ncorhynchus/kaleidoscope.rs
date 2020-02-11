use crate::lexer::Operator;
use crate::parser::{ExprAST, Prototype};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_uint;

use llvm_sys::analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction};
use llvm_sys::core::*;
use llvm_sys::prelude::*;

#[allow(non_camel_case_types)]
type size_t = usize;

#[derive(Debug, PartialEq)]
pub enum LLVMError {
    VariableNotFound(String),
    FunctionNotFound(String),
    InvalidArgumentsSize(String, usize),
}

type Result<T> = std::result::Result<T, LLVMError>;

pub struct LLVMContext {
    inner: LLVMContextRef,
}

impl LLVMContext {
    pub fn new() -> Self {
        Self {
            inner: unsafe { LLVMContextCreate() },
        }
    }

    pub fn create_module(&mut self, name: &str) -> LLVMModule {
        let name = CString::new(name).unwrap();
        LLVMModule {
            inner: unsafe { LLVMModuleCreateWithNameInContext(name.as_ptr(), self.inner) },
        }
    }

    pub fn create_basic_block(&mut self, f: LLVMValueRef) -> LLVMBasicBlockRef {
        let name = CStr::from_bytes_with_nul(b"entry\0").unwrap();
        unsafe { LLVMAppendBasicBlockInContext(self.inner, f, name.as_ptr()) }
    }
}

impl Drop for LLVMContext {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.inner);
        }
    }
}

pub struct LLVMModule {
    inner: LLVMModuleRef,
}

impl LLVMModule {
    pub fn get_function(&mut self, name: &str) -> Result<LLVMValueRef> {
        let c_name = CString::new(name).unwrap();
        let f = unsafe { LLVMGetNamedFunction(self.inner, c_name.as_ptr()) };
        if f.is_null() {
            Err(LLVMError::FunctionNotFound(name.to_string()))
        } else {
            Ok(f)
        }
    }

    pub fn add_function(&mut self, name: &str, ty: LLVMTypeRef) -> LLVMValueRef {
        let name = CString::new(name).unwrap();
        unsafe { LLVMAddFunction(self.inner, name.as_ptr(), ty) }
    }
}

pub struct LLVMBuilder {
    inner: LLVMBuilderRef,
}

impl LLVMBuilder {
    pub fn new(context: &mut LLVMContext) -> Self {
        Self {
            inner: unsafe { LLVMCreateBuilderInContext(context.inner) },
        }
    }

    pub fn create_fadd(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        let name = CStr::from_bytes_with_nul(b"addtmp\0").unwrap();
        unsafe { LLVMBuildFAdd(self.inner, lhs, rhs, name.as_ptr()) }
    }

    pub fn create_fsub(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        let name = CStr::from_bytes_with_nul(b"subtmp\0").unwrap();
        unsafe { LLVMBuildFSub(self.inner, lhs, rhs, name.as_ptr()) }
    }

    pub fn create_fmul(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        let name = CStr::from_bytes_with_nul(b"multmp\0").unwrap();
        unsafe { LLVMBuildFMul(self.inner, lhs, rhs, name.as_ptr()) }
    }

    pub fn create_fcmp(&mut self, lhs: LLVMValueRef, rhs: LLVMValueRef) -> LLVMValueRef {
        unsafe {
            let name = CStr::from_bytes_with_nul(b"cmptmp\0").unwrap();
            let l = LLVMBuildFCmp(
                self.inner,
                llvm_sys::LLVMRealPredicate::LLVMRealOLT,
                lhs,
                rhs,
                name.as_ptr(),
            );
            let name = CStr::from_bytes_with_nul(b"booltmp\0").unwrap();
            LLVMBuildUIToFP(self.inner, l, LLVMDoubleType(), name.as_ptr())
        }
    }

    pub fn create_call(&mut self, callee: LLVMValueRef, args: Vec<LLVMValueRef>) -> LLVMValueRef {
        let mut args = args;
        let num_args = args.len();
        let name = CStr::from_bytes_with_nul(b"calltmp\0").unwrap();
        unsafe {
            LLVMBuildCall(
                self.inner,
                callee,
                args.as_mut_ptr(),
                num_args as c_uint,
                name.as_ptr(),
            )
        }
    }

    pub fn set_insert_point(&mut self, block: LLVMBasicBlockRef) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.inner, block);
        }
    }

    pub fn create_ret(&mut self, value: LLVMValueRef) -> LLVMValueRef {
        unsafe { LLVMBuildRet(self.inner, value) }
    }
}

impl Drop for LLVMBuilder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.inner);
        }
    }
}

pub struct IRGenerator {
    context: LLVMContext,
    module: LLVMModule,
    builder: LLVMBuilder,
    named_values: HashMap<String, LLVMValueRef>,
}

impl IRGenerator {
    pub fn new() -> Self {
        let mut context = LLVMContext::new();
        let module = context.create_module("kaleidoscope");
        let builder = LLVMBuilder::new(&mut context);
        Self {
            context,
            module,
            builder,
            named_values: HashMap::new(),
        }
    }

    pub fn gen(&mut self, ast: &ExprAST) -> Result<LLVMValueRef> {
        match ast {
            ExprAST::Number(value) => {
                let value = unsafe { LLVMConstReal(LLVMDoubleType(), *value) };
                Ok(value)
            }
            ExprAST::Variable(name) => self
                .named_values
                .get(name)
                .copied()
                .ok_or_else(|| LLVMError::VariableNotFound(name.clone())),
            ExprAST::BinaryOp { op, lhs, rhs } => {
                let lhs = self.gen(lhs)?;
                let rhs = self.gen(rhs)?;
                match op {
                    Operator::LessThan => Ok(self.builder.create_fcmp(lhs, rhs)),
                    Operator::Plus => Ok(self.builder.create_fadd(lhs, rhs)),
                    Operator::Minus => Ok(self.builder.create_fsub(lhs, rhs)),
                    Operator::Times => Ok(self.builder.create_fmul(lhs, rhs)),
                }
            }
            ExprAST::Call { callee, args } => {
                let callee_name = callee.clone();
                let callee = self.module.get_function(&callee)?;
                let num_args = unsafe { LLVMCountParams(callee) } as usize;
                if num_args != args.len() {
                    return Err(LLVMError::InvalidArgumentsSize(callee_name, args.len()));
                }
                let mut values = Vec::with_capacity(num_args);
                for arg in args {
                    values.push(self.gen(arg)?);
                }
                Ok(self.builder.create_call(callee, values))
            }
            ExprAST::Prototype(proto) => self.gen_proto(proto),
            ExprAST::Function { proto, body } => {
                let f = match self.module.get_function(&proto.name) {
                    Ok(f) => f,
                    _ => self.gen_proto(proto)?,
                };

                let bb = self.context.create_basic_block(f);
                self.builder.set_insert_point(bb);

                self.named_values.clear();
                let num_args = unsafe { LLVMCountParams(f) } as usize;
                let mut args = vec![std::ptr::null_mut(); num_args];
                unsafe {
                    LLVMGetParams(f, args.as_mut_ptr());
                }
                for arg in args {
                    let mut _length: size_t = 0;
                    let name = unsafe {
                        let name = LLVMGetValueName2(arg, &mut _length as *mut size_t);
                        CStr::from_ptr(name)
                    };
                    self.named_values
                        .insert(name.to_str().unwrap().to_string(), arg);
                }

                match self.gen(body) {
                    Ok(body) => {
                        self.builder.create_ret(body);
                        unsafe {
                            LLVMVerifyFunction(
                                f,
                                LLVMVerifierFailureAction::LLVMPrintMessageAction,
                            );
                        }
                        Ok(f)
                    }
                    Err(err) => {
                        unsafe {
                            LLVMDeleteFunction(f);
                        }
                        Err(err)
                    }
                }
            }
        }
    }

    pub fn gen_proto(&mut self, proto: &Prototype) -> Result<LLVMValueRef> {
        let mut doubles = vec![unsafe { LLVMDoubleType() }; proto.args.len()];
        let num_args = doubles.len();
        let f_type = unsafe {
            LLVMFunctionType(
                LLVMDoubleType(),
                doubles.as_mut_ptr(),
                num_args as c_uint,
                false as LLVMBool,
            )
        };

        let f = self.module.add_function(&proto.name, f_type);
        let mut params = vec![std::ptr::null_mut(); num_args];
        unsafe {
            LLVMGetParams(f, params.as_mut_ptr());
        }
        for (arg, name) in params.iter().zip(proto.args.iter()) {
            let name = CString::new(name.as_str()).unwrap();
            let len = name.as_bytes().len();
            unsafe {
                LLVMSetValueName2(*arg, name.as_ptr(), len);
            }
        }
        Ok(f)
    }

    pub fn dump_module(&self) {
        unsafe {
            LLVMDumpModule(self.module.inner);
        }
    }
}
