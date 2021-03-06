use std::{
    convert::{TryFrom, TryInto},
    ffi::{c_void, CStr},
    os::raw::{c_char, c_int},
};

use binn_sys::binn_ptr;

#[non_exhaustive]
#[derive(Debug)]
pub enum BinnValue<'a> {
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Bool(bool),
    Str(&'a CStr),
    Object(BinnObject),
}

macro_rules! impl_from {
    ($t:ty, $enum:ident) => {
        impl<'a> From<$t> for BinnValue<'a> {
            fn from(x: $t) -> Self {
                BinnValue::$enum(x)
            }
        }
    };
}

impl_from!(i8, Int8);
impl_from!(i16, Int16);
impl_from!(i32, Int32);
impl_from!(i64, Int64);
impl_from!(u8, UInt8);
impl_from!(u16, UInt16);
impl_from!(u32, UInt32);
impl_from!(u64, UInt64);
impl_from!(f32, Float32);
impl_from!(f64, Float64);
impl_from!(bool, Bool);
impl_from!(&'a CStr, Str);
impl_from!(BinnObject, Object);

#[derive(Debug)]
pub struct WrongBinnValue;

macro_rules! impl_tryfrom {
    ($t:ty, $enum:ident) => {
        impl<'a> TryFrom<BinnValue<'a>> for $t {
            type Error = WrongBinnValue;

            fn try_from(value: BinnValue<'a>) -> Result<Self, Self::Error> {
                if let BinnValue::$enum(x) = value {
                    Ok(x)
                } else {
                    Err(WrongBinnValue)
                }
            }
        }
    };
}

impl_tryfrom!(i8, Int8);
impl_tryfrom!(i16, Int16);
impl_tryfrom!(i32, Int32);
impl_tryfrom!(i64, Int64);
impl_tryfrom!(u8, UInt8);
impl_tryfrom!(u16, UInt16);
impl_tryfrom!(u32, UInt32);
impl_tryfrom!(u64, UInt64);
impl_tryfrom!(f32, Float32);
impl_tryfrom!(f64, Float64);
impl_tryfrom!(bool, Bool);
impl_tryfrom!(&'a CStr, Str);
impl_tryfrom!(BinnObject, Object);

#[derive(Debug)]
pub struct BinnObject(*mut binn_sys::binn);

impl<'a> BinnObject {
    pub fn new() -> Self {
        unsafe {
            let mut obj = binn_sys::binn_object();
            (*obj).disable_int_compression = true as i32;
            Self(obj)
        }
    }

    pub fn set<T: Into<BinnValue<'a>>>(&mut self, key: &CStr, value: T) {
        fn addr<T>(x: &T) -> *mut c_void {
            x as *const T as *mut c_void
        }
        match value.into() {
            BinnValue::Int8(x) => self.set_object(key, binn_sys::BINN_INT8, addr(&x), 0),
            BinnValue::Int16(x) => self.set_object(key, binn_sys::BINN_INT16, addr(&x), 0),
            BinnValue::Int32(x) => self.set_object(key, binn_sys::BINN_INT32, addr(&x), 0),
            BinnValue::Int64(x) => self.set_object(key, binn_sys::BINN_INT64, addr(&x), 0),
            BinnValue::UInt8(x) => self.set_object(key, binn_sys::BINN_UINT8, addr(&x), 0),
            BinnValue::UInt16(x) => self.set_object(key, binn_sys::BINN_UINT16, addr(&x), 0),
            BinnValue::UInt32(x) => self.set_object(key, binn_sys::BINN_UINT32, addr(&x), 0),
            BinnValue::UInt64(x) => self.set_object(key, binn_sys::BINN_UINT64, addr(&x), 0),
            BinnValue::Float32(x) => self.set_object(key, binn_sys::BINN_FLOAT32, addr(&x), 0),
            BinnValue::Float64(x) => self.set_object(key, binn_sys::BINN_FLOAT64, addr(&x), 0),
            BinnValue::Bool(x) => self.set_object(key, binn_sys::BINN_BOOL, addr(&x), 0),
            BinnValue::Str(x) => {
                self.set_object(key, binn_sys::BINN_STRING, x.as_ptr() as *mut c_void, 0)
            }
            BinnValue::Object(x) => {
                let bytes = x.as_bytes();
                let ptr = bytes.as_ptr() as *mut c_void;
                let size = bytes.len();
                self.set_object(key, binn_sys::BINN_OBJECT, ptr, size)
            }
        };
    }

    fn set_object(&mut self, key: &CStr, ty: u32, value: *mut c_void, size: usize) {
        unsafe { binn_sys::binn_object_set(self.0, key.as_ptr(), ty as i32, value, size as i32) };
    }

    pub fn get(&self, key: &CStr) -> Option<BinnValue> {
        unsafe {
            let mut ptype: c_int = 0;
            let mut psize: c_int = 0;

            let ptr = binn_ptr(self.0 as *mut c_void);
            let pval = binn_sys::binn_object_read(
                ptr,
                key.as_ptr(),
                &mut ptype as *mut c_int,
                &mut psize as *mut c_int,
            );

            match ptype as u32 {
                binn_sys::BINN_INT8 => (pval as *const i8).as_ref().map(|p| BinnValue::Int8(*p)),
                binn_sys::BINN_INT16 => (pval as *const i16).as_ref().map(|p| BinnValue::Int16(*p)),
                binn_sys::BINN_INT32 => (pval as *const i32).as_ref().map(|p| BinnValue::Int32(*p)),
                binn_sys::BINN_INT64 => (pval as *const i64).as_ref().map(|p| BinnValue::Int64(*p)),
                binn_sys::BINN_UINT8 => (pval as *const u8).as_ref().map(|p| BinnValue::UInt8(*p)),
                binn_sys::BINN_UINT16 => {
                    (pval as *const u16).as_ref().map(|p| BinnValue::UInt16(*p))
                }
                binn_sys::BINN_UINT32 => {
                    (pval as *const u32).as_ref().map(|p| BinnValue::UInt32(*p))
                }
                binn_sys::BINN_UINT64 => {
                    (pval as *const u64).as_ref().map(|p| BinnValue::UInt64(*p))
                }
                binn_sys::BINN_FLOAT32 => (pval as *const f32)
                    .as_ref()
                    .map(|p| BinnValue::Float32(*p)),
                binn_sys::BINN_FLOAT64 => (pval as *const f64)
                    .as_ref()
                    .map(|p| BinnValue::Float64(*p)),
                binn_sys::BINN_BOOL => (pval as *const bool).as_ref().map(|p| BinnValue::Bool(*p)),
                binn_sys::BINN_STRING => (pval as *const c_char)
                    .as_ref()
                    .map(|p| BinnValue::Str(CStr::from_ptr(p))),
                binn_sys::BINN_OBJECT => {
                    let bytes = std::slice::from_raw_parts(pval as *const u8, psize as usize);
                    TryInto::<BinnObject>::try_into(bytes)
                        .ok()
                        .map(BinnValue::Object)
                }
                _ => None,
            }
        }
    }

    pub fn get_as<T: TryFrom<BinnValue<'a>>>(&'a self, key: &CStr) -> Option<T> {
        self.get(key).and_then(|v| v.try_into().ok())
    }
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let ptr = binn_sys::binn_ptr(self.0 as *mut c_void);
            let size = binn_sys::binn_size(self.0 as *mut c_void) as usize;
            std::slice::from_raw_parts(ptr as *const u8, size)
        }
    }
}

