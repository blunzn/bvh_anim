// Copyright © 2019-2020 George Burton
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software
// and associated documentation files (the "Software"), to deal in the Software without restriction,
// including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or substantial
// portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT
// LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN
// NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
// WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
// SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#![warn(missing_docs)]
#![deny(bare_trait_objects, unsafe_code)]

//! # About this library
//!
//! A small library for loading and manipulating BioVision motion files.
//!
//! ## The bvh file format
//!
//! The `Bvh` file format is comprised of two main sections: the 'Heirarchy' section,
//! which defines the joints of the skeleton, and the 'Motion' section, which defines
//! the motion values for each channel.
//!
//! This project contains some samples in the [`data` directory][`data` directory].
//!
//! ### Heierarchy
//!
//! The 'Heirarchy' section defines the skeleton as a tree of joints, where there is
//! a single root joint, and a chain of child joints extending out from each joint,
//! terminated by an 'End Site' section.
//!
//! Each joint has:
//!
//! * A list of channels, which are the degrees of freedom in which the joint may move.
//!   Channels are listed in the order in which the transformation should be applied
//!   to the global transform for the root.
//! * An offset, which is the vector distance from the parent joint.
//! * An optional end site, which is used to cap off a chain of joints. This is only used
//!   to calculate the length of the final bone in the chain.
//!
//! ```text
//! HEIRARCHY
//! ROOT <Root-name>
//! {
//!     OFFSET <Root-offset-x> <Root-offset-y> <Root-offset-z>
//!     CHANNELS <Num-root-joint-channels> Xposition Yposition <other-root-channels ...>
//!     JOINT <Joint-1-name>
//!     {
//!         OFFSET <Joint-1-offset-x> <Joint-1-offset-y> <Joint-1-offset-z>
//!         CHANNELS <Num-joint-1-channels> <Joint-1-channels ...>
//!         JOINT <Joint-2-name>
//!         {
//!             OFFSET <Joint-2-offset-x> <Joint-2-offset-y> <Joint-2-offset-z>
//!             CHANNELS <Num-joint-2-channels> <Joint-2-channels ...>
//!             JOINT <Joint-with-end-site>
//!             {
//!                 OFFSET ...
//!                 CHANNELS ...
//!                 End Site
//!                 {
//!                      OFFSET <end-site-offset-x> <end-site-offset-y> <end-site-offset-z>
//!                 }
//!             }
//!             ... More child joints
//!         }
//!         JOINT <Joint-3-name>
//!         {
//!             OFFSET <Joint-3-offset-x> <Joint-3-offset-y> <Joint-3-offset-z>
//!             CHANNELS <Num-joint-3-channels> <Joint-3-channels ...>
//!             ... More child joints
//!         }
//!         ... More child joints
//!     }
//!     ... More child joints
//! }
//! ```
//!
//! Note that the bvh data is defined in terms of a right-handed coordinate system, where
//! the positive y-axis is the up vector.
//!
//! ### Motion
//!
//! The `MOTION` section of the bvh file records the number of frames, the frame time, and
//! defines the full range of motions for each channel, frame by frame.
//!
//! ```text
//! MOTION
//! Frames: <num-frames>
//! Frame Time: <frame-time>
//! <frame-0-channel-0-value> <frame-0-channel-1-value> <frame-0-channel-2-value> ...
//! <frame-1-channel-0-value> <frame-1-channel-1-value> <frame-1-channel-2-value> ...
//! <frame-2-channel-0-value> <frame-2-channel-1-value> <frame-2-channel-2-value> ...
//! ⋮
//! ```
//!
//! The frame time is recorded in seconds, and tells the animation system how long each frame
//! of the animation should last for. This value is usually around 0.033333333, which is close
//! to 30 frames per second.
//!
//! The list of motion values is a matrix, where each row represents a frame. Each column of
//! the row represents a transformation around the channel axis - for example a motion value
//! of 130.0 for an `Xposition` channel would correspond to a rotation of 130.0 degrees around
//! the x-axis.
//!
//! Note that rotations are conventionally in degrees, although it will be up to your application
//! how to interpret each motion's value.
//!
//! ## Using this library.
//!
//! ### Creating a [`Bvh`][`Bvh`] struct:
//!
//! There are a few ways to create a [`Bvh`][`Bvh`] struct:
//!
//! * You can use the [`from_reader`][`from_reader`] function, which will parse a `BufRead`
//!   as a bvh file. The [`from_bytes`][`from_bytes`] function is a convenient wrapper function
//!   to parse an in-memory slice of bytes as a `bvh` file. Note that the file does not need to
//!   be strictly UTF-8, although it should be an ascii-compatible encoding. These functions are
//!   also available as associated methods on the `Bvh` type directly as [`Bvh::from_reader`]
//!   [`Bvh::from_reader`] and [`Bvh::from_bytes`][`Bvh::from_bytes`]
//!
//! * You can use the [`bvh!`][`bvh!`] macro to construct a [`Bvh`][`Bvh`] instance in your source files
//!   using the same syntax as you would use for a standard bvh file.
//!
//! * You can use the [`builder`][`builder`] module to dynamically construct a bvh. This is useful
//!   for converting data from other formats into a [`Bvh`][`Bvh`] struct.
//!
//! * You can create an empty [`Bvh`][`Bvh`] using the [`Bvh::new`][`Bvh::new`] or [`Default::default`]
//!   [`Default::default`] methods.
//!
//! ### Other operations:
//!
//! Once you have a valid [`Bvh`][`Bvh`] struct, there are a number of ways you can inspect and
//! manipulate it:
//!
//! * The [`Bvh::joints`][`Bvh::joints`] method can be used to iterate through each [`Joint`][`Joint`]
//!   of the [`Bvh`][`Bvh`]. Each [`Joint`][`Joint`] can be inspected through its [`JointData`]
//!   [`JointData`], which can be obtained with the [`Joint::data`][`Joint::data`] method.
//!
//! * The [`Bvh::frames`][`Bvh::frames`] method returns a [`Frames`][`Frames`] iterator over each
//!   frame of the animation. A [`Frame`][`Frame`] can only be indexed by a [`Channel`][`Channel`]
//!   belonging to an associated [`Joint`][`Joint`] of the [`Bvh`][`Bvh`], although you can convert
//!   it into an [`&[`][`slice`][`f32`][`f32`][`]`][`slice`] using the [`Frame::as_slice`][`Frame::as_slice`] method.
//!
//! * You can serialise the [`Bvh`][`Bvh`] into a [`Write`][`Write`] type using the [`Bvh::write_to`]
//!   [`Bvh::write_to`] method. There is also the [`Bvh::to_bstring`][`Bvh::to_bstring`] method, which
//!   converts the [`Bvh`][`Bvh`] into a [`BString`][`BString`]. Various aspects of the formatting
//!   can be customised using the [`WriteOptions`][`WriteOptions`] type, such as the line termination
//!   style, indentation method, and floating point accuracy.
//!
//! ## Examples
//!
//! This library comes with some example applications, which can be viewed on [Github][Github].
//!
//! ## Other resources
//!
//! * More information on this file format can be found [here][bvh_html].
//! * A large library of bvh files is freely available from [CMU's motion capture database]
//!   [CMU's motion capture database].
//!
//! [`data` directory]: https://github.com/burtonageo/bvh_anim/tree/master/data
//! [`bvh`]: struct.Bvh.html
//! [`from_reader`]: fn.from_reader.html
//! [`from_bytes`]: fn.from_bytes.html
//! [`Bvh::from_reader`]: struct.Bvh.html#method.from_reader
//! [`Bvh::from_bytes`]:  struct.Bvh.html#method.from_bytes
//! [`bvh!`]: macro.bvh.html
//! [`builder`]: builder/index.html
//! [`Bvh::new`]: struct.Bvh.html#method.new
//! [`Default::default`]: https://doc.rust-lang.org/stable/std/default/trait.Default.html#tymethod.default
//! [`Bvh::joints`]: struct.Bvh.html#method.joints
//! [`Joint`]: struct.Joint.html
//! [`JointData`]: enum.JointData.html
//! [`Joint::data`]: struct.Joint.html#method.data
//! [`Bvh::frames`]: struct.Bvh.html#method.frames
//! [`Frames`]: struct.Frames.html
//! [`Frame`]: struct.Frame.html
//! [`slice`]: https://doc.rust-lang.org/std/primitive.slice.html
//! [`f32`]: https://doc.rust-lang.org/stable/std/primitive.f32.html
//! [`Channel`]: struct.Channel.html
//! [`Frame::as_slice`]: struct.Frame.html#method.as_slice
//! [`Write`]: https://doc.rust-lang.org/stable/std/io/trait.Write.html
//! [`Bvh::write_to`]: struct.Bvh.html#method.write_to
//! [`Bvh::to_bstring`]: struct.Bvh.html#method.to_bstring
//! [`BString`]: https://docs.rs/bstr/0.1.2/bstr/struct.BString.html
//! [`WriteOptions`]: write/struct.WriteOptions.html
//! [Github]: https://github.com/burtonageo/bvh_anim/tree/master/examples
//! [bvh_html]: https://research.cs.wisc.edu/graphics/Courses/cs-838-1999/Jeff/BVH.html
//! [CMU's motion capture database]: https://sites.google.com/a/cgspeed.com/cgspeed/motion-capture/daz-friendly-release

