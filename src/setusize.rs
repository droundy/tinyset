// Copyright 2019 David Roundy
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

//! A set that is compact in size.

/// A set for usize elements.
#[cfg(target_pointer_width = "64")]
pub type SetUsize = crate::setu64::SetU64;

#[cfg(target_pointer_width = "32")]
pub type SetUsize = crate::setu32::SetU32;