impl Drop for BinnObject {
    fn drop(&mut self) {
        unsafe { binn_sys::binn_free(self.0) };
    }
}

impl Default for BinnObject {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct BinnOpenError;

impl TryFrom<&[u8]> for BinnObject {
    type Error = BinnOpenError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let binn = unsafe { binn_sys::binn_open(data.as_ptr() as *mut c_void) };
        if binn.is_null() {
            Err(BinnOpenError)
        } else {
            Ok(BinnObject(binn))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::TryInto, ffi::CString};

    use super::*;

    #[test]
    fn create_object_test() {
        let _binn = BinnObject::new();
    }

    #[test]
    fn setters_getters_test() {
        let mut binn = BinnObject::new();

        let k = |s: &str| -> CString { CString::new(s).unwrap() };
        let hello = CStr::from_bytes_with_nul(b"hello\0").unwrap();

        binn.set(&k("i8"), 42i8);
        binn.set(&k("i16"), 42i16);
        binn.set(&k("i32"), 42i32);
        binn.set(&k("i64"), 42i64);
        binn.set(&k("u8"), 42u8);
        binn.set(&k("u16"), 42u16);
        binn.set(&k("u32"), 42u32);
        binn.set(&k("u64"), 42u64);
        binn.set(&k("f32"), 3.14f32);
        binn.set(&k("f64"), 3.14f64);
        binn.set(&k("bool"), true);
        binn.set(&k("str"), hello);

        assert_eq!(binn.get_as::<i8>(&k("i8")), Some(42));
        assert_eq!(binn.get_as::<i16>(&k("i16")), Some(42));
        assert_eq!(binn.get_as::<i32>(&k("i32")), Some(42));
        assert_eq!(binn.get_as::<i64>(&k("i64")), Some(42));
        assert_eq!(binn.get_as::<u8>(&k("u8")), Some(42));
        assert_eq!(binn.get_as::<u16>(&k("u16")), Some(42));
        assert_eq!(binn.get_as::<u32>(&k("u32")), Some(42));
        assert_eq!(binn.get_as::<u64>(&k("u64")), Some(42));
        assert_eq!(binn.get_as::<f32>(&k("f32")), Some(3.14));
        assert_eq!(binn.get_as::<f64>(&k("f64")), Some(3.14));
        assert_eq!(binn.get_as::<bool>(&k("bool")), Some(true));
        assert_eq!(binn.get_as::<&CStr>(&k("str")), Some(hello));
        assert_eq!(binn.get_as::<bool>(&k("random")), None);
    }

