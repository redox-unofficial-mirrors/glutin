use glutin_osmesa_sys::*;
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};
use std::ffi::CStr;
use std::marker::PhantomData;
use std::num::NonZeroU32;
use std::sync::Mutex;
use std::{fmt, mem, ptr};

use crate::context::PossiblyCurrentGlContext;
use crate::display::{GetGlDisplay, GlDisplay};
use crate::error::{ErrorKind, Result};
use crate::private::Sealed;
use crate::surface::{
    AsRawSurface, GlSurface, RawSurface, ResizeableSurface, SurfaceAttributes, SurfaceTypeTrait,
    SwapInterval,
};

use super::config::Config;
use super::context::PossiblyCurrentContext;
use super::display::Display;

// Wrapper to get around lifetimes in GraphicsContext
#[derive(Debug)]
struct RawWindow {
    display_handle: RawDisplayHandle,
    window_handle: RawWindowHandle,
}

unsafe impl HasRawDisplayHandle for RawWindow {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        self.display_handle
    }
}

unsafe impl HasRawWindowHandle for RawWindow {
    fn raw_window_handle(&self) -> RawWindowHandle {
        self.window_handle
    }
}

pub(crate) struct SurfaceInner {
    width: u32,
    height: u32,
    buffer: Vec<u32>,
    flush: extern "C" fn(),
    softbuffer_opt: Option<softbuffer::GraphicsContext<RawWindow>>,
}

impl SurfaceInner {
    pub(crate) fn make_current(&mut self, ctx: OSMesaContext) {
        eprintln!(
            "make_current {:p}: {:p}, {}, {}",
            ctx,
            self.buffer.as_mut_ptr(),
            self.width,
            self.height
        );
        if unsafe {
            OSMesaMakeCurrent(
                ctx,
                self.buffer.as_mut_ptr() as *mut _,
                0x1401, // TODO: what is this type?
                self.width.try_into().unwrap(),
                self.height.try_into().unwrap(),
            )
        } == 0
        {
            // TODO: return error
            panic!("OSMesaMakeCurrent failed");
        }
        unsafe {
            OSMesaPixelStore(OSMESA_Y_UP, 0);
        }
    }

    fn swap_buffers(&mut self) {
        (self.flush)();

        match self.softbuffer_opt {
            Some(ref mut softbuffer) => {
                softbuffer.set_buffer(
                    &self.buffer,
                    self.width.try_into().unwrap(),
                    self.height.try_into().unwrap(),
                );
            },
            None => (),
        }
    }

    fn resize(&mut self, ctx: OSMesaContext, width: u32, height: u32) {
        let current = unsafe { OSMesaGetCurrentContext() == ctx };

        self.width = width;
        self.height = height;
        self.buffer = vec![0u32; width as usize * height as usize];

        if current {
            self.make_current(ctx);
        };
    }
}

pub struct Surface<T: SurfaceTypeTrait> {
    config: Config,
    pub(crate) inner: Mutex<SurfaceInner>,
    _ty: PhantomData<T>,
}

impl<T: SurfaceTypeTrait> fmt::Debug for Surface<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Surface").field("config", &self.config).finish()
    }
}

impl<T: SurfaceTypeTrait> Surface<T> {
    pub(crate) fn new(config: Config, attributes: &SurfaceAttributes<T>) -> Self {
        let width = attributes.width.map_or(0, |x| x.get());
        let height = attributes.height.map_or(0, |x| x.get());
        let buffer = vec![0u32; width as usize * height as usize];
        let flush = unsafe {
            mem::transmute(
                config.display.get_proc_address(CStr::from_bytes_with_nul(b"glFlush\0").unwrap()),
            )
        };
        let softbuffer_opt = match attributes.raw_window_handle {
            Some(raw_window_handle) => Some(unsafe {
                softbuffer::GraphicsContext::new(RawWindow {
                    display_handle: config.display.raw_display_handle,
                    window_handle: raw_window_handle,
                })
                .unwrap()
            }),
            None => None,
        };
        let inner = Mutex::new(SurfaceInner { width, height, buffer, flush, softbuffer_opt });
        Self { config, inner, _ty: PhantomData }
    }
}

impl<T: SurfaceTypeTrait> GlSurface<T> for Surface<T> {
    type Context = PossiblyCurrentContext;
    type SurfaceType = T;

    fn buffer_age(&self) -> u32 {
        0
    }

    fn width(&self) -> Option<u32> {
        let inner = self.inner.lock().unwrap();
        Some(inner.width)
    }

    fn height(&self) -> Option<u32> {
        let inner = self.inner.lock().unwrap();
        Some(inner.height)
    }

    fn is_single_buffered(&self) -> bool {
        true
    }

    fn swap_buffers(&self, _context: &Self::Context) -> Result<()> {
        self.inner.lock().unwrap().swap_buffers();
        Ok(())
    }

    fn is_current(&self, context: &Self::Context) -> bool {
        context.is_current()
    }

    fn is_current_draw(&self, context: &Self::Context) -> bool {
        self.is_current(context)
    }

    fn is_current_read(&self, context: &Self::Context) -> bool {
        self.is_current(context)
    }

    fn set_swap_interval(&self, _context: &Self::Context, _interval: SwapInterval) -> Result<()> {
        Err(ErrorKind::NotSupported("set_swap_interval not supported with OSMesa").into())
    }

    fn resize(&self, context: &Self::Context, width: NonZeroU32, height: NonZeroU32)
    where
        Self::SurfaceType: ResizeableSurface,
    {
        self.inner.lock().unwrap().resize(context.inner, width.get(), height.get());
    }
}

impl<T: SurfaceTypeTrait> GetGlDisplay for Surface<T> {
    type Target = Display;

    fn display(&self) -> Self::Target {
        self.config.display()
    }
}

impl<T: SurfaceTypeTrait> AsRawSurface for Surface<T> {
    fn raw_surface(&self) -> RawSurface {
        todo!()
    }
}

impl<T: SurfaceTypeTrait> Sealed for Surface<T> {}
