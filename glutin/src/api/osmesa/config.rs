use super::display::Display;
use crate::config::{Api, AsRawConfig, ColorBufferType, ConfigSurfaceTypes, GlConfig, RawConfig};
use crate::display::GetGlDisplay;
use crate::private::Sealed;
#[cfg(x11_platform)]
use crate::platform::x11::{X11GlConfigExt, X11VisualInfo};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub(crate) display: Display,
}

impl GlConfig for Config {
    fn color_buffer_type(&self) -> Option<ColorBufferType> {
        Some(ColorBufferType::Rgb { r_size: 8, g_size: 8, b_size: 8 })
    }

    fn float_pixels(&self) -> bool {
        false
    }

    fn alpha_size(&self) -> u8 {
        8
    }

    fn depth_size(&self) -> u8 {
        24
    }

    fn stencil_size(&self) -> u8 {
        8
    }

    fn num_samples(&self) -> u8 {
        0
    }

    fn srgb_capable(&self) -> bool {
        false
    }

    fn supports_transparency(&self) -> Option<bool> {
        None
    }

    fn config_surface_types(&self) -> ConfigSurfaceTypes {
        ConfigSurfaceTypes::WINDOW | ConfigSurfaceTypes::PBUFFER | ConfigSurfaceTypes::PIXMAP
    }

    fn api(&self) -> Api {
        Api::OPENGL
    }
}

impl GetGlDisplay for Config {
    type Target = Display;

    fn display(&self) -> Self::Target {
        self.display.clone()
    }
}

#[cfg(x11_platform)]
impl X11GlConfigExt for Config {
    fn x11_visual(&self) -> Option<X11VisualInfo> {
        None
    }
}

impl AsRawConfig for Config {
    fn raw_config(&self) -> RawConfig {
        todo!()
    }
}

impl Sealed for Config {}
