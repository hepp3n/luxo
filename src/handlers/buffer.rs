use smithay::wayland::buffer::BufferHandler;

use crate::state::Luxo;

impl BufferHandler for Luxo {
    fn buffer_destroyed(
        &mut self,
        _buffer: &smithay::reexports::wayland_server::protocol::wl_buffer::WlBuffer,
    ) {
    }
}

