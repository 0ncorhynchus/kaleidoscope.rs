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

#[derive(Debug, PartialEq)]
pub struct LLVMValue {
    ptr: LLVMValueRef,
}

impl LLVMValue {
    fn new(ptr: LLVMValueRef) -> Self {
        Self { ptr }
    }

    pub fn name(&self) -> String {
        let mut _len: size_t = 0;
        let name = unsafe { CStr::from_ptr(LLVMGetValueName2(self.ptr, &mut _len as *mut size_t)) };
        name.to_str().unwrap().to_string()
    }

    pub fn dump(&self) {
        unsafe {
            LLVMDumpValue(self.ptr);
        }
    }
}

impl From<FunctionRef> for LLVMValue {
    fn from(f: FunctionRef) -> Self {
        Self { ptr: f.ptr }
    }
}

#[derive(Debug, PartialEq)]
pub struct FunctionRef {
    ptr: LLVMValueRef,
}

impl FunctionRef {
    fn new(ptr: LLVMValueRef) -> Self {
        FunctionRef { ptr }
    }

    pub fn num_args(&self) -> usize {
        (unsafe { LLVMCountParams(self.ptr) }) as usize
    }

    pub fn args(&self) -> Vec<LLVMValue> {
        let mut args = vec![std::ptr::null_mut(); self.num_args()];
        unsafe {
            LLVMGetParams(self.ptr, args.as_mut_ptr());
        }
        args.iter().map(|ptr| LLVMValue::new(*ptr)).collect()
    }

    pub fn verify(&self, action: LLVMVerifierFailureAction) {
        unsafe {
            LLVMVerifyFunction(self.ptr, action);
        }
    }

    pub fn delete(&self) {
        unsafe {
            LLVMDeleteFunction(self.ptr);
        }
    }
}

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

    pub fn create_basic_block(&mut self, f: &FunctionRef) -> LLVMBasicBlockRef {
        let name = CStr::from_bytes_with_nul(b"entry\0").unwrap();
        unsafe { LLVMAppendBasicBlockInContext(self.inner, f.ptr, name.as_ptr()) }
    }

    pub fn get_double_type(&mut self) -> LLVMTypeRef {
        unsafe { LLVMDoubleTypeInContext(self.inner) }
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
    pub fn get_function(&mut self, name: &str) -> Result<FunctionRef> {
        let c_name = CString::new(name).unwrap();
        let f = unsafe { LLVMGetNamedFunction(self.inner, c_name.as_ptr()) };
        if f.is_null() {
            Err(LLVMError::FunctionNotFound(name.to_string()))
        } else {
            Ok(FunctionRef::new(f))
        }
    }

    pub fn add_function(&mut self, name: &str, ty: LLVMTypeRef) -> FunctionRef {
        let name = CString::new(name).unwrap();
        let ptr = unsafe { LLVMAddFunction(self.inner, name.as_ptr(), ty) };
        FunctionRef::new(ptr)
    }
}

pub struct LLVMBuilder {
    inner: LLVMBuilderRef,
    ty: LLVMTypeRef,
}

impl LLVMBuilder {
    pub fn new(context: &mut LLVMContext) -> Self {
        Self {
            inner: unsafe { LLVMCreateBuilderInContext(context.inner) },
            ty: context.get_double_type(),
        }
    }

    pub fn create_fadd(&mut self, lhs: &LLVMValue, rhs: &LLVMValue) -> LLVMValue {
        let name = CStr::from_bytes_with_nul(b"addtmp\0").unwrap();
        let ptr = unsafe { LLVMBuildFAdd(self.inner, lhs.ptr, rhs.ptr, name.as_ptr()) };
        LLVMValue::new(ptr)
    }

    pub fn create_fsub(&mut self, lhs: &LLVMValue, rhs: &LLVMValue) -> LLVMValue {
        let name = CStr::from_bytes_with_nul(b"subtmp\0").unwrap();
        let ptr = unsafe { LLVMBuildFSub(self.inner, lhs.ptr, rhs.ptr, name.as_ptr()) };
        LLVMValue::new(ptr)
    }

    pub fn create_fmul(&mut self, lhs: &LLVMValue, rhs: &LLVMValue) -> LLVMValue {
        let name = CStr::from_bytes_with_nul(b"multmp\0").unwrap();
        let ptr = unsafe { LLVMBuildFMul(self.inner, lhs.ptr, rhs.ptr, name.as_ptr()) };
        LLVMValue::new(ptr)
    }