#[macro_use]
mod macros;

pub mod errors;

pub mod write;

mod frame_cursor;
mod frame_iter;
mod joint;
mod parse;

use crate::{
    joint::JointData,
    errors::{LoadError, ParseChannelError, SetMotionError},
    frames::{FrameCursor, FrameIndex, Frames, FramesMut},
};
use bstr::{
    io::{BufReadExt, ByteLines},
    BStr, BString, ByteSlice,
};
use mint::Vector3;
use num_traits::{one, zero, One, Zero};
use std::{
    convert::TryFrom,
    fmt,
    io::{self, Cursor, Write},
    iter::Enumerate,
    ops::Index,
    str::{self, FromStr},
    time::Duration,
};

pub mod frames {
    //! This module contains types and functions used for accessing and modifying
    //! frame data.

    pub use crate::frame_cursor::FrameCursor;
    pub use crate::frame_iter::{Frame, FrameIndex, FrameMut, Frames, FramesMut};
}

pub use joint::{Joint, JointMut, Joints, JointsMut};
#[doc(hidden)]
pub use macros::BvhLiteralBuilder;


struct CachedEnumerate<I> {
    iter: Enumerate<I>,
    last_enumerator: Option<usize>,
}

impl<I> CachedEnumerate<I> {
    #[inline]
    fn new(iter: Enumerate<I>) -> Self {
        CachedEnumerate {
            iter,
            last_enumerator: None,
        }
    }

