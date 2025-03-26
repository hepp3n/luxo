use crate::{state::Backend, LuxoState};
use smithay::reexports::{
    wayland_protocols::ext::workspace::v1::server::{
        ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1,
        ext_workspace_handle_v1::ExtWorkspaceHandleV1,
        ext_workspace_manager_v1::ExtWorkspaceManagerV1,
    },
    wayland_server::{
        protocol::wl_surface::WlSurface, Client, Dispatch, DisplayHandle, GlobalDispatch,
    },
};
use std::collections::HashMap;

const VERSION: u32 = 1;

pub struct WorkspaceState {
    display: DisplayHandle,
    workspaces: Vec<Workspace>,
    active_workspace: usize,
    surface_to_workspace: HashMap<WlSurface, usize>,
    workspace_groups: Vec<WorkspaceGroup>,
}

#[derive(Clone)]
pub struct Workspace {
    pub id: usize,
    pub name: String,
    pub surfaces: Vec<WlSurface>,
}

impl Workspace {
    pub fn new(id: usize) -> Workspace {
        Workspace {
            id,
            name: format!("{}", id + 1),
            surfaces: Vec::new(),
        }
    }
}

pub struct WorkspaceGroup {
    pub id: usize,
    pub name: String,
    pub workspaces: Vec<Workspace>,
}

impl Default for WorkspaceGroup {
    fn default() -> Self {
        let id: usize = 0;

        let workspaces: Vec<Workspace> = (0..9).into_iter().map(|n| Workspace::new(n)).collect();

        WorkspaceGroup {
            id,
            name: format!("Group {}", id + 1),
            workspaces,
        }
    }
}

pub struct WorkspaceGlobalData;

pub trait WorkspaceManagerHandler {
    fn switch_workspace(&mut self, id: usize);
}

impl WorkspaceState {
    pub fn new<D>(display: &DisplayHandle) -> Self
    where
        D: GlobalDispatch<ExtWorkspaceManagerV1, WorkspaceGlobalData>,
        D: Dispatch<ExtWorkspaceManagerV1, ()>,
        D: Dispatch<ExtWorkspaceHandleV1, Workspace>,
        D: Dispatch<ExtWorkspaceGroupHandleV1, WorkspaceGroup>,
        D: WorkspaceManagerHandler,
        D: 'static,
    {
        display.create_global::<D, ExtWorkspaceManagerV1, _>(VERSION, WorkspaceGlobalData);

        let ws_group = vec![WorkspaceGroup::default()];

        Self {
            display: display.clone(),
            workspaces: ws_group[0].workspaces.clone(),
            active_workspace: 0,
            surface_to_workspace: HashMap::new(),
            workspace_groups: ws_group,
        }
    }
}

impl<B> GlobalDispatch<ExtWorkspaceManagerV1, WorkspaceGlobalData, LuxoState<B>> for WorkspaceState
where
    B: Backend + 'static,
{
    fn bind(
        state: &mut LuxoState<B>,
        handle: &DisplayHandle,
        client: &Client,
        resource: smithay::reexports::wayland_server::New<ExtWorkspaceManagerV1>,
        global_data: &WorkspaceGlobalData,
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, LuxoState<B>>,
    ) {
    }
}

impl<B> Dispatch<ExtWorkspaceManagerV1, (), LuxoState<B>> for WorkspaceState
where
    B: Backend + 'static,
{
    fn request(
        state: &mut LuxoState<B>,
        client: &Client,
        resource: &ExtWorkspaceManagerV1,
        request: <ExtWorkspaceManagerV1 as smithay::reexports::wayland_server::Resource>::Request,
        data: &(),
        dhandle: &DisplayHandle,
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, LuxoState<B>>,
    ) {
    }
}

impl<B> Dispatch<ExtWorkspaceHandleV1, Workspace, LuxoState<B>> for WorkspaceState
where
    B: Backend + 'static,
{
    fn request(
        state: &mut LuxoState<B>,
        client: &Client,
        resource: &ExtWorkspaceHandleV1,
        request: <ExtWorkspaceHandleV1 as smithay::reexports::wayland_server::Resource>::Request,
        data: &Workspace,
        dhandle: &DisplayHandle,
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, LuxoState<B>>,
    ) {
        todo!()
    }
}

impl<B> Dispatch<ExtWorkspaceGroupHandleV1, WorkspaceGroup, LuxoState<B>> for WorkspaceState
where
    B: Backend + 'static,
{
    fn request(
        state: &mut LuxoState<B>,
        client: &Client,
        resource: &ExtWorkspaceGroupHandleV1,
        request: <ExtWorkspaceGroupHandleV1 as smithay::reexports::wayland_server::Resource>::Request,
        data: &WorkspaceGroup,
        dhandle: &DisplayHandle,
        data_init: &mut smithay::reexports::wayland_server::DataInit<'_, LuxoState<B>>,
    ) {
    }
}

macro_rules! delegate_workspace {
    ($(@<$( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+>)? $ty: ty) => {

        smithay::reexports::wayland_server::delegate_global_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            smithay::reexports::wayland_protocols::ext::workspace::v1::server::ext_workspace_manager_v1::ExtWorkspaceManagerV1: $crate::protocols::ext_workspace_manager_v1::WorkspaceGlobalData
        ] => $crate::protocols::ext_workspace_manager_v1::WorkspaceState);
        smithay::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            smithay::reexports::wayland_protocols::ext::workspace::v1::server::ext_workspace_manager_v1::ExtWorkspaceManagerV1: ()
        ] => $crate::protocols::ext_workspace_manager_v1::WorkspaceState);
        smithay::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            smithay::reexports::wayland_protocols::ext::workspace::v1::server::ext_workspace_group_handle_v1::ExtWorkspaceGroupHandleV1: $crate::protocols::ext_workspace_manager_v1::WorkspaceGroup
        ] => $crate::protocols::ext_workspace_manager_v1::WorkspaceState);
        smithay::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            smithay::reexports::wayland_protocols::ext::workspace::v1::server::ext_workspace_handle_v1::ExtWorkspaceHandleV1: $crate::protocols::ext_workspace_manager_v1::Workspace
        ] => $crate::protocols::ext_workspace_manager_v1::WorkspaceState);

    };
}
pub(crate) use delegate_workspace;
