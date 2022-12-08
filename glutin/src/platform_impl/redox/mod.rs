use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle};
use std::mem;
use std::sync::Mutex;
use winit::dpi;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::WindowBuilder;

use crate::{
    ContextError, CreationError, GlAttributes, PixelFormat, PixelFormatRequirements, Rect,
    api::osmesa,
};

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

pub struct Context{
    softbuffer: Mutex<softbuffer::GraphicsContext<RawWindow>>,
    osmesa: osmesa::OsMesaContext,
    flush: extern "C" fn(),
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

impl core::fmt::Debug for Context {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Context")
            //TODO
            .finish()
    }
}

impl Context {
    #[inline]
    pub fn new_windowed<T>(
        wb: WindowBuilder,
        el: &EventLoopWindowTarget<T>,
        pf_reqs: &PixelFormatRequirements,
        gl_attr: &GlAttributes<&Context>,
    ) -> Result<(winit::window::Window, Self), CreationError> {
        let window = wb.build(el)?;

        let raw_window = RawWindow {
            display_handle: window.raw_display_handle(),
            window_handle: window.raw_window_handle(),
        };
        let softbuffer = match unsafe { softbuffer::GraphicsContext::new(raw_window) } {
            Ok(ok) => Mutex::new(ok),
            Err(err) => return Err(CreationError::NotSupported(err.to_string())),
        };

        let physical_size = window.inner_size();
        let gl_attr = gl_attr.clone().map_sharing(|ctxt| &ctxt.osmesa);
        let osmesa = osmesa::OsMesaContext::new(pf_reqs, &gl_attr, physical_size)?;

        let flush = osmesa.get_proc_address("glFlush");

        Ok((window, Context {
            softbuffer,
            osmesa,
            flush: unsafe { mem::transmute::<_, extern "C" fn()>(flush) },
        }))
    }

    #[inline]
    pub fn new_headless<T>(
        _el: &EventLoopWindowTarget<T>,
        _pf_reqs: &PixelFormatRequirements,
        _gl_attr: &GlAttributes<&Context>,
        _size: dpi::PhysicalSize<u32>,
    ) -> Result<Self, CreationError> {
        Err(CreationError::NotSupported("Context::new_headless is not supported on Redox".to_string()))
    }

    #[inline]
    pub fn resize(&self, width: u32, height: u32) {
        let is_current = self.is_current();
        if is_current {
            unsafe { self.make_not_current().unwrap() }
        }

        //TODO: is there a safer way to do this?
        unsafe {
            let ptr = &self.osmesa as *const osmesa::OsMesaContext as *mut osmesa::OsMesaContext;
            (*ptr).resize(width, height);
        }

        if is_current {
            unsafe { self.make_current().unwrap() }
        }
    }

    #[inline]
    pub unsafe fn make_current(&self) -> Result<(), ContextError> {
        self.osmesa.make_current()?;
        osmesa_sys::OSMesaPixelStore(osmesa_sys::OSMESA_Y_UP, 0);
        Ok(())
    }

    #[inline]
    pub unsafe fn make_not_current(&self) -> Result<(), ContextError> {
        self.osmesa.make_not_current()
    }

    #[inline]
    pub fn is_current(&self) -> bool {
        self.osmesa.is_current()
    }

    #[inline]
    pub fn get_proc_address(&self, addr: &str) -> *const core::ffi::c_void  {
        self.osmesa.get_proc_address(addr)
    }

    #[inline]
    pub fn buffer_age(&self) -> u32 {
        0
    }

    #[inline]
    pub fn swap_buffers(&self) -> Result<(), ContextError> {
        (self.flush)();

        self.softbuffer.lock().unwrap().set_buffer(
            &self.osmesa.buffer,
            self.osmesa.width as u16,
            self.osmesa.height as u16
        );

        Ok(())
    }

    #[inline]
    pub fn swap_buffers_with_damage(&self, _rects: &[Rect]) -> Result<(), ContextError> {
        Err(ContextError::OsError("buffer damage not suported".to_string()))
    }

    #[inline]
    pub fn swap_buffers_with_damage_supported(&self) -> bool {
        false
    }

    #[inline]
    pub fn get_api(&self) -> crate::Api {
        self.osmesa.get_api()
    }

    #[inline]
    pub fn get_pixel_format(&self) -> PixelFormat {
        PixelFormat {
            hardware_accelerated: false,
            color_bits: 24,
            alpha_bits: 8,
            depth_bits: 24, //TODO
            stencil_bits: 8, //TODO
            stereoscopy: false,
            double_buffer: true,
            multisampling: None,
            srgb: false,
        }
    }
}