    #[test]
    fn as_bytes_test() {
        let mut binn = BinnObject::new();

        let k = |s: &str| -> CString { CString::new(s).unwrap() };
        let hello = CStr::from_bytes_with_nul(b"hello\0").unwrap();

        binn.set(&k("i8"), 42i8);
        binn.set(&k("i16"), 42i16);
        binn.set(&k("i32"), 42i32);
        binn.set(&k("i64"), 42i64);
        binn.set(&k("u8"), 42u8);
        binn.set(&k("u16"), 42u16);
        binn.set(&k("u32"), 42u32);
        binn.set(&k("u64"), 42u64);
        binn.set(&k("f32"), 3.14f32);
        binn.set(&k("f64"), 3.14f64);
        binn.set(&k("bool"), true);
        binn.set(&k("str"), hello.as_ref());

        let other_binn: BinnObject = binn.as_bytes().try_into().expect("failed to deserialize");

        assert_eq!(other_binn.get_as::<i8>(&k("i8")), Some(42));
        assert_eq!(other_binn.get_as::<i16>(&k("i16")), Some(42));
        assert_eq!(other_binn.get_as::<i32>(&k("i32")), Some(42));
        assert_eq!(other_binn.get_as::<i64>(&k("i64")), Some(42));
        assert_eq!(other_binn.get_as::<u8>(&k("u8")), Some(42));
        assert_eq!(other_binn.get_as::<u16>(&k("u16")), Some(42));
        assert_eq!(other_binn.get_as::<u32>(&k("u32")), Some(42));
        assert_eq!(other_binn.get_as::<u64>(&k("u64")), Some(42));
        assert_eq!(other_binn.get_as::<f32>(&k("f32")), Some(3.14));
        assert_eq!(other_binn.get_as::<f64>(&k("f64")), Some(3.14));
        assert_eq!(other_binn.get_as::<bool>(&k("bool")), Some(true));
        assert_eq!(other_binn.get_as::<&CStr>(&k("str")), Some(hello));
        assert_eq!(other_binn.get_as::<bool>(&k("random")), None);
    }

    #[test]
    fn recursive_object_test() {
        let mut outer = BinnObject::new();
        let mut inner = BinnObject::new();

        let k = |s: &str| -> CString { CString::new(s).unwrap() };

        inner.set(&k("val"), 42i8);
        outer.set(&k("obj"), inner);

        assert_eq!(
            outer
                .get_as::<BinnObject>(&k("obj"))
                .and_then(|o| o.get_as::<i8>(&k("val"))),
            Some(42)
        );
    }
}