    #[inline]
    fn last_enumerator(&self) -> Option<usize> {
        self.last_enumerator
    }
}

impl<I: Iterator> Iterator for CachedEnumerate<I> {
    type Item = <Enumerate<I> as Iterator>::Item;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (curr, item) = self.iter.next()?;
        self.last_enumerator = Some(curr);
        Some((curr, item))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

type EnumeratedLines<'a> = CachedEnumerate<ByteLines<&'a mut dyn BufReadExt>>;

impl EnumeratedLines<'_> {
    pub(crate) fn next_non_empty_line(&mut self) -> Option<<Self as Iterator>::Item> {
        let mut next = self.next();
        loop {
            match next {
                None => return None,
                Some((idx, result)) => {
                    let string = match result {
                        Ok(s) => s,
                        Err(e) => return Some((idx, Err(e))),
                    };
                    if string.trim().is_empty() {
                        next = self.next()
                    } else {
                        return Some((idx, Ok(string)));
                    }
                }
            }
        }
    }
}

/// Loads the `Bvh` from the `reader`.
#[inline]
pub fn from_reader<R: BufReadExt>(data: R) -> Result<Bvh, LoadError> {
    Bvh::from_reader(data)
}

/// Parse a sequence of bytes as if it were an in-memory `Bvh` file.
///
/// # Examples
///
/// ```
/// # use bvh_anim::{self, from_bytes};
/// let bvh_string = br#"
///     HIERARCHY
///     ROOT Hips
///     {
///         OFFSET 0.0 0.0 0.0
///         CHANNELS 3 Xposition Yposition Zposition
///         End Site
///         {
///             OFFSET 0.0 0.0 0.0
///         }
///     }
///     MOTION
///     Frames: 1
///     Frame Time: 0.033333333
///     0.0 0.0 0.0
/// "#;
///
/// let bvh = from_bytes(&bvh_string[..])?;
/// # let _ = bvh;
/// # Result::<(), bvh_anim::errors::LoadError>::Ok(())
/// ```
#[inline]
pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Bvh, LoadError> {
    Bvh::from_bytes(bytes)
}

