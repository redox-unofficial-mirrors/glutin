use std::{mem, ptr};
use std::sync::{Arc, RwLock};
use orbclient::Renderer;
use winit::dpi;
use winit::event_loop::EventLoopWindowTarget;
use winit::platform::redox::WindowExtRedox;
use winit::window::WindowBuilder;

use crate::{
    ContextError, CreationError, GlAttributes, PixelFormat, PixelFormatRequirements, Rect,
    api::osmesa,
};

pub struct Context{
    osmesa: osmesa::OsMesaContext,
    flush: extern "C" fn(),
    window: Arc<RwLock<orbclient::Window>>,
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

        let physical_size = window.inner_size();
        let gl_attr = gl_attr.clone().map_sharing(|ctxt| &ctxt.osmesa);
        let osmesa = osmesa::OsMesaContext::new(pf_reqs, &gl_attr, physical_size)?;

        let flush = osmesa.get_proc_address("glFlush");

        let orbclient_window = window.orbclient_window();

        Ok((window, Context {
            osmesa: osmesa,
            flush: unsafe { mem::transmute::<_, extern "C" fn()>(flush) },
            window: orbclient_window,
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
    pub fn resize(&self, _width: u32, _height: u32) {
        unreachable!();
    }

    #[inline]
    pub unsafe fn make_current(&self) -> Result<(), ContextError> {
        self.osmesa.make_current()?;
        osmesa_sys::OSMesaPixelStore(osmesa_sys::OSMESA_Y_UP, 0);
        Ok(())
    }

    #[inline]
    pub unsafe fn make_not_current(&self) -> Result<(), ContextError> {
        self.osmesa.make_not_current()?;
        osmesa_sys::OSMesaPixelStore(osmesa_sys::OSMESA_Y_UP, 0);
        Ok(())
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

        let mut win = self.window.write().unwrap();
        {
            let win_fb = win.data_mut();
            let osmesa_fb = &self.osmesa.buffer;
            unsafe {
                ptr::copy(osmesa_fb.as_ptr(), win_fb.as_mut_ptr() as *mut u32, win_fb.len());
            }
        }
        win.sync();

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
