// See LICENSE file for copyright and license details.

use glutin::{Event};
use zgl::{Time};
use context::{Context};

pub enum ScreenCommand {
    PopScreen,
    PushScreen(Box<Screen>),
}

pub trait Screen {
    fn tick(&mut self, context: &mut Context, dtime: &Time);
    fn handle_event(&mut self, context: &mut Context, event: &Event);
}

// vim: set tabstop=4 shiftwidth=4 softtabstop=4 expandtab:
