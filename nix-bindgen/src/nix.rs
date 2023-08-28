use crate::ffi;
use anyhow::{bail, Context};
use std::cell::RefCell;
use std::mem::ManuallyDrop;
use std::rc::Rc;
use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_uint, c_void},
    path::{Path, PathBuf},
    ptr::{null_mut, NonNull},
};

type Result<T> = anyhow::Result<T>;

fn c_str_ptr(s: impl AsRef<str>) -> Result<*const c_char> {
    let c_str = CString::new(s.as_ref().as_bytes()).context("couldn't convert &str to CString")?;
    Ok(c_str.into_raw() as *const c_char)
}

fn raw_str_to_string(raw: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw) };
    c_str.to_string_lossy().into_owned()
}

fn err_msg(ctx: Ctx) -> String {
    let raw = unsafe {
        ffi::nix_err_msg(
            null_mut::<ffi::nix_c_context>(),
            ctx.borrow_mut().ptr(),
            null_mut::<c_uint>(),
        )
    };
    raw_str_to_string(raw)
}

fn check_err(ctx: Ctx, err: ffi::nix_err) -> Result<()> {
    if err != ffi::NIX_OK.try_into().unwrap() {
        let msg = err_msg(ctx);
        bail!("{}", msg);
    }
    Ok(())
}

/// A context type corresponding to `nix_c_context`
#[derive(Debug, Clone)]
pub struct NixContext(NonNull<ffi::nix_c_context>);

impl NixContext {
    /// Creates a new Nix context
    pub fn new() -> Result<Self> {
        let ctx = unsafe { ffi::nix_c_context_create() };
        if ctx.is_null() {
            bail!("nix_c_context_create returned a null pointer");
        }
        Ok(NixContext(NonNull::new(ctx).unwrap())) // already checked that it's non-null
    }

    /// Returns a mutable reference to the inner pointer
    unsafe fn ptr(&mut self) -> *mut ffi::nix_c_context {
        let inner_ptr = self.0.clone().as_ptr();
        inner_ptr
    }
}

impl Drop for NixContext {
    fn drop(&mut self) {
        unsafe {
            ffi::nix_c_context_free(self.0.clone().as_ptr());
        }
    }
}

type Ctx = ManuallyDrop<Rc<RefCell<NixContext>>>;

/// A guard returned by initializing `libexpr`
#[derive(Debug, Copy, Clone)]
pub struct NixInitGuard;

impl NixInitGuard {
    pub fn new(ctx: Ctx) -> Result<Self> {
        let err = unsafe { ffi::nix_libexpr_init(ctx.borrow_mut().ptr()) };
        check_err(ctx, err)?;
        Ok(Self)
    }
}

/// The different Nix stores that you can open
#[derive(Debug, Clone)]
pub enum NixStoreType {
    Dummy,
    Auto,
    Path(PathBuf),
    Daemon,
    Local,
    // TODO:
    // SSH
    // HTTP
    // S3
}

impl From<NixStoreType> for String {
    fn from(value: NixStoreType) -> String {
        use NixStoreType::*;
        match value {
            Dummy => String::from("dummy://"),
            Auto => String::from("auto"),
            Path(path) => path.as_path().to_string_lossy().into_owned(),
            Daemon => String::from("daemon"),
            Local => String::from("local"),
        }
    }
}

/// A handle to an initialized Nix store
#[derive(Debug, Clone)]
pub struct NixStore {
    inner: NonNull<ffi::Store>,
    ctx: Ctx,
}

impl NixStore {
    /// Open a connection to the specific Nix store
    pub fn new(ctx: Ctx, store_type: NixStoreType) -> Result<Self> {
        let store_url: String = store_type.into();
        let url_ptr = c_str_ptr(store_url)?;
        let store = unsafe {
            ffi::nix_store_open(
                ctx.borrow_mut().ptr(),
                url_ptr,
                null_mut::<*mut *const c_char>(),
            )
        };
        if store.is_null() {
            bail!("nix_store_open returned a null pointer");
        }
        let store = NixStore {
            inner: NonNull::new(store).unwrap(),
            ctx: ctx.clone(),
        };
        Ok(store)
    }

    /// Returns a mutable reference to the inner pointer
    unsafe fn ptr(&mut self) -> *mut ffi::Store {
        let inner_ptr = self.inner.clone().as_ptr();
        inner_ptr
    }
}

impl Drop for NixStore {
    fn drop(&mut self) {
        unsafe { ffi::nix_store_unref(self.ptr()) };
        let ctx = unsafe { ManuallyDrop::take(&mut self.ctx) };
        drop(ctx);
    }
}

/// A handle to a Nix evaluator state
#[derive(Debug, Clone)]
pub struct NixState(NonNull<ffi::State>);

impl NixState {
    /// Initialize a new evaluator state
    pub fn new(ctx: Ctx, store: &mut NixStore, _init_guard: NixInitGuard) -> Result<Self> {
        let state = unsafe {
            ffi::nix_state_create(
                ctx.borrow_mut().ptr(),
                null_mut::<*const c_char>(),
                store.ptr(),
            )
        };
        if state.is_null() {
            bail!("nix_state_create returned a null pointer");
        }
        Ok(NixState(NonNull::new(state).unwrap()))
    }

