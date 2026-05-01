//! # PureReason Procedural Macros
//!
//! This crate provides the `#[kantian]` attribute macro, which wraps an async
//! function returning `String` with automatic Kantian epistemic validation via
//! the full `KantianPipeline`. If the pipeline's verdict reaches `RiskLevel::High`,
//! the function returns `Err(PureReasonError::HighRisk { .. })` instead of the raw
//! output.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use pure_reason_macros::kantian;
//!
//! #[kantian]
//! async fn generate_answer(question: &str) -> String {
//!     format!("The answer to '{}' is 42.", question)
//! }
//!
//! // `generate_answer` now returns `Result<String, pure_reason_core::error::PureReasonError>`
//! ```
//!
//! ## Restrictions
//!
//! - Only `async fn` is supported. Applying `#[kantian]` to a non-async function
//!   is a compile-time error.
//! - The function **must** return `-> String`. Returning `Result<String, _>` or
//!   any other type is a compile-time error.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type};

/// Returns `true` when the type is exactly the bare identifier `String`.
fn is_plain_string(ty: &Type) -> bool {
    matches!(ty, Type::Path(p) if p.qself.is_none() && p.path.is_ident("String"))
}

/// Wraps an `async fn` returning `String` with Kantian epistemic validation.
///
/// The transformed function returns
/// `Result<String, pure_reason_core::error::PureReasonError>`.
///
/// At runtime the macro:
/// 1. Executes the original function body in an inner `async move` block, capturing
///    the `String` output as `__kantian_output`.
/// 2. Runs `KantianPipeline::new().process(__kantian_output.clone())`.
/// 3. If `report.verdict.risk >= RiskLevel::High`, returns
///    `Err(PureReasonError::HighRisk { risk, report_json })`.
/// 4. Otherwise returns `Ok(__kantian_output)`.
///
/// # Errors
///
/// - Compile error if applied to a non-`async` function.
/// - Compile error if the function does not return `-> String`.
#[proc_macro_attribute]
pub fn kantian(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // ── Guard: must be async ────────────────────────────────────────────────
    if input.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            input.sig.fn_token,
            "#[kantian] can only be applied to `async fn`",
        )
        .to_compile_error()
        .into();
    }

    // ── Guard: must return -> String ────────────────────────────────────────
    let returns_string = match &input.sig.output {
        ReturnType::Type(_, ty) => is_plain_string(ty),
        ReturnType::Default => false,
    };

    if !returns_string {
        return syn::Error::new_spanned(
            &input.sig.output,
            "#[kantian] requires the function to return `-> String`. \
             Returning `Result<String, _>` or other types is not supported.",
        )
        .to_compile_error()
        .into();
    }

    // ── Build the new signature (same but returns Result) ───────────────────
    let vis = &input.vis;
    let attrs = &input.attrs;
    let block = &input.block;

    let mut new_sig = input.sig.clone();
    new_sig.output = syn::parse_quote!(
        -> ::std::result::Result<
            ::std::string::String,
            ::pure_reason_core::error::PureReasonError,
        >
    );

    // ── Emit the transformed function ───────────────────────────────────────
    //
    // The original body runs inside `async move { ... }` so that:
    //   - all function arguments are captured by move (they are still in scope
    //     because we have not used them in the outer function body yet),
    //   - `return expr` inside the original body returns from the async *block*
    //     (i.e. sets the block's value to `expr : String`), not from the outer
    //     function, avoiding a type mismatch.
    let expanded = quote! {
        #(#attrs)*
        #vis #new_sig {
            // Run the original body; capture its String result.
            let __kantian_output: ::std::string::String = (async move #block).await;

            // Run the full Kantian pipeline on the output.
            let __kantian_pipeline = ::pure_reason_core::pipeline::KantianPipeline::new();
            let __kantian_report = match __kantian_pipeline.process(__kantian_output.clone()) {
                Ok(r) => r,
                Err(e) => return Err(e),
            };

            // Reject outputs that reach RiskLevel::High.
            if __kantian_report.verdict.risk >= ::pure_reason_core::pipeline::RiskLevel::High {
                return Err(::pure_reason_core::error::PureReasonError::HighRisk {
                    risk: __kantian_report.verdict.risk.to_string(),
                    report_json: __kantian_report.to_json().unwrap_or_default(),
                });
            }

            Ok(__kantian_output)
        }
    };

    expanded.into()
}

#[cfg(test)]
mod tests {
    // Compile-time tests for the helper predicate.
    #[test]
    fn is_plain_string_recognises_string() {
        let ty: syn::Type = syn::parse_quote!(String);
        assert!(super::is_plain_string(&ty));
    }

    #[test]
    fn is_plain_string_rejects_result() {
        let ty: syn::Type = syn::parse_quote!(Result<String, ()>);
        assert!(!super::is_plain_string(&ty));
    }

    #[test]
    fn is_plain_string_rejects_str() {
        let ty: syn::Type = syn::parse_quote!(&str);
        assert!(!super::is_plain_string(&ty));
    }
}
