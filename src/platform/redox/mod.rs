extern crate orbclient;

use std::{mem, ptr};
use std::sync::{Arc, Mutex};
use winit::os::redox::WindowExt;

use {ContextError, CreationError, GlAttributes, PixelFormat, PixelFormatRequirements};
use api::osmesa;

use self::orbclient::Renderer;

pub struct Context{
    osmesa: osmesa::OsMesaContext,
    flush: extern "C" fn(),
    window: Arc<Mutex<orbclient::Window>>,
}

impl Context {
    #[inline]
    pub unsafe fn new(
        window_builder: winit::WindowBuilder,
        events_loop: &winit::EventsLoop,
        pf_reqs: &PixelFormatRequirements,
        gl_attr: &GlAttributes<&Context>,
    ) -> Result<(winit::Window, Self), CreationError> {
        let window = window_builder.build(events_loop)?;

        let dpi = window.get_hidpi_factor();
        let logical_size = window.get_inner_size().unwrap();
        let physical_size = logical_size.to_physical(dpi);
        let (w, h) = physical_size.into();
        let gl_attr = gl_attr.clone().map_sharing(|ctxt| &ctxt.osmesa);
        let osmesa = osmesa::OsMesaContext::new((w, h), pf_reqs, &gl_attr)?;

        let flush = osmesa.get_proc_address("glFlush");
        println!("flush: {:p}", flush);

        let orbclient_window = window.get_orbclient_window();

        Ok((window, Context {
            osmesa: osmesa,
            flush: mem::transmute::<*const (), extern "C" fn()>(flush),
            window: orbclient_window,
        }))
    }

    #[inline]
    pub unsafe fn new_context(
        _el: &winit::EventsLoop,
        _pf_reqs: &PixelFormatRequirements,
        _gl_attr: &GlAttributes<&Context>,
        _shareable_with_windowed_contexts: bool,
    ) -> Result<Self, CreationError> {
        Err(CreationError::NotSupported("Context::new_context is not supported on Redox"))
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
    pub fn is_current(&self) -> bool {
        self.osmesa.is_current()
    }

    #[inline]
    pub fn get_proc_address(&self, addr: &str) -> *const () {
        self.osmesa.get_proc_address(addr)
    }

    #[inline]
    pub fn swap_buffers(&self) -> Result<(), ContextError> {
        (self.flush)();

        let mut win = self.window.lock().unwrap();
        {
            let win_fb = win.data_mut();
            let osmesa_fb = self.osmesa.get_framebuffer();
            unsafe {
                ptr::copy(osmesa_fb.as_ptr(), win_fb.as_mut_ptr() as *mut u32, win_fb.len());
            }
        }
        win.sync();

        Ok(())
    }

    #[inline]
    pub fn get_api(&self) -> ::Api {
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