/// Parse a `str` as if it were an in-memory `Bvh` file.
///
/// # Examples
///
/// ```
/// # use bvh_anim::{self, from_str};
/// let bvh_string = "
///     HIERARCHY
///     ROOT Hips
///     {
///         OFFSET 0.0 0.0 0.0
///         CHANNELS 3 Xposition Yposition Zposition
///         End Site
///         {
///             OFFSET 0.0 0.0 0.0
///         }
///     }
///     MOTION
///     Frames: 1
///     Frame Time: 0.033333333
///     0.0 0.0 0.0
/// ";
///
/// let bvh = from_str(bvh_string)?;
/// # let _ = bvh;
/// # Result::<(), bvh_anim::errors::LoadError>::Ok(())
/// ```
#[inline]
pub fn from_str(string: &str) -> Result<Bvh, LoadError> {
    Bvh::from_str(string)
}

/// A complete `bvh` file.
///
/// See the [module documentation](index.html#using-this-library)
/// for more information.
#[derive(Clone, Debug, PartialEq)]
pub struct Bvh {
    /// The list of joints. If the root joint exists, it is always at
    /// index `0`.
    joints: Vec<JointData>,
    /// The motion values of the `Frame`.
    motion_values: Vec<f32>,
    /// The number of frames in the bvh.
    num_frames: usize,
    /// The number of `Channel`s in the bvh.
    num_channels: usize,
    /// The total time it takes to play one frame.
    frame_time: Duration,
}

