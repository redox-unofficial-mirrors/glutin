use glutin_osmesa_sys::*;
use raw_window_handle::RawDisplayHandle;
use std::ffi::{self, CStr};
use std::ptr;

use crate::config::ConfigTemplate;
use crate::context::ContextAttributes;
use crate::display::{AsRawDisplay, DisplayFeatures, GlDisplay, RawDisplay};
use crate::error::Result;
use crate::private::Sealed;
use crate::surface::{PbufferSurface, PixmapSurface, SurfaceAttributes, WindowSurface};

use super::config::Config;
use super::context::NotCurrentContext;
use super::surface::Surface;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Display {
    pub(crate) raw_display_handle: RawDisplayHandle,
}

impl Display {
    pub(crate) fn new(raw_display_handle: RawDisplayHandle) -> Result<Self> {
        Ok(Self { raw_display_handle })
    }
}

impl GlDisplay for Display {
    type Config = Config;
    type NotCurrentContext = NotCurrentContext;
    type PbufferSurface = Surface<PbufferSurface>;
    type PixmapSurface = Surface<PixmapSurface>;
    type WindowSurface = Surface<WindowSurface>;

    unsafe fn find_configs(
        &self,
        template: ConfigTemplate,
    ) -> Result<Box<dyn Iterator<Item = Self::Config> + '_>> {
        // TODO: support config options!
        Ok(Box::new(std::iter::once(Config { display: self.clone() })))
    }

    unsafe fn create_context(
        &self,
        config: &Self::Config,
        context_attributes: &ContextAttributes,
    ) -> Result<Self::NotCurrentContext> {
        Ok(NotCurrentContext::new(config.clone(), context_attributes))
    }

    unsafe fn create_window_surface(
        &self,
        config: &Self::Config,
        surface_attributes: &SurfaceAttributes<WindowSurface>,
    ) -> Result<Self::WindowSurface> {
        Ok(Surface::new(config.clone(), surface_attributes))
    }

    unsafe fn create_pbuffer_surface(
        &self,
        config: &Self::Config,
        surface_attributes: &SurfaceAttributes<PbufferSurface>,
    ) -> Result<Self::PbufferSurface> {
        Ok(Surface::new(config.clone(), surface_attributes))
    }

    unsafe fn create_pixmap_surface(
        &self,
        config: &Self::Config,
        surface_attributes: &SurfaceAttributes<PixmapSurface>,
    ) -> Result<Self::PixmapSurface> {
        Ok(Surface::new(config.clone(), surface_attributes))
    }

    fn get_proc_address(&self, addr: &CStr) -> *const ffi::c_void {
        match unsafe { OSMesaGetProcAddress(addr.as_ptr()) } {
            Some(func) => func as *const ffi::c_void,
            None => ptr::null(),
        }
    }

    fn version_string(&self) -> String {
        todo!()
    }

    fn supported_features(&self) -> DisplayFeatures {
        todo!()
    }
}

impl AsRawDisplay for Display {
    fn raw_display(&self) -> RawDisplay {
        todo!()
    }
}

impl Sealed for Display {}
