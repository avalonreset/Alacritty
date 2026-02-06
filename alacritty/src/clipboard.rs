use log::{debug, warn};
use winit::raw_window_handle::RawDisplayHandle;

use alacritty_terminal::term::ClipboardType;

#[cfg(any(feature = "x11", target_os = "macos", windows))]
use copypasta::ClipboardContext;
use copypasta::ClipboardProvider;
use copypasta::nop_clipboard::NopClipboardContext;
#[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
use copypasta::wayland_clipboard;
#[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
use copypasta::x11_clipboard::{Primary as X11SelectionClipboard, X11ClipboardContext};

pub struct Clipboard {
    clipboard: Box<dyn ClipboardProvider>,
    selection: Option<Box<dyn ClipboardProvider>>,
}

impl Clipboard {
    pub unsafe fn new(display: RawDisplayHandle) -> Self {
        match display {
            #[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
            RawDisplayHandle::Wayland(display) => {
                let (selection, clipboard) = unsafe {
                    wayland_clipboard::create_clipboards_from_external(display.display.as_ptr())
                };
                Self { clipboard: Box::new(clipboard), selection: Some(Box::new(selection)) }
            },
            _ => Self::default(),
        }
    }

    /// Used for tests, to handle missing clipboard provider when built without the `x11`
    /// feature, and as default clipboard value.
    pub fn new_nop() -> Self {
        Self { clipboard: Box::new(NopClipboardContext::new().unwrap()), selection: None }
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        #[cfg(any(target_os = "macos", windows))]
        return Self { clipboard: Box::new(ClipboardContext::new().unwrap()), selection: None };

        #[cfg(all(feature = "x11", not(any(target_os = "macos", windows))))]
        return Self {
            clipboard: Box::new(ClipboardContext::new().unwrap()),
            selection: Some(Box::new(X11ClipboardContext::<X11SelectionClipboard>::new().unwrap())),
        };

        #[cfg(not(any(feature = "x11", target_os = "macos", windows)))]
        return Self::new_nop();
    }
}

impl Clipboard {
    pub fn store(&mut self, ty: ClipboardType, text: impl Into<String>) {
        let clipboard = match (ty, &mut self.selection) {
            (ClipboardType::Selection, Some(provider)) => provider,
            (ClipboardType::Selection, None) => return,
            _ => &mut self.clipboard,
        };

        clipboard.set_contents(text.into()).unwrap_or_else(|err| {
            warn!("Unable to store text in clipboard: {err}");
        });
    }

    pub fn load(&mut self, ty: ClipboardType) -> String {
        let clipboard = match (ty, &mut self.selection) {
            (ClipboardType::Selection, Some(provider)) => provider,
            _ => &mut self.clipboard,
        };

        match clipboard.get_contents() {
            Err(err) => {
                debug!("Unable to load text from clipboard: {err}");

                // On Windows, screenshots are commonly placed into the clipboard as an image
                // without any textual representation. As a small quality-of-life improvement,
                // attempt to persist a PNG clipboard image to disk and paste its path instead.
                #[cfg(windows)]
                if ty == ClipboardType::Clipboard {
                    if let Some(url) = windows_image::save_image_to_temp_url() {
                        return url;
                    }
                }

                String::new()
            },
            Ok(text) => {
                if !text.is_empty() {
                    return text;
                }

                #[cfg(windows)]
                if ty == ClipboardType::Clipboard {
                    if let Some(url) = windows_image::save_image_to_temp_url() {
                        return url;
                    }
                }

                text
            },
        }
    }
}