impl Bvh {
    /// Create an empty `Bvh`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            joints: Vec::new(),
            motion_values: Vec::new(),
            num_frames: 0,
            num_channels: 0,
            frame_time: Duration::from_secs(0),
    }
    }

    /// Parse a sequence of bytes as if it were an in-memory `Bvh` file.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bvh_anim::{self, Bvh};
    /// let bvh_string = br#"
    ///     HIERARCHY
    ///     ROOT Hips
    ///     {
    ///         OFFSET 0.0 0.0 0.0
    ///         CHANNELS 3 Xposition Yposition Zposition
    ///         End Site
    ///         {
    ///             OFFSET 0.0 0.0 0.0
    ///         }
    ///     }
    ///     MOTION
    ///     Frames: 1
    ///     Frame Time: 0.033333333
    ///     0.0 0.0 0.0
    /// "#;
    ///
    /// let bvh = Bvh::from_bytes(&bvh_string[..])?;
    /// # let _ = bvh;
    /// # Result::<(), bvh_anim::errors::LoadError>::Ok(())
    /// ```
    #[inline]
    pub fn from_bytes<B: AsRef<[u8]>>(bytes: B) -> Result<Self, LoadError> {
        Bvh::from_reader(Cursor::new(bytes))
    }

    /// Loads the `Bvh` from the `reader`.
    pub fn from_reader<R: BufReadExt>(mut reader: R) -> Result<Self, LoadError> {
        Self::from_reader_(reader.by_ref())
    }

    fn from_reader_(reader: &mut dyn BufReadExt) -> Result<Self, LoadError> {
        let mut lines = CachedEnumerate::new(reader.byte_lines().enumerate());

        let mut bvh = Bvh::default();

        bvh.read_joints(&mut lines)?;
        bvh.read_motion(&mut lines)?;

        Ok(bvh)
    }

    /// Writes the `Bvh` using the `bvh` file format to the `writer`, with
    /// the default formatting options.
    ///
    /// # Notes
    ///
    /// To customise the formatting, see the [`WriteOptions`][`WriteOptions`] type.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use bvh_anim::bvh;
    /// # use std::io;
    /// # use std::fs::File;
    /// let bvh = bvh! {
    ///     // fields unspecified
    /// };
    ///
    /// let mut out_file = File::create("./out_file.bvh")?;
    /// bvh.write_to(&mut out_file)?;
    /// # Result::<(), io::Error>::Ok(())
    /// ```
    ///
    /// [`WriteOptions`]: write/struct.WriteOptions.html
    #[inline]
    pub fn write_to<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        write::WriteOptions::default().write(self, writer)
    }

    /// Writes the `Bvh` using the `bvh` file format into a `BString` with
    /// the default formatting options.
    ///
    /// # Notes
    ///
    /// To customise the formatting, see the [`WriteOptions`][`WriteOptions`] type.
    ///
    /// [`WriteOptions`]: write/struct.WriteOptions.html
    #[inline]
    pub fn to_bstring(&self) -> BString {
        write::WriteOptions::default().write_to_string(self)
    }

    /// Returns the root joint if it exists, or `None` if the skeleton is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bvh_anim::{Bvh, bvh};
    /// let bvh = Bvh::new();
    /// assert!(bvh.root_joint().is_none());
    ///
    /// let bvh = bvh! {
    ///     HIERARCHY
    ///     ROOT Hips
    ///     {
    /// #       OFFSET 0.0 0.0 0.0
    /// #       CHANNELS 0
    /// #       End Site
    /// #       {
    /// #           OFFSET 0.0 0.0 0.0
    /// #       }
    ///         // Joints...
    ///     }
    ///     MOTION
    /// #   Frames: 0
    /// #   Frame Time: 0.033333333
    ///     // Frames...
    /// };
    ///
    /// assert!(bvh.root_joint().is_some());
    /// ```
    #[inline]
    pub fn root_joint(&self) -> Option<Joint<'_>> {
        if self.joints.is_empty() {
            None
        } else {
            Some(Joint {
                index: 0,
                joints: &self.joints[..],
            })
        }
    }

    /// Returns an iterator over all the `Joint`s in the `Bvh`.
    #[inline]
    pub fn joints(&self) -> Joints<'_> {
        Joints::iter_root(&self.joints[..])
    }

    /// Returns a mutable iterator over all the joints in the `Bvh`.
    pub fn joints_mut(&mut self) -> JointsMut<'_> {
        JointsMut::iter_root(&mut self.joints[..])
    }

    /// Returns a `Frames` iterator over the frames of the bvh.
    #[inline]
    pub fn frames(&self) -> Frames<'_> {
        Frames {
            chunks: if self.num_channels != 0 {
                Some(self.motion_values.as_slice().chunks_exact(self.num_channels))
            } else {
                None
            },
        }
    }

    /// Returns a mutable iterator over the frames of the bvh.
    #[inline]
    pub fn frames_mut(&mut self) -> FramesMut<'_> {
        FramesMut {
            chunks: if self.num_channels != 0 {
                Some(self.motion_values.as_mut_slice().chunks_exact_mut(self.num_channels))
            } else {
                None
            },
        }
    }

    /// Gets the motion value at `frame` and `channel`.
    ///
    /// # Panics
    ///
    /// This method will panic if `frame` is greater than `self.num_frames()` or
    /// if `channel` is out of bounds.
    #[inline]
    pub fn get_motion<I>(&self, frame: usize, channel: I) -> f32
    where
        I: FrameIndex<SliceIndex = usize>,
    {
        *self.frames().nth(frame).unwrap().index(channel)
    }

    /// Returns the motion value at `frame` and `channel` if they are in bounds,
    /// `None` otherwise.
    #[inline]
    pub fn try_get_motion<I>(&self, frame: usize, index: I) -> Option<f32>
    where
        I: FrameIndex<SliceIndex = usize>,
    {
        self.frames()
            .nth(frame)
            .and_then(|f| f.get(index).copied())
    }

    /// Updates the `motion` value at `frame` and `channel` to `new_motion`, and
    /// returns the old motion value.
    ///
    /// # Panics
    ///
    /// This method will panic if `frame` is greater than `self.num_frames()` or
    /// if `channel` is out of bounds.
    #[inline]
    pub fn set_motion<I>(&mut self, frame: usize, channel: I, new_motion: f32) -> f32
    where
        I: FrameIndex<SliceIndex = usize>,
    {
        self.try_set_motion(frame, channel, new_motion)
            .expect("Could not set motion because frame or channel was out of bounds")
    }

    /// Updates the `motion` value at `frame` and `channel` to `new_motion`.
    ///
    /// # Notes
    ///
    /// Returns toe previous motion value if the operation was successful, and `Err(_)` if
    /// the operation was out of bounds.
    #[inline]
    pub fn try_set_motion<I>(
        &mut self,
        frame: usize,
        channel: I,
        new_motion: f32,
    ) -> Result<f32, SetMotionError>
    where
        I: FrameIndex<SliceIndex = usize>,
    {
        self.frames_mut()
            .nth(frame)
            .ok_or(SetMotionError::BadFrame(frame))
            .and_then(|mut f| f.try_set_motion(channel, new_motion))
    }

    /// Get the number of frames in the `Bvh`.
    #[inline]
    pub const fn num_frames(&self) -> usize {
        self.num_frames
    }

    /// Get the number of channels in the `Bvh`.
    #[inline]
    pub const fn num_channels(&self) -> usize {
        self.num_channels
    }

    /// Get the duration each frame should play for in the `Bvh`.
    #[inline]
    pub const fn frame_time(&self) -> &Duration {
        &self.frame_time
    }

    /// Set the duration each frame should play for in the `Bvh` to `new_frame_time`.
    #[inline]
    pub fn set_frame_time(&mut self, new_frame_time: Duration) {
        self.frame_time = new_frame_time;
    }

    /// Create a new `FrameCursor` for inserting and removing frames.
    #[inline]
    pub fn frame_cursor(&mut self) -> FrameCursor<'_> {
        From::from(self)
    }
}

