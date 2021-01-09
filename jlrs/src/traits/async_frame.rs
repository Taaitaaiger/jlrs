
use crate::{frame::OutputScope, prelude::DynamicAsyncFrame};

use super::{Frame, Scope};
pub trait AsyncFrame<'frame>: Frame<'frame> {
}

impl<'frame> AsyncFrame<'frame> for DynamicAsyncFrame<'frame> {}

pub trait AsyncScope<'scope, 'frame, 'data, F: AsyncFrame<'frame>>: Scope<'scope, 'frame, 'data, F> {}

impl<'frame, 'data, F: AsyncFrame<'frame>> AsyncScope<'frame, 'frame, 'data, F> for &mut F {

}
impl<'scope, 'frame, 'data, 'borrow, F: AsyncFrame<'frame>> AsyncScope<'scope, 'frame, 'data, F> for OutputScope<'scope, 'frame, 'borrow, F> {}