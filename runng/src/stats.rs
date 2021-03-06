/*!
Runtime statistics

Statistics are organized as a tree.  Starting at the root, nodes may have a child whose siblings are likewise children of the parent.

The lifetime of the children is bound to that of the root.  This won't compile:
```no_compile
use log::{debug};
use runng::{stats::{NngStat, NngStatChild, NngStatRoot}};
let mut child: Option<NngStatChild> = None;
{
    let root = NngStatRoot::new().unwrap();
    child = root.child();
}
debug!("Name = {}", child.unwrap().name().unwrap());
```

## Examples

```rust
use log::{debug};
use runng::{
    *,
    stats::NngStat,
    stats::NngStatRoot,
};

#[test]
fn stats_example() -> NngReturn {
    // https://github.com/nanomsg/nng/issues/841
    let url = "inproc://test";
    let factory = Latest::default();
    let _pusher = factory.pusher_open()?.listen(&url)?;
    let _puller = factory.puller_open()?.dial(&url)?;

    let stats = NngStatRoot::new()?;
    let child = stats.child().unwrap();
    for stat in child.iter() {
        debug!("{}", stat.name().unwrap());
    }
    Ok(())
}
```
*/

use crate::*;
use runng_sys::*;
use std::marker;

/// Type of statistic.  See `NngStatChild::stat_type`.
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum NngStatType {
    Scope = nng_stat_type_enum_NNG_STAT_SCOPE as i32,
    Level = nng_stat_type_enum_NNG_STAT_LEVEL as i32,
    Counter = nng_stat_type_enum_NNG_STAT_COUNTER as i32,
    String = nng_stat_type_enum_NNG_STAT_STRING as i32,
    Boolean = nng_stat_type_enum_NNG_STAT_BOOLEAN as i32,
    Id = nng_stat_type_enum_NNG_STAT_ID as i32,
}

impl NngStatType {
    /// Converts value returned by [nng_stat_type](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_type.3) into `NngStatType`.
    pub fn from_i32(value: i32) -> Option<NngStatType> {
        match value {
            value if value == NngStatType::Scope as i32 => Some(NngStatType::Scope),
            value if value == NngStatType::Level as i32 => Some(NngStatType::Level),
            value if value == NngStatType::Counter as i32 => Some(NngStatType::Counter),
            value if value == NngStatType::String as i32 => Some(NngStatType::String),
            value if value == NngStatType::Boolean as i32 => Some(NngStatType::Boolean),
            value if value == NngStatType::Id as i32 => Some(NngStatType::Id),
            _ => None,
        }
    }
}

/// Unit of quantity measured by statistic.  See `NngStatChild::unit()`.
#[derive(Clone, Copy, Debug)]
#[repr(i32)]
pub enum NngStatUnit {
    None = nng_unit_enum_NNG_UNIT_NONE as i32,
    Bytes = nng_unit_enum_NNG_UNIT_BYTES as i32,
    Messages = nng_unit_enum_NNG_UNIT_MESSAGES as i32,
    Millis = nng_unit_enum_NNG_UNIT_MILLIS as i32,
    Events = nng_unit_enum_NNG_UNIT_EVENTS as i32,
}

impl NngStatUnit {
    /// Converts value returned by [nng_stat_unit](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_unit.3) into `NngStatUnit`.
    pub fn from_i32(value: i32) -> Option<NngStatUnit> {
        match value {
            value if value == NngStatUnit::None as i32 => Some(NngStatUnit::None),
            value if value == NngStatUnit::Bytes as i32 => Some(NngStatUnit::Bytes),
            value if value == NngStatUnit::Messages as i32 => Some(NngStatUnit::Messages),
            value if value == NngStatUnit::Millis as i32 => Some(NngStatUnit::Millis),
            value if value == NngStatUnit::Events as i32 => Some(NngStatUnit::Events),
            _ => None,
        }
    }
}

pub trait NngStat {
    /// Obtain underlying [`nng_stat`](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat.5).
    unsafe fn nng_stat(&self) -> *mut nng_stat;
    /// Returns the first child statistic, if any.
    fn child(&self) -> Option<NngStatChild> {
        unsafe {
            let node = nng_stat_child(self.nng_stat());
            if node.is_null() {
                None
            } else {
                Some(NngStatChild::new(node))
            }
        }
    }
}

/* Root of tree of statistics snapshot.
## Examples
```rust,no_run
use runng::{stats::NngStat, stats::NngStatRoot};
let child = NngStatRoot::new().unwrap().child();
```
*/
pub struct NngStatRoot<'root> {
    node: *mut nng_stat,
    _phantom: marker::PhantomData<&'root nng_stat>,
}

impl<'root> NngStatRoot<'root> {
    /// Get statistics snapshot.  See [nng_stats_get](https://nanomsg.github.io/nng/man/v1.1.0/nng_stats_get.3).
    pub fn create() -> NngResult<NngStatRoot<'root>> {
        unsafe {
            let mut node: *mut nng_stat = std::ptr::null_mut();
            let res = nng_stats_get(&mut node);
            NngFail::succeed_then(res, || NngStatRoot {
                node,
                _phantom: marker::PhantomData,
            })
        }
    }
}

impl<'root> NngStat for NngStatRoot<'root> {
    unsafe fn nng_stat(&self) -> *mut nng_stat {
        self.node
    }
}

impl<'root> Drop for NngStatRoot<'root> {
    fn drop(&mut self) {
        unsafe {
            //trace!("Drop NngStatRoot");
            nng_stats_free(self.node)
        }
    }
}

pub struct NngStatChild<'root> {
    node: *mut nng_stat,
    _phantom: marker::PhantomData<&'root nng_stat>,
}

/// Child of statistic node in tree of statistics.  See `NngStat::child()`.
impl<'root> NngStatChild<'root> {
    pub fn new(node: *mut nng_stat) -> NngStatChild<'root> {
        NngStatChild {
            node,
            _phantom: marker::PhantomData,
        }
    }
    /// See [nng_stat_name](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_name.3).
    pub fn name(&self) -> Result<&str, std::str::Utf8Error> {
        unsafe {
            let ptr = nng_stat_name(self.nng_stat());
            std::ffi::CStr::from_ptr(ptr).to_str()
        }
    }

    /// See [nng_stat_desc](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_desc.3).
    pub fn desc(&self) -> Result<&str, std::str::Utf8Error> {
        unsafe {
            let ptr = nng_stat_desc(self.nng_stat());
            std::ffi::CStr::from_ptr(ptr).to_str()
        }
    }

    /// See [nng_stat_type](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_type.3).
    pub fn stat_type(&self) -> Option<NngStatType> {
        unsafe {
            let val = nng_stat_type(self.nng_stat());
            NngStatType::from_i32(val)
        }
    }

    /// See [nng_stat_value](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_timestamp.3).
    pub fn value(&self) -> u64 {
        unsafe { nng_stat_value(self.nng_stat()) }
    }

    /// If the statistic type is of type `NNG_STAT_STRING` returns the string value.
    /// Otherwise, `None` is returned.
    /// See [nng_stat_string](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_string.3).
    pub fn string(&self) -> Option<&str> {
        unsafe {
            let ptr = nng_stat_string(self.nng_stat());
            if ptr.is_null() {
                return None;
            }
            let string = std::ffi::CStr::from_ptr(ptr).to_str();
            if let Ok(string) = string {
                Some(string)
            } else {
                None
            }
        }
    }

    /// See [nng_stat_unit](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_unit.3).
    pub fn unit(&self) -> Option<NngStatUnit> {
        unsafe {
            let val = nng_stat_unit(self.nng_stat());
            NngStatUnit::from_i32(val)
        }
    }

    /// See [nng_stat_timestamp](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_timestamp.3).
    pub fn timestamp(&self) -> u64 {
        unsafe { nng_stat_timestamp(self.nng_stat()) }
    }

    /// Returns an iterator over sibling statistics.  See [nng_stat_next](https://nanomsg.github.io/nng/man/v1.1.0/nng_stat_next.3).
    pub fn iter(&self) -> Iter {
        unsafe {
            let node = self.nng_stat();
            Iter {
                node: Some(NngStatChild::new(node)),
            }
        }
    }

    // The explicit `'root` lifetime is important here so the lifetime is the
    // top-level `NngStatRoot` rather than &self.
    pub fn next(&self) -> Option<NngStatChild<'root>> {
        unsafe {
            let node = self.nng_stat();
            let node = nng_stat_next(node);
            if node.is_null() {
                None
            } else {
                Some(NngStatChild::new(node))
            }
        }
    }
}

impl<'root> NngStat for NngStatChild<'root> {
    unsafe fn nng_stat(&self) -> *mut nng_stat {
        self.node
    }
}

/// Iterator over sibling statistics
pub struct Iter<'root> {
    node: Option<NngStatChild<'root>>,
}

impl<'root> Iterator for Iter<'root> {
    type Item = NngStatChild<'root>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.node.take();
        if let Some(ref node) = next {
            self.node = node.next();
        }
        next
    }
}
