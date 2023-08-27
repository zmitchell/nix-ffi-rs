use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_uint, c_void},
    ptr::null_mut,
};

mod nix;

use anyhow::{anyhow, Context};

type Error = anyhow::Error;

fn main() -> Result<(), Error> {
    unsafe {
        // Initialize all the things
        let ctx = nix::ffi::nix_c_context_create();
        nix::ffi::nix_libexpr_init(ctx);
        let store_name = c_str_ptr("auto")?;
        let store = nix::ffi::nix_store_open(ctx, store_name, null_mut::<*mut *const i8>());
        // Are we writing Go now?
        let err = nix::ffi::nix_libexpr_init(ctx);
        if err != nix::ffi::NIX_OK.try_into().unwrap() {
            return Err(anyhow!("couldn't initialize libexpr"))
                .with_context(|| format!("{}", err_msg(ctx)))?;
            // bail!("failed to initialize libexpr: code {}", err);
        }
        let state = nix::ffi::nix_state_create(ctx, null_mut::<*const c_char>(), store);
        // You can now do stuff
        let expr_val = nix::ffi::nix_alloc_value(ctx, state);
        let expr = std::env::args()
            .nth(1)
            .ok_or_else(|| anyhow!("no expression provided"))?;
        let err = nix::ffi::nix_expr_eval_from_string(
            ctx,
            state,
            c_str_ptr(expr)?,
            c_str_ptr(".")?,
            expr_val,
        );
        if err != nix::ffi::NIX_OK.try_into().unwrap() {
            return Err(anyhow!("{}", err_msg(ctx)));
        }
        let err = nix::ffi::nix_value_force(ctx, state, expr_val);
        if err != nix::ffi::NIX_OK.try_into().unwrap() {
            return Err(anyhow!("{}", err_msg(ctx)));
        }
        let expr_result = raw_str_to_string(nix::ffi::nix_get_string(ctx, expr_val));
        println!("Evaluation result: {}", expr_result);
        // Clean up
        let err = nix::ffi::nix_gc_decref(ctx, expr_val as *const c_void);
        if err != nix::ffi::NIX_OK.try_into().unwrap() {
            return Err(anyhow!("{}", err_msg(ctx)));
        }
        nix::ffi::nix_state_free(state);
        nix::ffi::nix_store_unref(store);
        nix::ffi::nix_c_context_free(ctx);
    }
    Ok(())
}

fn c_str_ptr(s: impl AsRef<str>) -> Result<*const c_char, Error> {
    let c_str = CString::new(s.as_ref().as_bytes()).context("couldn't convert &str to CString")?;
    Ok(c_str.into_raw() as *const c_char)
}

fn raw_str_to_string(raw: *const c_char) -> String {
    let c_str = unsafe { CStr::from_ptr(raw) };
    c_str.to_string_lossy().into_owned()
}

fn err_msg(ctx: *mut nix::ffi::nix_c_context) -> String {
    let raw = unsafe {
        nix::ffi::nix_err_msg(
            null_mut::<nix::ffi::nix_c_context>(),
            ctx,
            null_mut::<c_uint>(),
        )
    };
    raw_str_to_string(raw)
}
