use glutin_osmesa_sys::*;
use std::ptr;

use crate::config::{GetGlConfig, GlConfig};
use crate::context::{
    AsRawContext, ContextApi, ContextAttributes, GlProfile, NotCurrentGlContext,
    NotCurrentGlContextSurfaceAccessor, PossiblyCurrentContextGlSurfaceAccessor,
    PossiblyCurrentGlContext, RawContext,
};
use crate::display::GetGlDisplay;
use crate::error::{ErrorKind, Result};
use crate::private::Sealed;
use crate::surface::SurfaceTypeTrait;

use super::config::Config;
use super::display::Display;
use super::surface::Surface;

#[derive(Debug)]
pub struct NotCurrentContext {
    config: Config,
    inner: OSMesaContext,
}

impl NotCurrentContext {
    pub(crate) fn new(config: Config, attributes: &ContextAttributes) -> Self {
        #[cfg(not(target_os = "redox"))]
        OsMesa::try_loading().expect("failed to load osmesa");

        let mut attribs = Vec::new();

        if let Some(profile) = attributes.profile {
            attribs.push(OSMESA_PROFILE);

            match profile {
                GlProfile::Compatibility => {
                    attribs.push(OSMESA_COMPAT_PROFILE);
                },
                GlProfile::Core => {
                    attribs.push(OSMESA_CORE_PROFILE);
                },
            }
        }

        match attributes.api {
            None => (),
            Some(ContextApi::OpenGl(None)) => (),
            Some(ContextApi::OpenGl(Some(version))) => {
                attribs.push(OSMESA_CONTEXT_MAJOR_VERSION);
                attribs.push(version.major as _);
                attribs.push(OSMESA_CONTEXT_MINOR_VERSION);
                attribs.push(version.minor as _);
            },
            Some(ContextApi::Gles(_)) => {
                panic!("GLES not supported by OSMesa");
            },
        }

        attribs.push(OSMESA_FORMAT);
        attribs.push(OSMESA_BGRA as _);

        attribs.push(OSMESA_DEPTH_BITS);
        attribs.push(config.depth_size() as _);

        attribs.push(OSMESA_STENCIL_BITS);
        attribs.push(config.stencil_size() as _);

        attribs.push(OSMESA_ACCUM_BITS);
        attribs.push(0);

        // attribs array must be NULL terminated.
        attribs.push(0);

        let inner = unsafe { OSMesaCreateContextAttribs(attribs.as_ptr(), ptr::null_mut()) };
        if inner.is_null() {
            panic!("OSMesaCreateContextAttribs failed");
        }

        Self { config, inner }
    }
}

impl NotCurrentGlContext for NotCurrentContext {
    type PossiblyCurrentContext = PossiblyCurrentContext;

    fn treat_as_possibly_current(self) -> Self::PossiblyCurrentContext {
        PossiblyCurrentContext { config: self.config, inner: self.inner }
    }
}

impl<T: SurfaceTypeTrait> NotCurrentGlContextSurfaceAccessor<T> for NotCurrentContext {
    type PossiblyCurrentContext = PossiblyCurrentContext;
    type Surface = Surface<T>;

    fn make_current(self, surface: &Self::Surface) -> Result<Self::PossiblyCurrentContext> {
        let context = PossiblyCurrentContext { config: self.config, inner: self.inner };
        context.make_current(surface)?;
        Ok(context)
    }

    fn make_current_draw_read(
        self,
        surface_draw: &Self::Surface,
        surface_read: &Self::Surface,
    ) -> Result<Self::PossiblyCurrentContext> {
        let context = PossiblyCurrentContext { config: self.config, inner: self.inner };
        context.make_current_draw_read(surface_draw, surface_read)?;
        Ok(context)
    }
}

impl AsRawContext for NotCurrentContext {
    fn raw_context(&self) -> RawContext {
        todo!()
    }
}

impl GetGlConfig for NotCurrentContext {
    type Target = Config;

    fn config(&self) -> Self::Target {
        self.config.clone()
    }
}

impl GetGlDisplay for NotCurrentContext {
    type Target = Display;

    fn display(&self) -> Self::Target {
        self.config.display()
    }
}

impl Sealed for NotCurrentContext {}

#[derive(Debug)]
pub struct PossiblyCurrentContext {
    pub(crate) config: Config,
    pub(crate) inner: OSMesaContext,
}

impl PossiblyCurrentGlContext for PossiblyCurrentContext {
    type NotCurrentContext = NotCurrentContext;

    fn is_current(&self) -> bool {
        unsafe { OSMesaGetCurrentContext() == self.inner }
    }

    fn make_not_current(self) -> Result<Self::NotCurrentContext> {
        Err(ErrorKind::NotSupported("make_not_current not supported by OSMesa").into())
    }
}

impl<T: SurfaceTypeTrait> PossiblyCurrentContextGlSurfaceAccessor<T> for PossiblyCurrentContext {
    type Surface = Surface<T>;

    fn make_current(&self, surface: &Self::Surface) -> Result<()> {
        surface.inner.lock().unwrap().make_current(self.inner);
        Ok(())
    }

    fn make_current_draw_read(
        &self,
        _surface_draw: &Self::Surface,
        _surface_read: &Self::Surface,
    ) -> Result<()> {
        Err(ErrorKind::NotSupported("make_current_draw_read not supported by OSMesa").into())
    }
}

impl AsRawContext for PossiblyCurrentContext {
    fn raw_context(&self) -> RawContext {
        todo!()
    }
}

impl GetGlConfig for PossiblyCurrentContext {
    type Target = Config;

    fn config(&self) -> Self::Target {
        self.config.clone()
    }
}

impl GetGlDisplay for PossiblyCurrentContext {
    type Target = Display;

    fn display(&self) -> Self::Target {
        self.config.display()
    }
}

impl Sealed for PossiblyCurrentContext {}