    /// Returns a mutable reference to the inner pointer
    unsafe fn ptr(&mut self) -> *mut ffi::State {
        let inner_ptr = self.0.clone().as_ptr();
        inner_ptr
    }
}

impl Drop for NixState {
    fn drop(&mut self) {
        unsafe { ffi::nix_state_free(self.ptr()) };
    }
}

/// The state of an unevaluated NixValue
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct ValueInit;

/// The state of an unevaluated NixValue
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct ValueReady;

pub trait NixValueState {}
impl NixValueState for ValueInit {}
impl NixValueState for ValueReady {}

/// The type of a value
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueType {
    Foo,
}

/// A Nix value
#[derive(Debug)]
pub struct NixValue<S: NixValueState> {
    inner: NonNull<ffi::Value>,
    ctx: Ctx,
    eval_state: S,
    value_type: Option<ValueType>,
}

impl<S: NixValueState> NixValue<S> {
    /// Returns a mutable reference to the inner pointer
    unsafe fn ptr(&mut self) -> *mut ffi::Value {
        let inner_ptr = self.inner.clone().as_ptr();
        inner_ptr
    }
}

impl NixValue<ValueInit> {
    /// Create a new value
    pub fn new(ctx: Ctx, state: &mut NixState) -> Result<Self> {
        let val = unsafe { ffi::nix_alloc_value(ctx.borrow_mut().ptr(), state.ptr()) };
        if val.is_null() {
            bail!("nix_alloc_value returned a null pointer");
        }
        let val = NixValue {
            inner: NonNull::new(val).unwrap(),
            ctx: ctx.clone(),
            eval_state: ValueInit,
            value_type: None,
        };
        Ok(val)
    }

    /// Evaluate a string and assign the expression to this value
    pub fn assign_from_expr(
        mut self,
        expr: impl AsRef<str>,
        state: &mut NixState,
        expr_file_path: Option<PathBuf>,
    ) -> Result<NixValue<ValueReady>> {
        let path = match expr_file_path {
            Some(p) => c_str_ptr(p.to_string_lossy())?,
            None => c_str_ptr(".")?,
        };
        let err = unsafe {
            let ctx_ptr = self.ctx.borrow_mut().ptr();
            let err = ffi::nix_expr_eval_from_string(
                ctx_ptr,
                state.ptr(),
                c_str_ptr(expr)?,
                path,
                self.ptr(),
            );
            err
        };
        check_err(self.ctx.clone(), err)?;
        // TODO: get the value type
        let value_type = Some(ValueType::Foo);
        let ctx = self.ctx.clone();
        Ok(NixValue {
            inner: self.inner,
            ctx,
            eval_state: ValueReady,
            value_type,
        })
    }
}

impl NixValue<ValueReady> {
    /// Force evaluation of the value.
    ///
    /// Nix is lazily evaluated, so values are not evaluated until necessary.
    /// This method forces the value to be evaluated.
    pub fn force(&mut self, state: &mut NixState) -> Result<()> {
        let err = unsafe {
            ffi::nix_value_force(
                self.ctx.borrow_mut().ptr(),
                state.ptr(),
                self.inner.clone().as_ptr(),
            )
        };
        check_err(self.ctx.clone(), err)?;
        Ok(())
    }

    /// Returns the type of the value
    pub fn value_type(&self) -> ValueType {
        assert!(
            self.value_type.is_some(),
            "NixValue<ValueReady> must have a type upon construction"
        );
        self.value_type.unwrap()
    }

    /// Returns a printable string of the evaluated value.
    pub fn display(&mut self) -> String {
        // TODO: match on value type, then figure out how to print those values
        let raw = unsafe {
            let ctx_ptr = self.ctx.borrow_mut().ptr();
            let raw = ffi::nix_get_string(ctx_ptr, self.ptr());
            raw
        };
        raw_str_to_string(raw)
    }
}

impl<S: NixValueState> Drop for NixValue<S> {
    fn drop(&mut self) {
        // Uh oh, this destructor should be fallible!
        let _err = unsafe {
            let err = ffi::nix_gc_decref(
                self.ctx.borrow_mut().ptr(),
                self.inner.as_ptr() as *const c_void,
            );
            err
        };
    }
}

/// Evaluate a Nix expression
pub fn eval_string(input: impl AsRef<str>) -> Result<String> {
    let ctx = ManuallyDrop::new(Rc::new(RefCell::new(NixContext::new()?)));
    let guard = NixInitGuard::new(ctx.clone())?;
    let mut store = NixStore::new(ctx.clone(), NixStoreType::Auto)?;
    let mut state = NixState::new(ctx.clone(), &mut store, guard)?;
    let mut value = NixValue::new(ctx.clone(), &mut state)?;
    let mut value = value.assign_from_expr(input, &mut state, None)?;
    value.force(&mut state)?;
    Ok(value.display())
}
