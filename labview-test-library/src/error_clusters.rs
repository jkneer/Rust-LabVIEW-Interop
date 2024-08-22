use labview_interop::errors::LVStatusCode;
/// A simple type for testing the error integration.
///
use labview_interop::types::{ErrorClusterPtr, ToLvError};
use labview_interop::with_lverrorhandling;
struct ErrorText(&'static str);

#[cfg(target_pointer_width = "64")]
impl ToLvError for ErrorText {
    fn source(&self) -> std::borrow::Cow<'_, str> {
        "Rust".into()
    }

    fn description(&self) -> std::borrow::Cow<'_, str> {
        self.0.into()
    }
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub extern "C" fn set_error_cluster(error_cluster: ErrorClusterPtr) -> LVStatusCode {
    let error = ErrorText("This is a test");
    error.write_error(error_cluster).into()
}

#[cfg(target_pointer_width = "64")]
#[no_mangle]
pub extern "C" fn auto_lv_errorhandling(
    error_cluster: ErrorClusterPtr,
    param1: isize,
    param2: isize,
) -> LVStatusCode {
    use labview_interop::errors::LVInteropError;

    with_lverrorhandling!(
        &error_cluster,
        |p1, p2| {
            // DO stuff, use ?, etc to deal with errors in rust, the context macro takes care
            // of updating the error cluster

            Err::<(), LVInteropError>(LVInteropError::InternalError(
                labview_interop::errors::InternalError::ArrayDimensionMismatch,
            ))
        },
        param1,
        param2
    )
}
