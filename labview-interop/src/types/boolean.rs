//! Handling for boolean types to and from LabVIEW.

/// The LVBool type is used by LabVIEW for boolean types
/// on the block diagram.
///
/// When you pass data to and from LabVIEW as boolean
/// types, this is the equivalent.
///
/// You can use `.into()` to convert between this and
/// rust [`bool`] types.
#[repr(transparent)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct LVBool(u8);

/// A false constant in the LVBool format.
pub const LV_FALSE: LVBool = LVBool(0);
/// A true constant in the LVBool format.
pub const LV_TRUE: LVBool = LVBool(1);

impl From<bool> for LVBool {
    fn from(value: bool) -> Self {
        match value {
            true => LV_TRUE,
            false => LV_FALSE,
        }
    }
}

impl From<LVBool> for bool {
    fn from(value: LVBool) -> Self {
        !matches!(value.0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_false_to_lv_bool() {
        let value: LVBool = false.into();
        assert_eq!(value, LV_FALSE)
    }

    #[test]
    fn test_boolean_true_to_lv_bool() {
        let value: LVBool = true.into();
        assert_eq!(value, LV_TRUE)
    }

    #[test]
    fn test_boolean_lvfalse_to_bool() {
        let value: bool = LV_FALSE.into();
        assert_eq!(value, false)
    }

    #[test]
    fn test_boolean_lvtrue_to_bool() {
        let value: bool = LV_TRUE.into();
        assert_eq!(value, true)
    }

    #[test]
    fn test_any_non_zero_to_bool() {
        let value: bool = LVBool(23).into();
        assert_eq!(value, true)
    }
}
