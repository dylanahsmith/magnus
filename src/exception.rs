//! Types and functions for working with Ruby exceptions.

use std::{fmt, ops::Deref};

use rb_sys::{
    rb_eArgError, rb_eEOFError, rb_eEncCompatError, rb_eEncodingError, rb_eException, rb_eFatal,
    rb_eFloatDomainError, rb_eFrozenError, rb_eIOError, rb_eIndexError, rb_eInterrupt,
    rb_eKeyError, rb_eLoadError, rb_eLocalJumpError, rb_eMathDomainError, rb_eNameError,
    rb_eNoMemError, rb_eNoMethodError, rb_eNotImpError, rb_eRangeError, rb_eRegexpError,
    rb_eRuntimeError, rb_eScriptError, rb_eSecurityError, rb_eSignal, rb_eStandardError,
    rb_eStopIteration, rb_eSyntaxError, rb_eSysStackError, rb_eSystemCallError, rb_eSystemExit,
    rb_eThreadError, rb_eTypeError, rb_eZeroDivError, VALUE,
};

#[cfg(ruby_gte_2_7)]
use rb_sys::rb_eNoMatchingPatternError;

#[cfg(ruby_gte_3_1)]
use rb_sys::rb_eNoMatchingPatternKeyError;

use crate::{
    class::RClass,
    debug_assert_value,
    error::Error,
    module::Module,
    object::Object,
    r_array::RArray,
    value::{private, NonZeroValue, ReprValue, Value},
};

/// Wrapper type for a Value known to be an instance of Ruby's Exception class.
///
/// All [`Value`] methods should be available on this type through [`Deref`],
/// but some may be missed by this documentation.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Exception(NonZeroValue);

impl Exception {
    /// Return `Some(Exception)` if `val` is an `Exception`, `None` otherwise.
    #[inline]
    pub fn from_value(val: Value) -> Option<Self> {
        debug_assert_value!(val);
        unsafe {
            val.class()
                .is_inherited(RClass::from_rb_value_unchecked(rb_eException))
                .then(|| Self(NonZeroValue::new_unchecked(val)))
        }
    }

    #[inline]
    pub(crate) unsafe fn from_rb_value_unchecked(val: VALUE) -> Self {
        Self(NonZeroValue::new_unchecked(Value::new(val)))
    }

    /// Return the Ruby backtrace for the exception, as a [`RArray`] of
    /// [`RString`](`crate::r_string::RString`)s.
    pub fn backtrace(&self) -> Result<Option<RArray>, Error> {
        self.funcall("backtrace", ())
    }
}

impl Deref for Exception {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}

impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.to_s_infallible() })
    }
}

impl fmt::Debug for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            unsafe {
                writeln!(f, "{}: {}", self.classname(), self)?;
                if let Ok(Some(backtrace)) = self.backtrace() {
                    for line in backtrace.each() {
                        if let Ok(line) = line {
                            writeln!(f, "{}", line)?;
                        } else {
                            break;
                        }
                    }
                }
            }
            Ok(())
        } else {
            write!(f, "{}", self.inspect())
        }
    }
}

impl From<Exception> for Value {
    fn from(val: Exception) -> Self {
        *val
    }
}

unsafe impl private::ReprValue for Exception {
    fn to_value(self) -> Value {
        *self
    }

    unsafe fn from_value_unchecked(val: Value) -> Self {
        Self(NonZeroValue::new_unchecked(val))
    }
}

impl ReprValue for Exception {}

/// A Value known to be an instance of Class and subclass of Exception.
///
/// All [`Value`] methods should be available on this type through [`Deref`],
/// but some may be missed by this documentation.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct ExceptionClass(NonZeroValue);

impl ExceptionClass {
    /// Return `Some(ExceptionClass)` if `val` is an `ExceptionClass`, `None`
    /// otherwise.
    #[inline]
    pub fn from_value(val: Value) -> Option<Self> {
        debug_assert_value!(val);
        unsafe {
            RClass::from_value(val).and_then(|class| {
                class
                    .is_inherited(RClass::from_rb_value_unchecked(rb_eException))
                    .then(|| Self(NonZeroValue::new_unchecked(val)))
            })
        }
    }

    #[inline]
    pub(crate) unsafe fn from_rb_value_unchecked(val: VALUE) -> Self {
        Self(NonZeroValue::new_unchecked(Value::new(val)))
    }
}

impl Deref for ExceptionClass {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}

impl Default for ExceptionClass {
    fn default() -> Self {
        unsafe { Self::from_rb_value_unchecked(rb_eRuntimeError) }
    }
}

impl fmt::Display for ExceptionClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.to_s_infallible() })
    }
}

impl fmt::Debug for ExceptionClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

