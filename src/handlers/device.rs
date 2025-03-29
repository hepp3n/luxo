use std::os::fd::OwnedFd;

use smithay::{
    delegate_data_device, delegate_primary_selection,
    input::Seat,
    wayland::selection::{
        data_device::{ClientDndGrabHandler, DataDeviceHandler, ServerDndGrabHandler},
        primary_selection::PrimarySelectionHandler,
        SelectionHandler, SelectionSource, SelectionTarget,
    },
};

use crate::state::Luxo;

impl ClientDndGrabHandler for Luxo {
    fn started(
        &mut self,
        _source: Option<smithay::reexports::wayland_server::protocol::wl_data_source::WlDataSource>,
        _icon: Option<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
        _seat: Seat<Self>,
    ) {
    }

    fn dropped(
        &mut self,
        _target: Option<smithay::reexports::wayland_server::protocol::wl_surface::WlSurface>,
        _validated: bool,
        _seat: Seat<Self>,
    ) {
    }
}

impl ServerDndGrabHandler for Luxo {
    fn accept(&mut self, _mime_type: Option<String>, _seat: Seat<Self>) {}

    fn action(
        &mut self,
        _action: smithay::reexports::wayland_server::protocol::wl_data_device_manager::DndAction,
        _seat: Seat<Self>,
    ) {
    }

    fn dropped(&mut self, _seat: Seat<Self>) {}

    fn cancelled(&mut self, _seat: Seat<Self>) {}

    fn send(&mut self, _mime_type: String, _fd: OwnedFd, _seat: Seat<Self>) {}

    fn finished(&mut self, _seat: Seat<Self>) {}
}

impl DataDeviceHandler for Luxo {
    fn data_device_state(&self) -> &smithay::wayland::selection::data_device::DataDeviceState {
        &self.data_device_state
    }
}

delegate_data_device!(Luxo);

impl PrimarySelectionHandler for Luxo {
    fn primary_selection_state(
        &self,
    ) -> &smithay::wayland::selection::primary_selection::PrimarySelectionState {
        &self.primary_selection_state
    }
}

impl SelectionHandler for Luxo {
    type SelectionUserData = ();

    fn new_selection(
        &mut self,
        ty: SelectionTarget,
        source: Option<SelectionSource>,
        _seat: Seat<Self>,
    ) {
        if let Some(xwm) = self.xwm.as_mut() {
            if let Err(err) = xwm.new_selection(ty, source.map(|source| source.mime_types())) {
                tracing::warn!(?err, ?ty, "Failed to set Xwayland selection");
            }
        }
    }

    fn send_selection(
        &mut self,
        ty: SelectionTarget,
        mime_type: String,
        fd: OwnedFd,
        _seat: Seat<Self>,
        _user_data: &(),
    ) {
        if let Some(xwm) = self.xwm.as_mut() {
            if let Err(err) = xwm.send_selection(ty, mime_type, fd, self.handle.clone()) {
                tracing::warn!(?err, "Failed to send primary (X11 -> Wayland)");
            }
        }
    }
}

delegate_primary_selection!(Luxo);
