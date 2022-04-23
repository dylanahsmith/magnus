//! Types and functions for working with Ruby classes.

use std::{fmt, ops::Deref, os::raw::c_int};

use rb_sys::{
    self, rb_cArray, rb_cBasicObject, rb_cBinding, rb_cClass, rb_cComplex, rb_cDir, rb_cEncoding,
    rb_cEnumerator, rb_cFalseClass, rb_cFile, rb_cFloat, rb_cHash, rb_cIO, rb_cInteger, rb_cMatch,
    rb_cMethod, rb_cModule, rb_cNameErrorMesg, rb_cNilClass, rb_cNumeric, rb_cObject, rb_cProc,
    rb_cRandom, rb_cRange, rb_cRational, rb_cRegexp, rb_cStat, rb_cString, rb_cStruct, rb_cSymbol,
    rb_cThread, rb_cTime, rb_cTrueClass, rb_cUnboundMethod, rb_class_new, rb_class_new_instance,
    ruby_value_type, VALUE,
};

#[cfg(ruby_gte_3_1)]
use rb_sys::rb_cRefinement;

use crate::{
    debug_assert_value,
    error::{protect, Error},
    exception,
    module::Module,
    object::Object,
    try_convert::{ArgList, TryConvert},
    value::{private, NonZeroValue, ReprValue, Value},
};

/// A Value pointer to a RClass struct, Ruby's internal representation of
/// classes.
///
/// See the [`Module`] trait for defining instance methods and nested
/// classes/modules.
/// See the [`Object`] trait for defining singlton methods (aka class methods).
///
/// All [`Value`] methods should be available on this type through [`Deref`],
/// but some may be missed by this documentation.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct RClass(NonZeroValue);

impl RClass {
    /// Return `Some(RClass)` if `val` is a `RClass`, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use magnus::{eval, RClass};
    /// # let _cleanup = unsafe { magnus::embed::init() };
    ///
    /// assert!(RClass::from_value(eval("String").unwrap()).is_some());
    /// assert!(RClass::from_value(eval("Enumerable").unwrap()).is_none());
    /// assert!(RClass::from_value(eval("nil").unwrap()).is_none());
    /// ```
    #[inline]
    pub fn from_value(val: Value) -> Option<Self> {
        unsafe {
            (val.rb_type() == ruby_value_type::RUBY_T_CLASS)
                .then(|| Self(NonZeroValue::new_unchecked(val)))
        }
    }

    #[inline]
    pub(crate) unsafe fn from_rb_value_unchecked(val: VALUE) -> Self {
        Self(NonZeroValue::new_unchecked(Value::new(val)))
    }

    /// Create a new anonymous class.
    ///
    /// # Examples
    ///
    /// ```
    /// use magnus::{class, RClass};
    /// # let _cleanup = unsafe { magnus::embed::init() };
    ///
    /// let class = RClass::new(Default::default()).unwrap();
    /// assert!(class.is_kind_of(class::class()));
    /// ```
    pub fn new(superclass: RClass) -> Result<RClass, Error> {
        debug_assert_value!(superclass);
        let superclass = superclass.as_rb_value();
        unsafe {
            let res = protect(|| Value::new(rb_class_new(superclass)));
            res.map(|v| Self::from_rb_value_unchecked(v.as_rb_value()))
        }
    }

    /// Create a new object, an instance of `self`, passing the arguments
    /// `args` to the initialiser.
    ///
    /// # Examples
    ///
    /// ```
    /// use magnus::class;
    /// # let _cleanup = unsafe { magnus::embed::init() };
    ///
    /// let s = class::string().new_instance(()).unwrap();
    /// assert!(s.is_kind_of(class::string()));
    /// assert_eq!(s.to_string(), "");
    /// ```
    pub fn new_instance<T>(self, args: T) -> Result<Value, Error>
    where
        T: ArgList,
    {
        let args = args.into_arg_list();
        let slice = args.as_ref();
        unsafe {
            protect(|| {
                Value::new(rb_class_new_instance(
                    slice.len() as c_int,
                    slice.as_ptr() as *const VALUE,
                    self.as_rb_value(),
                ))
            })
        }
    }
}

impl Default for RClass {
    fn default() -> Self {
        unsafe { RClass::from_rb_value_unchecked(rb_sys::rb_cObject) }
    }
}

impl Deref for RClass {
    type Target = Value;

    fn deref(&self) -> &Self::Target {
        self.0.get_ref()
    }
}

impl fmt::Display for RClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.to_s_infallible() })
    }
}