impl Default for Bvh {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Bvh {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.to_bstring(), f)
    }
}

impl FromStr for Bvh {
    type Err = LoadError;
    #[inline]
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Bvh::from_bytes(string.as_bytes())
    }
}

impl TryFrom<&'_ str> for Bvh {
    type Error = LoadError;
    #[inline]
    fn try_from(string: &'_ str) -> Result<Self, Self::Error> {
        FromStr::from_str(string)
    }
}

impl TryFrom<&'_ BStr> for Bvh {
    type Error = LoadError;
    #[inline]
    fn try_from(string: &'_ BStr) -> Result<Self, Self::Error> {
        Bvh::from_bytes(string.as_bytes())
    }
}

impl TryFrom<&'_ [u8]> for Bvh {
    type Error = LoadError;
    #[inline]
    fn try_from(bytes: &'_ [u8]) -> Result<Self, Self::Error> {
        Bvh::from_bytes(bytes)
    }
}

/// A `Channel` composed of a `ChannelType` and an index into the
/// corresponding motion data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Channel {
    /// The type of the `Channel`.
    channel_type: ChannelType,
    /// The index into the `Frame` which corresponds to this `Channel`.
    motion_index: usize,
}

impl Channel {
    #[inline]
    const fn new(channel_type: ChannelType, motion_index: usize) -> Self {
        Channel {
            channel_type,
            motion_index,
        }
    }

    /// Returns the `ChannelType` to which this `Channel` corresponds.
    #[inline]
    pub const fn channel_type(&self) -> ChannelType {
        self.channel_type
    }

    /// Returns the index of the motion value to which this `Channel` corresponds.
    #[inline]
    pub const fn motion_index(&self) -> usize {
        self.motion_index
    }
}

/// The available degrees of freedom along which a `Joint` may be manipulated.
#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChannelType {
    /// Can be translated along the `x` axis.
    PositionX,
    /// Can be translated along the `y` axis.
    PositionY,
    /// Can be translated along the `z` axis.
    PositionZ,
    /// Can be rotated along the `x` axis.
    RotationX,
    /// Can be rotated along the `y` axis.
    RotationY,
    /// Can be rotated along the `z` axis.
    RotationZ,
}

impl ChannelType {
    /// Attempt to parse a bvh channel byte string into a `ChannelType`.
    /// Returns `Err` if the string cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bvh_anim::ChannelType;
    /// assert_eq!(
    ///     ChannelType::from_bytes("Xrotation").unwrap(),
    ///     ChannelType::RotationX);
    ///
    /// let err = ChannelType::from_bytes("Hello").unwrap_err();
    /// assert_eq!(err.into_inner(), "Hello");
    /// ```
    #[inline]
    pub fn from_bytes<B>(s: &B) -> Result<Self, ParseChannelError>
    where
        B: AsRef<[u8]> + ?Sized,
    {
        let s = s.as_ref();
        match s {
            b"Xrotation" => Ok(ChannelType::RotationX),
            b"Yrotation" => Ok(ChannelType::RotationY),
            b"Zrotation" => Ok(ChannelType::RotationZ),

            b"Xposition" => Ok(ChannelType::PositionX),
            b"Yposition" => Ok(ChannelType::PositionY),
            b"Zposition" => Ok(ChannelType::PositionZ),

            _ => Err(ParseChannelError::from(s)),
        }
    }