impl From<ExceptionClass> for Value {
    fn from(val: ExceptionClass) -> Self {
        *val
    }
}

impl Object for ExceptionClass {}
impl Module for ExceptionClass {}

unsafe impl private::ReprValue for ExceptionClass {
    fn to_value(self) -> Value {
        *self
    }

    unsafe fn from_value_unchecked(val: Value) -> Self {
        Self(NonZeroValue::new_unchecked(val))
    }
}

impl ReprValue for ExceptionClass {}

/// Return Ruby's `ArgumentError` class.
#[inline]
pub fn arg_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eArgError) }
}

/// Return Ruby's `EOFError` class.
#[inline]
pub fn eof_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eEOFError) }
}

/// Return Ruby's `Encoding::CompatibilityError` class.
#[inline]
pub fn enc_compat_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eEncCompatError) }
}

/// Return Ruby's `EncodingError` class.
#[inline]
pub fn encoding_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eEncodingError) }
}

/// Return Ruby's `Exception` class.
#[inline]
pub fn exception() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eException) }
}

/// Return Ruby's `fatal` class.
#[inline]
pub fn fatal() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eFatal) }
}

/// Return Ruby's `FloatDomainError` class.
#[inline]
pub fn float_domain_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eFloatDomainError) }
}

/// Return Ruby's `FrozenError` class.
#[inline]
pub fn frozen_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eFrozenError) }
}

/// Return Ruby's `IOError` class.
#[inline]
pub fn io_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eIOError) }
}

/// Return Ruby's `IndexError` class.
#[inline]
pub fn index_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eIndexError) }
}

/// Return Ruby's `Interrupt` class.
#[inline]
pub fn interrupt() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eInterrupt) }
}

/// Return Ruby's `KeyError` class.
#[inline]
pub fn key_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eKeyError) }
}

/// Return Ruby's `LoadError` class.
#[inline]
pub fn load_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eLoadError) }
}

/// Return Ruby's `LocalJumpError` class.
#[inline]
pub fn local_jump_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eLocalJumpError) }
}

/// Return Ruby's `Math::DomainError` class.
#[inline]
pub fn math_domain_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eMathDomainError) }
}

/// Return Ruby's `NameError` class.
#[inline]
pub fn name_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eNameError) }
}

/// Return Ruby's `NoMatchingPatternError` class.
#[cfg(ruby_gte_2_7)]
#[inline]
pub fn no_matching_pattern_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eNoMatchingPatternError) }
}

/// Return Ruby's `NoMatchingPatternKeyError` class.
#[cfg(ruby_gte_3_1)]
#[inline]
pub fn no_matching_pattern_key_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eNoMatchingPatternKeyError) }
}

/// Return Ruby's `NoMemoryError` class.
#[inline]
pub fn no_mem_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eNoMemError) }
}

/// Return Ruby's `NoMethodError` class.
#[inline]
pub fn no_method_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eNoMethodError) }
}

/// Return Ruby's `NotImpError` class.
#[inline]
pub fn not_imp_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eNotImpError) }
}

/// Return Ruby's `RangeError` class.
#[inline]
pub fn range_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eRangeError) }
}

/// Return Ruby's `RegexpError` class.
#[inline]
pub fn regexp_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eRegexpError) }
}

/// Return Ruby's `RuntimeError` class.
#[inline]
pub fn runtime_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eRuntimeError) }
}

/// Return Ruby's `ScriptError` class.
#[inline]
pub fn script_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eScriptError) }
}

/// Return Ruby's `SecurityError` class.
#[inline]
pub fn security_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eSecurityError) }
}

/// Return Ruby's `SignalException` class.
#[inline]
pub fn signal() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eSignal) }
}

/// Return Ruby's `StandardError` class.
#[inline]
pub fn standard_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eStandardError) }
}

/// Return Ruby's `StopIteration` class.
#[inline]
pub fn stop_iteration() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eStopIteration) }
}

/// Return Ruby's `SyntaxError` class.
#[inline]
pub fn syntax_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eSyntaxError) }
}

/// Return Ruby's `SystemStackError` class.
#[inline]
pub fn sys_stack_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eSysStackError) }
}

/// Return Ruby's `SystemCallError` class.
#[inline]
pub fn system_call_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eSystemCallError) }
}

/// Return Ruby's `SystemExit` class.
#[inline]
pub fn system_exit() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eSystemExit) }
}

/// Return Ruby's `ThreadError` class.
#[inline]
pub fn thread_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eThreadError) }
}

/// Return Ruby's `TypeError` class.
#[inline]
pub fn type_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eTypeError) }
}

/// Return Ruby's `ZeroDivisionError` class.
#[inline]
pub fn zero_div_error() -> ExceptionClass {
    unsafe { ExceptionClass::from_rb_value_unchecked(rb_eZeroDivError) }
}