impl fmt::Debug for RClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inspect())
    }
}

impl From<RClass> for Value {
    fn from(val: RClass) -> Self {
        *val
    }
}

impl Object for RClass {}
impl Module for RClass {}

unsafe impl private::ReprValue for RClass {
    fn to_value(self) -> Value {
        *self
    }

    unsafe fn from_value_unchecked(val: Value) -> Self {
        Self(NonZeroValue::new_unchecked(val))
    }
}

impl ReprValue for RClass {}

impl TryConvert for RClass {
    #[inline]
    fn try_convert(val: &Value) -> Result<Self, Error> {
        match Self::from_value(*val) {
            Some(v) => Ok(v),
            None => Err(Error::new(
                exception::type_error(),
                format!("no implicit conversion of {} into Class", unsafe {
                    val.classname()
                },),
            )),
        }
    }
}

/// Return Ruby's `Array` class.
#[inline]
pub fn array() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cArray) }
}

/// Return Ruby's `BasicObject` class.
#[inline]
pub fn basic_object() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cBasicObject) }
}

/// Return Ruby's `Binding` class.
#[inline]
pub fn binding() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cBinding) }
}

/// Return Ruby's `Class` class.
#[inline]
pub fn class() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cClass) }
}

/// Return Ruby's `Complex` class.
#[inline]
pub fn complex() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cComplex) }
}

/// Return Ruby's `Dir` class.
#[inline]
pub fn dir() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cDir) }
}

/// Return Ruby's `Encoding` class.
#[inline]
pub fn encoding() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cEncoding) }
}

/// Return Ruby's `Enumerator` class.
#[inline]
pub fn enumerator() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cEnumerator) }
}

/// Return Ruby's `FalseClass` class.
#[inline]
pub fn false_class() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cFalseClass) }
}

/// Return Ruby's `File` class.
#[inline]
pub fn file() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cFile) }
}

/// Return Ruby's `Float` class.
#[inline]
pub fn float() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cFloat) }
}

/// Return Ruby's `Hash` class.
#[inline]
pub fn hash() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cHash) }
}

/// Return Ruby's `IO` class.
#[inline]
pub fn io() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cIO) }
}

/// Return Ruby's `Integer` class.
#[inline]
pub fn integer() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cInteger) }
}

/// Return Ruby's `MatchData` class.
#[inline]
pub fn match_class() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cMatch) }
}

/// Return Ruby's `Method` class.
#[inline]
pub fn method() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cMethod) }
}

/// Return Ruby's `Module` class.
#[inline]
pub fn module() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cModule) }
}

/// Return Ruby's `NameError::Message` class.
#[inline]
pub fn name_error_mesg() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cNameErrorMesg) }
}

/// Return Ruby's `NilClass` class.
#[inline]
pub fn nil_class() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cNilClass) }
}

/// Return Ruby's `Numeric` class.
#[inline]
pub fn numeric() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cNumeric) }
}

/// Return Ruby's `Object` class.
#[inline]
pub fn object() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cObject) }
}

/// Return Ruby's `Proc` class.
#[inline]
pub fn proc() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cProc) }
}

/// Return Ruby's `Random` class.
#[inline]
pub fn random() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cRandom) }
}

/// Return Ruby's `Range` class.
#[inline]
pub fn range() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cRange) }
}

/// Return Ruby's `Rational` class.
#[inline]
pub fn rational() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cRational) }
}

/// Return Ruby's `Refinement` class.
#[cfg(ruby_gte_3_1)]
#[inline]
pub fn refinement() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cRefinement) }
}

/// Return Ruby's `Regexp` class.
#[inline]
pub fn regexp() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cRegexp) }
}

/// Return Ruby's `File::Stat` class.
#[inline]
pub fn stat() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cStat) }
}

/// Return Ruby's `String` class.
#[inline]
pub fn string() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cString) }
}

/// Return Ruby's `Struct` class.
#[inline]
pub fn struct_class() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cStruct) }
}

/// Return Ruby's `Symbol` class.
#[inline]
pub fn symbol() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cSymbol) }
}

/// Return Ruby's `Thread` class.
#[inline]
pub fn thread() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cThread) }
}

/// Return Ruby's `Time` class.
#[inline]
pub fn time() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cTime) }
}

/// Return Ruby's `TrueClass` class.
#[inline]
pub fn true_class() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cTrueClass) }
}

/// Return Ruby's `UnboundMethod` class.
#[inline]
pub fn unbound_method() -> RClass {
    unsafe { RClass::from_rb_value_unchecked(rb_cUnboundMethod) }
}
