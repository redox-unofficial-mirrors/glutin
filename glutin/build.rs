use cfg_aliases::cfg_aliases;

fn main() {
    // Setup alias to reduce `cfg` boilerplate.
    cfg_aliases! {
        // Systems.
        android: { target_os = "android" },
        wasm: { target_arch = "wasm32" },
        macos: { target_os = "macos" },
        ios: { target_os = "ios" },
        apple: { any(target_os = "ios", target_os = "macos") },
        free_unix: { all(unix, not(apple), not(android)) },
        redox: { target_os = "redox" },

        // Native displays.
        x11_platform: { all(feature = "x11", free_unix, not(wasm), not(redox)) },
        wayland_platform: { all(feature = "wayland", free_unix, not(wasm), not(redox)) },

        // Backends.
        egl_backend: { all(feature = "egl", any(windows, unix), not(apple), not(wasm), not(redox)) },
        glx_backend: { all(feature = "glx", x11_platform, not(wasm)) },
        wgl_backend: { all(feature = "wgl", windows, not(wasm)) },
        cgl_backend: { all(macos, not(wasm)) },
        osmesa_backend: { redox },
    }
}