#[cfg(windows)]
mod windows_image {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use windows_sys::Win32::Foundation::HGLOBAL;
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, GetClipboardData, IsClipboardFormatAvailable, OpenClipboard,
        RegisterClipboardFormatW,
    };
    use windows_sys::Win32::System::Memory::{GlobalLock, GlobalSize, GlobalUnlock};

    /// Persist a clipboard image to a temporary file.
    ///
    /// This supports the `"PNG"` clipboard format when present, otherwise it falls back to the
    /// standard CF_DIBV5/CF_DIB formats (saved as `.bmp`).
    pub fn save_image_to_temp_url() -> Option<String> {
        let (ext, bytes) = load_png_bytes()
            .map(|b| ("png", b))
            .or_else(|| load_dib_as_bmp_bytes().map(|b| ("bmp", b)))?;

        let mut dir = std::env::temp_dir();
        dir.push("alacritty");
        dir.push("paste");
        std::fs::create_dir_all(&dir).ok()?;

        let ts = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_millis();
        let path = dir.join(format!("clipboard-{ts}.{ext}"));
        std::fs::write(&path, bytes).ok()?;

        Some(path_to_file_url(&path))
    }

    fn path_to_file_url(path: &Path) -> String {
        // Convert `C:\foo\bar.png` -> `file:///C:/foo/bar.png` so Alacritty's URL hints recognize it.
        let mut s = path.to_string_lossy().replace('\\', "/");

        // Best-effort encoding for common path characters.
        // This is intentionally minimal; the primary goal is clickable "file:" links for typical paths.
        if s.contains(' ') {
            s = s.replace(' ', "%20");
        }

        if s.starts_with("//") {
            format!("file:{s}")
        } else {
            format!("file:///{s}")
        }
    }

    fn load_png_bytes() -> Option<Vec<u8>> {
        // The clipboard uses a named format for PNG data.
        let png_format = register_format("PNG")?;
        if unsafe { IsClipboardFormatAvailable(png_format) } == 0 {
            return None;
        }

        let _guard = ClipboardGuard::open()?;

        // SAFETY: We hold the clipboard open for the duration of this function.
        let handle = unsafe { GetClipboardData(png_format) } as HGLOBAL;
        if handle.is_null() {
            return None;
        }

        // SAFETY: The returned handle is a global memory object with size reported by GlobalSize.
        let ptr = unsafe { GlobalLock(handle) } as *const u8;
        if ptr.is_null() {
            return None;
        }
        let _lock_guard = GlobalLockGuard(handle);

        let size = unsafe { GlobalSize(handle) };
        let size = usize::try_from(size).ok()?;

        // SAFETY: The buffer is valid for `size` bytes while locked.
        let bytes = unsafe { std::slice::from_raw_parts(ptr, size) }.to_vec();

        Some(bytes)
    }

    fn load_dib_as_bmp_bytes() -> Option<Vec<u8>> {
        // Standard clipboard bitmap formats.
        const CF_DIB: u32 = 8;
        const CF_DIBV5: u32 = 17;

        let format = if unsafe { IsClipboardFormatAvailable(CF_DIBV5) } != 0 {
            CF_DIBV5
        } else if unsafe { IsClipboardFormatAvailable(CF_DIB) } != 0 {
            CF_DIB
        } else {
            return None;
        };

        let _guard = ClipboardGuard::open()?;

        // SAFETY: We hold the clipboard open for the duration of this function.
        let handle = unsafe { GetClipboardData(format) } as HGLOBAL;
        if handle.is_null() {
            return None;
        }

        // SAFETY: The returned handle is a global memory object with size reported by GlobalSize.
        let ptr = unsafe { GlobalLock(handle) } as *const u8;
        if ptr.is_null() {
            return None;
        }
        let _lock_guard = GlobalLockGuard(handle);

        let size = unsafe { GlobalSize(handle) };
        let size = usize::try_from(size).ok()?;
        if size < 4 {
            return None;
        }

        // SAFETY: The buffer is valid for `size` bytes while locked.
        let dib = unsafe { std::slice::from_raw_parts(ptr, size) };

        // Build a BMP file by prepending a BITMAPFILEHEADER.
        //
        // CF_DIB/CF_DIBV5 do not include BITMAPFILEHEADER, but the rest of the bytes match the BMP
        // layout after that header.
        let dib_pixel_offset = dib_pixel_data_offset(dib)?;
        let bf_off_bits = 14u32.checked_add(u32::try_from(dib_pixel_offset).ok()?)?;
        let bf_size = 14u32.checked_add(u32::try_from(dib.len()).ok()?)?;

        let mut bmp = Vec::with_capacity(14 + dib.len());
        // bfType = "BM"
        bmp.extend_from_slice(&[0x42, 0x4D]);
        // bfSize
        bmp.extend_from_slice(&bf_size.to_le_bytes());
        // bfReserved1, bfReserved2
        bmp.extend_from_slice(&0u16.to_le_bytes());
        bmp.extend_from_slice(&0u16.to_le_bytes());
        // bfOffBits
        bmp.extend_from_slice(&bf_off_bits.to_le_bytes());
        // DIB bytes
        bmp.extend_from_slice(dib);

        Some(bmp)
    }

    fn dib_pixel_data_offset(dib: &[u8]) -> Option<usize> {
        // DIB starts with a header size (DWORD).
        let header_size = u32::from_le_bytes([*dib.get(0)?, *dib.get(1)?, *dib.get(2)?, *dib.get(3)?])
            as usize;
        if header_size < 40 || header_size > dib.len() {
            return None;
        }

        // For BITMAPINFOHEADER (40 bytes), BI_BITFIELDS/BI_ALPHABITFIELDS add masks after header.
        // For BITMAPV4/V5, masks are included in the larger header and don't add extra bytes here.
        let mut mask_bytes = 0usize;
        let palette_entries: usize;

        if header_size == 40 {
            // biBitCount (WORD) at offset 14, biCompression (DWORD) at offset 16, biClrUsed (DWORD) at offset 32.
            let bit_count = u16::from_le_bytes([*dib.get(14)?, *dib.get(15)?]) as usize;
            let compression = u32::from_le_bytes([*dib.get(16)?, *dib.get(17)?, *dib.get(18)?, *dib.get(19)?]);
            let clr_used = u32::from_le_bytes([*dib.get(32)?, *dib.get(33)?, *dib.get(34)?, *dib.get(35)?]) as usize;

            // BI_RGB=0, BI_BITFIELDS=3, BI_ALPHABITFIELDS=6
            mask_bytes = match compression {
                3 => 12,
                6 => 16,
                _ => 0,
            };

            palette_entries = if clr_used != 0 {
                clr_used
            } else if bit_count <= 8 {
                1usize.checked_shl(bit_count as u32)?
            } else {
                0
            };
        } else {
            // For BITMAPV4HEADER/BITMAPV5HEADER and others, the palette is still present for <=8bpp,
            // but we do not attempt to special-case bitfield masks beyond BITMAPINFOHEADER.
            palette_entries = 0;
        }

        let palette_bytes = palette_entries.checked_mul(4)?;
        let offset = header_size.checked_add(mask_bytes)?.checked_add(palette_bytes)?;
        if offset > dib.len() {
            return None;
        }

        Some(offset)
    }

    fn register_format(name: &str) -> Option<u32> {
        let wide: Vec<u16> = OsStr::new(name).encode_wide().chain(std::iter::once(0)).collect();

        // SAFETY: `wide` is null-terminated for the call.
        let id = unsafe { RegisterClipboardFormatW(wide.as_ptr()) };
        if id == 0 {
            return None;
        }

        Some(id)
    }

    struct ClipboardGuard;

    impl ClipboardGuard {
        fn open() -> Option<Self> {
            // SAFETY: Passing NULL opens the clipboard for the current task.
            if unsafe { OpenClipboard(std::ptr::null_mut()) } == 0 {
                return None;
            }

            Some(Self)
        }
    }

    impl Drop for ClipboardGuard {
        fn drop(&mut self) {
            // SAFETY: Best-effort close; if it fails, there's nothing actionable to do here.
            unsafe {
                CloseClipboard();
            }
        }
    }

    struct GlobalLockGuard(HGLOBAL);

    impl Drop for GlobalLockGuard {
        fn drop(&mut self) {
            // SAFETY: Best-effort unlock; if it fails, there's nothing actionable to do here.
            unsafe {
                GlobalUnlock(self.0);
            }
        }
    }
}
