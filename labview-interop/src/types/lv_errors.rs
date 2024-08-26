//! Functions for working with the LabVIEW error clusters.
//!
//! This is only available in 64 bit currently due to restrictions
//! on unaligned pointer access.
use std::borrow::Cow;

use crate::errors::{LVInteropError, LVStatusCode, MgError, Result};
use crate::labview_layout;
use crate::memory::UPtr;
use crate::types::LStrHandle;
use crate::types::LVBool;

labview_layout!(
    /// The cluster format used by LabVIEW for transmitting errors.
    pub struct ErrorCluster<'a> {
        status: LVBool,
        code: LVStatusCode,
        source: LStrHandle<'a>,
    }
);

/// The pointer as passed by LabVIEW when using "Handles By Value" for type.
///
/// Debugging shows only one level of indirection hence UPtr here.
///
/// It is recommended to manually call `ErrorClusterPtr::as_ref` or `ErrorClusterPtr::as_mut`
/// so that null pointeres can be detected.
///
/// Many string manipulation functions are only available with the `link` feature enabled so
/// it can manipulate LabVIEW Strings.
pub type ErrorClusterPtr<'a> = UPtr<ErrorCluster<'a>>;

#[cfg(feature = "link")]
impl<'a> ErrorCluster<'a> {
    pub fn status(&self) -> bool {
        self.status == super::boolean::LV_TRUE
    }
    pub fn code(&self) -> LVStatusCode {
        self.code
    }

    fn format_error_source(source: &str, description: &str) -> String {
        match (source, description) {
            ("", description) => format!("<ERR>\n{description}"),
            (source, "") => source.to_string(),
            (source, description) => format!("{source}\n<ERR>\n{description}"),
        }
    }

    /// Set a description and source in the format that LabVIEW will interpret for display.
    fn set_source(&mut self, source: &str, description: &str) -> Result<()> {
        // Probably a clever way to avoid this allocation but for now we will take it.
        let full_source = Self::format_error_source(source, description);
        self.source.set_str(&full_source)
    }

    /// Set the error cluster to a warning state.
    pub fn set_warning(
        &mut self,
        code: LVStatusCode,
        source: &str,
        description: &str,
    ) -> Result<()> {
        self.code = code;
        self.status = super::boolean::LV_FALSE;
        self.set_source(source, description)
    }

    /// Set the error cluster to an error state.
    pub fn set_error(&mut self, code: LVStatusCode, source: &str, description: &str) -> Result<()> {
        self.code = code;
        self.status = super::boolean::LV_TRUE;
        self.set_source(source, description)
    }
}

/// A trait that can be implemented on types to allow them to be written into a
/// error cluster with `ToLvError::write_error`.
pub trait ToLvError {
    /// The code for the error. Default is 42.
    fn code(&self) -> LVStatusCode {
        MgError::BogusError.into() // code 42, Generic Error
    }

    /// True if is error. Default is true.
    fn is_error(&self) -> bool {
        true
    }

    /// The source of the error if available. Default: none.
    fn source(&self) -> Cow<'_, str> {
        "".into()
    }

    /// The description of the error;
    fn description(&self) -> Cow<'_, str>;

    /// Write into the LabVIEW Error Pointer.
    ///
    /// The pointer is the type that is recieved through the Call Library Node so
    /// there is no need to deal with references before this point.
    ///
    /// This requires the `link` feature to enable string manipulation.
    #[cfg(feature = "link")]
    fn write_error(&self, error_cluster: ErrorClusterPtr) -> Result<()> {
        let cluster = unsafe { error_cluster.as_ref_mut()? };
        let code = self.code();
        let source = self.source();
        let source = source.as_ref();
        let description = self.description();
        let description = description.as_ref();
        if self.is_error() {
            cluster.set_error(code, source, description)
        } else {
            cluster.set_warning(code, source, description)
        }
    }
}

impl ToLvError for LVInteropError {
    fn description(&self) -> Cow<'_, str> {
        self.to_string().into()
    }
}

#[macro_export]
/// the with_lverrorhandling macro
///
/// Error Type needs to implement ToLvError
///
/// # Example
///
/// ```
/// #[no_mangle]
/// pub extern "C" fn set_x_mode_by_id(
///     error: ErrorClusterPtr,
///     id: LStrHandle,
///     lv_x: UPtr<i32>,
/// ) -> LVStatusCode {
///     with_lverrorhandling!(
///         &error,
///         |id1: LStrHandle, lv_x1: UPtr<i32>| -> Result<()> {
///             let id = id1.to_rust_string();
///             let x = Xode::try_from_primitive(*lv_x1)?;
///
///             with_mut_device(&id, |device, _is_live| {
///                 info!("Setting X mode {:?}", x);
///                 device.set_x_mode(x)?;
///                 Ok(())
///             })
///         },
///         id,
///         lv_x
///     )
/// }
/// ```
macro_rules! with_lverrorhandling {
    // Match a variadic number of parameters
    ($error_cluster:expr, $func:expr, $($params:expr),*) => {
        // only run if the incoming error cluster does not have an error
        if !$error_cluster.status() {
        // Call the function within a error handling context
            match ($func)($($params),*) {
                Ok(_) => LVStatusCode::SUCCESS,
                Err(err) => {
                    let errstr = err.to_string();
                    let errcode = err.code();

                    // Update the error cluster if there's an error
                    if let Ok(cluster) = unsafe { $error_cluster.as_ref_mut() } {

                        cluster.set_error(errcode, "Rust Interop Error", &errstr).unwrap();
                    }
                    // Return the appropriate LVStatusCode
                    errcode
                }
            }
        } else {
            $error_cluster.code()
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_source_writer_empty_description() {
        let source = ErrorCluster::format_error_source("Rust", "");
        assert_eq!(source, "Rust");
    }

    #[test]
    fn test_source_writer_with_description() {
        let source = ErrorCluster::format_error_source("Rust", "An Error Occured");
        let expected = "Rust\n<ERR>\nAn Error Occured";
        assert_eq!(source, expected)
    }

    #[test]
    fn test_source_writer_empty_source() {
        let source = ErrorCluster::format_error_source("", "An Error Occured");
        let expected = "<ERR>\nAn Error Occured";
        assert_eq!(source, expected)
    }
}