    pub fn create_fcmp(&mut self, lhs: &LLVMValue, rhs: &LLVMValue) -> LLVMValue {
        let ptr = unsafe {
            let name = CStr::from_bytes_with_nul(b"cmptmp\0").unwrap();
            let l = LLVMBuildFCmp(
                self.inner,
                llvm_sys::LLVMRealPredicate::LLVMRealOLT,
                lhs.ptr,
                rhs.ptr,
                name.as_ptr(),
            );
            let name = CStr::from_bytes_with_nul(b"booltmp\0").unwrap();
            LLVMBuildUIToFP(self.inner, l, self.ty, name.as_ptr())
        };
        LLVMValue::new(ptr)
    }

    pub fn create_call(&mut self, callee: &FunctionRef, args: Vec<LLVMValue>) -> LLVMValue {
        let mut args: Vec<_> = args.into_iter().map(|v| v.ptr).collect();
        let num_args = args.len();
        let name = CStr::from_bytes_with_nul(b"calltmp\0").unwrap();
        let ptr = unsafe {
            LLVMBuildCall(
                self.inner,
                callee.ptr,
                args.as_mut_ptr(),
                num_args as c_uint,
                name.as_ptr(),
            )
        };
        LLVMValue::new(ptr)
    }

    pub fn set_insert_point(&mut self, block: LLVMBasicBlockRef) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.inner, block);
        }
    }

    pub fn create_ret(&mut self, value: &LLVMValue) -> LLVMValue {
        let ptr = unsafe { LLVMBuildRet(self.inner, value.ptr) };
        LLVMValue::new(ptr)
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
    named_values: HashMap<String, LLVMValue>,
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

    pub fn gen(&mut self, ast: &ExprAST) -> Result<LLVMValue> {
        match ast {
            ExprAST::Number(value) => {
                let value = unsafe { LLVMConstReal(self.context.get_double_type(), *value) };
                Ok(LLVMValue::new(value))
            }
            ExprAST::Variable(name) => match self.named_values.get(name) {
                Some(value) => Ok(LLVMValue::new(value.ptr)),
                None => Err(LLVMError::VariableNotFound(name.clone())),
            },
            ExprAST::BinaryOp { op, lhs, rhs } => {
                let lhs = self.gen(lhs)?;
                let rhs = self.gen(rhs)?;
                match op {
                    Operator::LessThan => Ok(self.builder.create_fcmp(&lhs, &rhs)),
                    Operator::Plus => Ok(self.builder.create_fadd(&lhs, &rhs)),
                    Operator::Minus => Ok(self.builder.create_fsub(&lhs, &rhs)),
                    Operator::Times => Ok(self.builder.create_fmul(&lhs, &rhs)),
                }
            }
            ExprAST::Call { callee, args } => {
                let callee_name = callee.clone();
                let callee = self.module.get_function(&callee)?;
                let num_args = callee.num_args();
                if num_args != args.len() {
                    return Err(LLVMError::InvalidArgumentsSize(callee_name, args.len()));
                }
                let mut values = Vec::with_capacity(num_args);
                for arg in args {
                    values.push(self.gen(arg)?);
                }
                Ok(self.builder.create_call(&callee, values))
            }
            ExprAST::Prototype(proto) => Ok(self.gen_proto(proto)?.into()),
            ExprAST::Function { proto, body } => {
                let f = match self.module.get_function(&proto.name) {
                    Ok(f) => f,
                    _ => self.gen_proto(proto)?,
                };

                let bb = self.context.create_basic_block(&f);
                self.builder.set_insert_point(bb);

                self.named_values.clear();
                for arg in f.args() {
                    self.named_values.insert(arg.name(), arg);
                }

                match self.gen(body) {
                    Ok(body) => {
                        self.builder.create_ret(&body);
                        f.verify(LLVMVerifierFailureAction::LLVMPrintMessageAction);
                        Ok(f.into())
                    }
                    Err(err) => {
                        f.delete();
                        Err(err)
                    }
                }
            }
        }
    }

    pub fn gen_proto(&mut self, proto: &Prototype) -> Result<FunctionRef> {
        let mut doubles = vec![self.context.get_double_type(); proto.args.len()];
        let num_args = doubles.len();
        let f_type = unsafe {
            LLVMFunctionType(
                self.context.get_double_type(),
                doubles.as_mut_ptr(),
                num_args as c_uint,
                false as LLVMBool,
            )
        };

        let f = self.module.add_function(&proto.name, f_type);
        for (arg, name) in f.args().iter().zip(proto.args.iter()) {
            let name = CString::new(name.as_str()).unwrap();
            let len = name.as_bytes().len();
            unsafe {
                LLVMSetValueName2(arg.ptr, name.as_ptr(), len);
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