    /// Returns `true` if this channel corresponds to a rotational
    /// transform, otherwise `false`.
    ///
    /// # Example
    ///
    /// ```
    /// # use bvh_anim::ChannelType;
    /// let channel_type = ChannelType::RotationX;
    /// assert!(channel_type.is_rotation());
    /// ```
    #[inline]
    pub fn is_rotation(&self) -> bool {
        match *self {
            ChannelType::RotationX | ChannelType::RotationY | ChannelType::RotationZ => true,
            _ => false,
        }
    }

    /// Returns `true` if this channel corresponds to a positional
    /// transform, otherwise `false`.
    ///
    /// # Example
    ///
    /// ```
    /// # use bvh_anim::ChannelType;
    /// let channel_type = ChannelType::PositionZ;
    /// assert!(channel_type.is_position());
    /// ```
    #[inline]
    pub fn is_position(&self) -> bool {
        !self.is_rotation()
    }

    /// Get the `Axis` about which this `Channel` transforms.
    ///
    /// # Example
    ///
    /// ```
    /// # use bvh_anim::{Axis, ChannelType};
    /// let channel_type = ChannelType::PositionX;
    /// assert_eq!(channel_type.axis(), Axis::X);
    /// ```
    #[inline]
    pub fn axis(&self) -> Axis {
        match *self {
            ChannelType::RotationX | ChannelType::PositionX => Axis::X,
            ChannelType::RotationY | ChannelType::PositionY => Axis::Y,
            ChannelType::RotationZ | ChannelType::PositionZ => Axis::Z,
        }
    }

    /// Returns the `Vector3` of the channel axis. See the [`Axis::vector`]
    /// [`Axis::vector`] method for more info.
    ///
    /// [`Axis::vector`]: enum.Axis.html#method.vector
    #[inline]
    pub fn axis_vector<T: One + Zero>(&self) -> Vector3<T> {
        self.axis().vector::<T>()
    }

    /// Returns the string representation of the `ChannelType`.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match *self {
            ChannelType::RotationX => "Xrotation",
            ChannelType::RotationY => "Yrotation",
            ChannelType::RotationZ => "Zrotation",

            ChannelType::PositionX => "Xposition",
            ChannelType::PositionY => "Yposition",
            ChannelType::PositionZ => "Zposition",
        }
    }

    /// Returns the string representation of the `ChannelType`.
    #[inline]
    pub fn as_bstr(&self) -> &'static BStr {
        <&BStr>::from(self.as_str())
    }
}

impl TryFrom<&'_ BStr> for ChannelType {
    type Error = ParseChannelError;
    #[inline]
    fn try_from(string: &BStr) -> Result<Self, Self::Error> {
        ChannelType::from_bytes(string)
    }
}

impl TryFrom<&'_ [u8]> for ChannelType {
    type Error = ParseChannelError;
    #[inline]
    fn try_from(string: &[u8]) -> Result<Self, Self::Error> {
        ChannelType::from_bytes(string)
    }
}

impl TryFrom<&'_ str> for ChannelType {
    type Error = ParseChannelError;
    #[inline]
    fn try_from(string: &str) -> Result<Self, Self::Error> {
        ChannelType::from_str(string)
    }
}

impl FromStr for ChannelType {
    type Err = ParseChannelError;
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ChannelType::from_bytes(s)
    }
}

impl fmt::Display for ChannelType {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An enum which represents an axis along a direction in 3D space.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Axis {
    /// `x` axis.
    X,
    /// `y` axis.
    Y,
    /// `z` axis.
    Z,
}

impl Axis {
    /// Returns the `Vector3` which represents the axis.
    ///
    /// # Examples
    ///
    /// ```
    /// # use bvh_anim::Axis;
    /// assert_eq!(Axis::X.vector(), [1.0, 0.0, 0.0].into());
    /// assert_eq!(Axis::Y.vector(), [0.0, 1.0, 0.0].into());
    /// assert_eq!(Axis::Z.vector(), [0.0, 0.0, 1.0].into());
    /// ```
    #[inline]
    pub fn vector<T: One + Zero>(&self) -> Vector3<T> {
        let (o, z) = (one, zero);
        match *self {
            Axis::X => [o(), z(), z()].into(),
            Axis::Y => [z(), o(), z()].into(),
            Axis::Z => [z(), z(), o()].into(),
        }
    }
}

impl fmt::Display for Axis {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match *self {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Z => "z",
        };
        f.write_str(s)
    }
}
