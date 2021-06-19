//! `NcDirect` methods and associated functions.

use core::ptr::{null, null_mut};

use crate::ffi::sigset_t;
use crate::{
    cstring, error, error_ref_mut, rstring, NcAlign, NcBlitter, NcCapabilities, NcChannels,
    NcComponent, NcDim, NcDirect, NcDirectFlags, NcDirectV, NcEgc, NcError, NcInput, NcOffset,
    NcPaletteIndex, NcResult, NcRgb, NcScale, NcStyle, NcTime, NCRESULT_ERR,
};

/// # `NcDirect` constructors and destructors
impl NcDirect {
    /// New NcDirect with the default options.
    ///
    /// Initializes a direct-mode notcurses context on the tty.
    ///
    /// Direct mode supports a limited subset of notcurses routines,
    /// and neither supports nor requires
    /// [notcurses_render()][crate::notcurses_render]. This can be used to add
    /// color and styling to text in the standard output paradigm.
    ///
    /// *C style function: [ncdirect_init()][crate::ncdirect_init].*
    pub fn new<'a>() -> NcResult<&'a mut NcDirect> {
        Self::with_flags(0)
    }

    /// New NcDirect with optional flags.
    ///
    /// `flags` is a bitmask over:
    /// - [NCDIRECT_OPTION_INHIBIT_CBREAK][crate::NCDIRECT_OPTION_INHIBIT_CBREAK]
    /// - [NCDIRECT_OPTION_INHIBIT_SETLOCALE][crate::NCDIRECT_OPTION_INHIBIT_SETLOCALE]
    ///
    /// *C style function: [ncdirect_init()][crate::ncdirect_init].*
    pub fn with_flags<'a>(flags: NcDirectFlags) -> NcResult<&'a mut NcDirect> {
        let res = unsafe { crate::ncdirect_init(null(), null_mut(), flags) };
        error_ref_mut![res, "Initializing NcDirect"]
    }

    /// Releases this NcDirect and any associated resources.
    ///
    /// *C style function: [ncdirect_stop()][crate::ncdirect_stop].*
    pub fn stop(&mut self) -> NcResult<()> {
        error![unsafe { crate::ncdirect_stop(self) }, "NcDirect.stop()"]
    }
}

/// ## NcDirect methods: clear, flush, render
impl NcDirect {
    /// Clears the screen.
    ///
    /// *C style function: [ncdirect_clear()][crate::ncdirect_clear].*
    pub fn clear(&mut self) -> NcResult<()> {
        error![unsafe { crate::ncdirect_clear(self) }, "NcDirect.clear()"]
    }

    /// Forces a flush.
    ///
    /// *C style function: [ncdirect_flush()][crate::ncdirect_flush].*
    pub fn flush(&self) -> NcResult<()> {
        error![unsafe { crate::ncdirect_flush(self) }, "NcDirect.clear()"]
    }

    /// Takes the result of [`render_frame`][NcDirect#method.render_frame]
    /// and writes it to the output.
    ///
    /// *C style function: [ncdirect_raster_frame()][crate::ncdirect_raster_frame].*
    pub fn raster_frame(&mut self, frame: &mut NcDirectV, align: NcAlign) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_raster_frame(self, frame, align) },
            "NcDirect.raster_frame()"
        ]
    }

    /// Renders an image using the specified blitter and scaling,
    /// but doesn't write the result.
    ///
    /// The image may be arbitrarily many rows -- the output will scroll --
    /// but will only occupy the column of the cursor, and those to the right.
    ///
    /// To actually write (and free) this, invoke ncdirect_raster_frame().
    ///
    /// `max_y' and 'max_x` (cell geometry, *not* pixel), if greater than 0,
    /// are used for scaling; the terminal's geometry is otherwise used.
    ///
    /// *C style function: [ncdirect_render_frame()][crate::ncdirect_render_frame].*
    pub fn render_frame<'a>(
        &mut self,
        filename: &str,
        blitter: NcBlitter,
        scale: NcScale,
        max_y: NcDim,
        max_x: NcDim,
    ) -> NcResult<&'a mut NcDirectV> {
        let res = unsafe {
            crate::ncdirect_render_frame(self, cstring![filename], blitter, scale, max_y as i32, max_x as i32)
        };
        error_ref_mut![
            res,
            &format!(
                "NcDirect.render_frame({:?}, {:?}, {:?})",
                filename, blitter, scale
            )
        ]
    }

    /// Displays an image using the specified blitter and scaling.
    ///
    /// The image may be arbitrarily many rows -- the output will scroll -- but
    /// will only occupy the column of the cursor, and those to the right.
    ///
    /// The render/raster process can be split by using
    /// [render_frame()][#method.render_frame] and
    /// [raster_frame()][#method.raster_frame].
    ///
    /// *C style function: [ncdirect_render_image()][crate::ncdirect_render_image].*
    pub fn render_image(
        &mut self,
        filename: &str,
        align: NcAlign,
        blitter: NcBlitter,
        scale: NcScale,
    ) -> NcResult<()> {
        error![
            unsafe {
                crate::ncdirect_render_image(self, cstring![filename], align, blitter, scale)
            },
            &format!(
                "NcDirect.render_image({:?}, {:?}, {:?}, {:?})",
                filename, align, blitter, scale
            )
        ]
    }
}

/// ## NcDirect methods: `NcPaletteIndex`, `NcRgb`, `NcStyle` & default color
impl NcDirect {
    /// Sets the foreground [NcPaletteIndex].
    ///
    /// *C style function: [ncdirect_set_fg_palindex()][crate::ncdirect_set_fg_palindex].*
    pub fn set_fg_palindex(&mut self, index: NcPaletteIndex) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_set_fg_palindex(self, index as i32) },
            &format!("NcDirect.set_fg_palindex({})", index)
        ]
    }

    /// Sets the background [NcPaletteIndex].
    ///
    /// *C style function: [ncdirect_set_bg_palindex()][crate::ncdirect_set_bg_palindex].*
    pub fn set_bg_palindex(&mut self, index: NcPaletteIndex) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_set_bg_palindex(self, index as i32) },
            &format!("NcDirect.set_fg_palindex({})", index)
        ]
    }

    /// Returns the number of simultaneous colors claimed to be supported,
    /// if there is color support.
    ///
    /// Note that several terminal emulators advertise more colors than they
    /// actually support, downsampling internally.
    ///
    /// *C style function: [ncdirect_palette_size()][crate::ncdirect_palette_size].*
    pub fn palette_size(&self) -> NcResult<u32> {
        let res = unsafe { crate::ncdirect_palette_size(self) };
        if res == 1 {
            return Err(NcError::with_msg(
                1,
                "No color support ← NcDirect.palette_size()",
            ));
        }
        Ok(res)
    }

    /// Sets the foreground [NcRgb].
    ///
    /// *C style function: [ncdirect_set_fg_rgb()][crate::ncdirect_set_fg_rgb].*
    pub fn set_fg_rgb(&mut self, rgb: NcRgb) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_set_fg_rgb(self, rgb) },
            &format!("NcDirect.set_fg_rgb({})", rgb)
        ]
    }

    /// Sets the background [NcRgb].
    ///
    /// *C style function: [ncdirect_set_bg_rgb()][crate::ncdirect_set_bg_rgb].*
    pub fn set_bg_rgb(&mut self, rgb: NcRgb) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_set_bg_rgb(self, rgb) },
            &format!("NcDirect.set_bg_rgb({})", rgb)
        ]
    }

    /// Sets the foreground [NcComponent] components.
    ///
    /// *C style function: [ncdirect_set_fg_rgb8()][crate::ncdirect_set_fg_rgb8].*
    pub fn set_fg_rgb8(
        &mut self,
        red: NcComponent,
        green: NcComponent,
        blue: NcComponent,
    ) -> NcResult<()> {
        error![
            crate::ncdirect_set_fg_rgb8(self, red, green, blue),
            &format!("NcDirect.set_fg_rgb8({}, {}, {})", red, green, blue)
        ]
    }

    /// Sets the background [NcComponent] components.
    ///
    /// *C style function: [ncdirect_set_bg_rgb()][crate::ncdirect_set_bg_rgb].*
    pub fn set_bg_rgb8(
        &mut self,
        red: NcComponent,
        green: NcComponent,
        blue: NcComponent,
    ) -> NcResult<()> {
        error![
            crate::ncdirect_set_bg_rgb8(self, red, green, blue),
            &format!("NcDirect.set_bg_rgb8({}, {}, {})", red, green, blue)
        ]
    }

    /// Removes the specified styles.
    ///
    /// *C style function: [ncdirect_off_styles()][crate::ncdirect_off_styles].*
    pub fn styles_off(&mut self, stylebits: NcStyle) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_off_styles(self, stylebits.into()) },
            &format!("NcDirect.styles_off({:0X})", stylebits)
        ]
    }

    /// Adds the specified styles.
    ///
    /// *C style function: [ncdirect_on_styles()][crate::ncdirect_on_styles].*
    pub fn styles_on(&mut self, stylebits: NcStyle) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_on_styles(self, stylebits.into()) },
            &format!("NcDirect.styles_on({:0X})", stylebits)
        ]
    }

    /// Sets just the specified styles.
    ///
    /// *C style function: [ncdirect_set_styles()][crate::ncdirect_set_styles].*
    pub fn styles_set(&mut self, stylebits: NcStyle) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_set_styles(self, stylebits.into()) },
            &format!("NcDirect.styles_set({:0X})", stylebits)
        ]
    }

    /// Indicates to use the "default color" for the foreground.
    ///
    /// *C style function: [ncdirect_set_fg_default()][crate::ncdirect_set_fg_default].*
    pub fn set_fg_default(&mut self) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_set_fg_default(self) },
            "NcDirect.set_fg_default()"
        ]
    }

    /// Indicates to use the "default color" for the background.
    ///
    /// *C style function: [ncdirect_set_bg_default()][crate::ncdirect_set_bg_default].*
    pub fn set_bg_default(&mut self) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_set_bg_default(self) },
            "NcDirect.set_bg_default()"
        ]
    }
}

/// ## NcDirect methods: capabilities, cursor, dimensions
impl NcDirect {
    /// Can we reliably use Unicode braille?
    ///
    /// *C style function: [ncdirect_canbraille()][crate::ncdirect_canbraille].*
    pub fn canbraille(&self) -> bool {
        crate::ncdirect_canbraille(self)
    }

    /// Can we set the "hardware" palette?
    ///
    /// Requires the "ccc" terminfo capability.
    ///
    /// *C style function: [ncdirect_canchangecolor()][crate::ncdirect_canchangecolor].*
    pub fn canchangecolor(&self) -> bool {
        crate::ncdirect_canchangecolor(self)
    }

    /// Can we fade?
    ///
    /// Requires either the "rgb" or "ccc" terminfo capability.
    ///
    /// *C style function: [ncdirect_canfade()][crate::ncdirect_canfade].*
    pub fn canfade(&self) -> bool {
        crate::ncdirect_canfade(self)
    }

    /// Can we reliably use Unicode halfblocks?
    ///
    /// *C style function: [ncdirect_canhalfblock()][crate::ncdirect_canhalfblock].*
    pub fn canhalfblock(&self) -> bool {
        crate::ncdirect_canhalfblock(self)
    }

    /// Can we load images?
    ///
    /// Requires being built against FFmpeg/OIIO.
    ///
    /// *C style function: [ncdirect_canopen_images()][crate::ncdirect_canopen_images].*
    pub fn canopen_images(&self) -> bool {
        unsafe { crate::ncdirect_canopen_images(self) }
    }

    /// Can we load videos?
    ///
    /// Requires being built against FFmpeg/OIIO.
    ///
    /// *C style function: [ncdirect_canopen_videos()][crate::ncdirect_canopen_videos].*
    pub fn canopen_videos(&self) -> bool {
        crate::ncdirect_canopen_videos(self)
    }

    /// Can we reliably use Unicode quadrants?
    ///
    /// *C style function: [ncdirect_canquadrant()][crate::ncdirect_canquadrant].*
    pub fn canquadrant(&self) -> bool {
        crate::ncdirect_canquadrant(self)
    }

    /// Can we reliably use Unicode sextants?
    ///
    /// *C style function: [ncdirect_cansextant()][crate::ncdirect_cansextant].*
    pub fn cansextant(&self) -> bool {
        crate::ncdirect_cansextant(self)
    }

    /// Can we directly specify RGB values per cell, or only use palettes?
    ///
    /// *C style function: [ncdirect_cantruecolor()][crate::ncdirect_cantruecolor].*
    pub fn cantruecolor(&self) -> bool {
        crate::ncdirect_cantruecolor(self)
    }

    /// Is our encoding UTF-8?
    ///
    /// Requires LANG being set to a UTF8 locale.
    ///
    /// *C style function: [ncdirect_canutf8()][crate::ncdirect_canutf8].*
    pub fn canutf8(&self) -> bool {
        unsafe { crate::ncdirect_canutf8(self) }
    }

    /// Returns the [`NcCapabilities`].
    ///
    /// *C style function: [ncdirect_capabilities()][crate::ncdirect_capabilities].*
    pub fn capabilities(&self) -> NcCapabilities {
        crate::ncdirect_capabilities(self)
    }

    /// Checks for pixel support.
    ///
    /// Returns `false` for no support, or `true` if pixel output is supported.
    ///
    /// This function must successfully return before NCBLIT_PIXEL is available.
    ///
    /// Must not be called concurrently with either input or rasterization.
    ///
    /// *C style function: [ncdirect_check_pixel_support()][crate::ncdirect_check-pixel_support].*
    #[allow(clippy::wildcard_in_or_patterns)]
    pub fn check_pixel_support(&self) -> NcResult<bool> {
        let res = unsafe { crate::ncdirect_check_pixel_support(self) };
        match res {
            0 => Ok(false),
            1 => Ok(true),
            NCRESULT_ERR | _ => Err(NcError::with_msg(res, "NcDirect.check_pixel_support()")),
        }
    }

    /// Disables the terminal's cursor, if supported.
    ///
    /// *C style function: [ncdirect_cursor_disable()][crate::ncdirect_cursor_disable].*
    pub fn cursor_disable(&mut self) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_cursor_disable(self) },
            "NcDirect.cursor_disable()"
        ]
    }

    /// Enables the terminal's cursor, if supported.
    ///
    /// *C style function: [ncdirect_cursor_enable()][crate::ncdirect_cursor_enable].*
    pub fn cursor_enable(&mut self) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_cursor_enable(self) },
            "NcDirect.cursor_enable()"
        ]
    }

    /// Moves the cursor down any number of rows.
    ///
    /// *C style function: [ncdirect_cursor_down()][crate::ncdirect_cursor_down].*
    pub fn cursor_down(&mut self, rows: NcOffset) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_cursor_down(self, rows as i32) },
            &format!("NcDirect.cursor_down({})", rows)
        ]
    }

    /// Moves the cursor left any number of columns.
    ///
    /// *C style function: [ncdirect_cursor_left()][crate::ncdirect_cursor_left].*
    pub fn cursor_left(&mut self, cols: NcOffset) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_cursor_left(self, cols as i32) },
            &format!("NcDirect.cursor_left({})", cols)
        ]
    }

    /// Moves the cursor right any number of columns.
    ///
    /// *C style function: [ncdirect_cursor_right()][crate::ncdirect_cursor_right].*
    pub fn cursor_right(&mut self, cols: NcOffset) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_cursor_right(self, cols as i32) },
            &format!("NcDirect.cursor_right({})", cols)
        ]
    }

    /// Moves the cursor up any number of rows.
    ///
    /// *C style function: [ncdirect_cursor_up()][crate::ncdirect_cursor_up].*
    pub fn cursor_up(&mut self, rows: NcOffset) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_cursor_up(self, rows as i32) },
            &format!("NcDirect.cursor_up({})", rows)
        ]
    }

    /// Sets the cursor to the specified row `y`, column `x`.
    ///
    /// *C style function: [ncdirect_cursor_move_yx()][crate::ncdirect_cursor_move_yx].*
    pub fn cursor_set_yx(&mut self, y: NcDim, x: NcDim) -> NcResult<()> {
        error![unsafe { crate::ncdirect_cursor_move_yx(self, y as i32, x as i32) }]
    }

    /// Sets the cursor to the specified row `y`.
    ///
    /// *(No equivalent C style function)*
    pub fn cursor_set_y(&mut self, y: NcDim) -> NcResult<()> {
        error![unsafe { crate::ncdirect_cursor_move_yx(self, y as i32, -1) }]
    }

    /// Sets the cursor to the specified column `x`.
    ///
    /// *(No equivalent C style function)*
    pub fn cursor_set_x(&mut self, x: NcDim) -> NcResult<()> {
        error![unsafe { crate::ncdirect_cursor_move_yx(self, -1, x as i32) }]
    }

    /// Gets the cursor (y, x) position, when supported.
    ///
    /// This requires writing to the terminal, and then reading from it.
    /// If the terminal doesn't reply, or doesn't reply in a way we understand,
    /// the results might be detrimental.
    ///
    /// *C style function: [ncdirect_cursor_yx()][crate::ncdirect_cursor_yx].*
    pub fn cursor_yx(&mut self) -> NcResult<(NcDim, NcDim)> {
        let (mut y, mut x) = (0, 0);
        error![
            unsafe { crate::ncdirect_cursor_yx(self, &mut y, &mut x) },
            "",
            (y as NcDim, x as NcDim)
        ]
    }

    /// Pushes the cursor location to the terminal's stack.
    ///
    /// The depth of this stack, and indeed its existence, is terminal-dependent.
    ///
    /// *C style function: [ncdirect_cursor_push()][crate::ncdirect_cursor_push].*
    pub fn cursor_push(&mut self) -> NcResult<()> {
        error![unsafe { crate::ncdirect_cursor_push(self) }]
    }

    /// Pops the cursor location from the terminal's stack.
    ///
    /// The depth of this stack, and indeed its existence, is terminal-dependent.
    ///
    /// *C style function: [ncdirect_cursor_pop()][crate::ncdirect_cursor_pop].*
    pub fn cursor_pop(&mut self) -> NcResult<()> {
        error![unsafe { crate::ncdirect_cursor_pop(self) }]
    }

    /// Gets the current number of rows.
    ///
    /// *C style function: [ncdirect_dim_y()][crate::ncdirect_dim_y].*
    pub fn dim_y(&mut self) -> NcDim {
        unsafe { crate::ncdirect_dim_y(self) as NcDim }
    }

    /// Gets the current number of columns.
    ///
    /// *C style function: [ncdirect_dim_x()][crate::ncdirect_dim_x].*
    pub fn dim_x(&mut self) -> NcDim {
        unsafe { crate::ncdirect_dim_x(self) as NcDim }
    }

    /// Gets the current number of rows and columns.
    ///
    /// *C style function: [ncdirect_dim_y()][crate::ncdirect_dim_y].*
    pub fn dim_yx(&mut self) -> (NcDim, NcDim) {
        let y = unsafe { crate::ncdirect_dim_y(self) as NcDim };
        let x = unsafe { crate::ncdirect_dim_x(self) as NcDim };
        (y, x)
    }

    /// Returns the name of the detected terminal.
    ///
    /// *C style function: [ncdirect_detected_terminal()][crate::ncdirect_detected_terminal].*
    pub fn detected_terminal(&self) -> String {
        rstring![crate::ncdirect_detected_terminal(self)].to_string()
    }
}

/// ## NcDirect methods: I/O
impl NcDirect {
    /// Returns a [char] representing a single unicode point.
    ///
    /// If an event is processed, the return value is the `id` field from that
    /// event.
    ///
    /// Provide a None `time` to block at length, a `time` of 0 for non-blocking
    /// operation, and otherwise a timespec to bound blocking.
    ///
    /// Signals in sigmask (less several we handle internally) will be atomically
    /// masked and unmasked per [ppoll(2)](https://linux.die.net/man/2/ppoll).
    ///
    /// `*sigmask` should generally contain all signals.
    ///
    /// *C style function: [ncdirect_getc()][crate::ncdirect_getc].*
    //
    // CHECK returns 0 on a timeout.
    pub fn getc(
        &mut self,
        time: Option<NcTime>,
        sigmask: Option<&mut sigset_t>,
        input: Option<&mut NcInput>,
    ) -> NcResult<char> {
        let ntime;
        if let Some(time) = time {
            ntime = &time as *const _;
        } else {
            ntime = null();
        }

        let nsigmask;
        if let Some(sigmask) = sigmask {
            nsigmask = sigmask as *mut _;
        } else {
            nsigmask = null_mut() as *mut _;
        }
        let ninput;
        if let Some(input) = input {
            ninput = input as *mut _;
        } else {
            ninput = null_mut();
        }
        let c = unsafe {
            core::char::from_u32_unchecked(crate::ncdirect_getc(self, ntime, nsigmask, ninput))
        };
        if c as u32 as i32 == NCRESULT_ERR {
            return Err(NcError::new());
        }
        Ok(c)
    }

    ///
    /// *C style function: [ncdirect_getc_nblock()][crate::ncdirect_getc_nblock].*
    pub fn getc_nblock(&mut self, input: &mut NcInput) -> char {
        crate::ncdirect_getc_nblock(self, input)
    }

    ///
    /// *C style function: [ncdirect_getc_blocking()][crate::ncdirect_getc_blocking].*
    pub fn getc_blocking(&mut self, input: &mut NcInput) -> char {
        crate::ncdirect_getc_blocking(self, input)
    }

    /// Get a file descriptor suitable for input event poll()ing.
    ///
    /// When this descriptor becomes available, you can call
    /// [getc_nblock()][NcDirect#method.getc_nblock], and input ought be ready.
    ///
    /// This file descriptor is not necessarily the file descriptor associated
    /// with stdin (but it might be!).
    ///
    /// *C style function: [ncdirect_inputready_fd()][crate::ncdirect_inputready_fd].*
    pub fn inputready_fd(&mut self) -> NcResult<()> {
        error![unsafe { crate::ncdirect_inputready_fd(self) }]
    }

    /// Outputs the `string` according to the `channels`, and
    /// returns the total number of characters written on success.
    ///
    /// Note that it does not explicitly flush output buffers, so it will not
    /// necessarily be immediately visible.
    ///
    /// It will fail if the NcDirect context and the foreground channel
    /// are both marked as using the default color.
    ///
    /// *C style function: [ncdirect_putstr()][crate::ncdirect_putstr].*
    pub fn putstr(&mut self, channels: NcChannels, string: &str) -> NcResult<()> {
        error![
            unsafe { crate::ncdirect_putstr(self, channels, cstring![string]) },
            &format!("NcDirect.putstr({:0X}, {:?})", channels, string)
        ]
    }

    /// Reads a (heap-allocated) line of text using the Readline library.
    ///
    /// Initializes Readline the first time it's called.
    ///
    /// For input to be echoed to the terminal, it is necessary that the flag
    /// [NCDIRECT_OPTION_INHIBIT_CBREAK][crate::NCDIRECT_OPTION_INHIBIT_CBREAK]
    /// be provided to the constructor.
    ///
    /// *C style function: [ncdirect_readline()][crate::ncdirect_readline].*
    pub fn readline(&mut self, prompt: &str) -> NcResult<&str> {
        let res = unsafe { crate::ncdirect_readline(self, cstring![prompt]) };
        if !res.is_null() {
            return Ok(rstring![res]);
        } else {
            Err(NcError::with_msg(
                NCRESULT_ERR,
                &format!["NcDirect.readline({})", prompt],
            ))
        }
    }

    /// Draws a box with its upper-left corner at the current cursor position,
    /// having dimensions `ylen` * `xlen`.
    ///
    /// See NcPlane.[box()][crate::NcPlane#method.box] for more information.
    ///
    /// The minimum box size is 2x2, and it cannot be drawn off-screen.
    ///
    /// `wchars` is an array of 6 characters: UL, UR, LL, LR, HL, VL.
    ///
    /// *C style function: [ncdirect_box()][crate::ncdirect_box].*
    // TODO: CHECK, specially wchars.
    pub fn r#box(
        &mut self,
        ul: NcChannels,
        ur: NcChannels,
        ll: NcChannels,
        lr: NcChannels,
        wchars: &[char; 6],
        y_len: NcDim,
        x_len: NcDim,
        ctlword: u32,
    ) -> NcResult<()> {
        error![
            unsafe {
                let wchars = core::mem::transmute(wchars);
                crate::ncdirect_box(
                    self,
                    ul,
                    ur,
                    ll,
                    lr,
                    wchars,
                    y_len as i32,
                    x_len as i32,
                    ctlword,
                )
            },
            &format!(
                "NcDirect.box({:0X}, {:0X}, {:0X}, {:0X}, {:?}, {}, {}, {})",
                ul, ur, ll, lr, wchars, y_len, x_len, ctlword
            )
        ]
    }

    /// NcDirect.[box()][NcDirect#method.box] with the double box-drawing characters.
    ///
    /// *C style function: [ncdirect_double_box()][crate::ncdirect_double_box].*
    pub fn double_box(
        &mut self,
        ul: NcChannels,
        ur: NcChannels,
        ll: NcChannels,
        lr: NcChannels,
        y_len: NcDim,
        x_len: NcDim,
        ctlword: u32,
    ) -> NcResult<()> {
        error![unsafe {
            crate::ncdirect_double_box(self, ul, ur, ll, lr, y_len as i32, x_len as i32, ctlword)
        }]
    }

    /// NcDirect.[box()][NcDirect#method.box] with the rounded box-drawing characters.
    ///
    /// *C style function: [ncdirect_rounded_box()][crate::ncdirect_rounded_box].*
    pub fn rounded_box(
        &mut self,
        ul: NcChannels,
        ur: NcChannels,
        ll: NcChannels,
        lr: NcChannels,
        y_len: NcDim,
        x_len: NcDim,
        ctlword: u32,
    ) -> NcResult<()> {
        error![unsafe {
            crate::ncdirect_rounded_box(self, ul, ur, ll, lr, y_len as i32, x_len as i32, ctlword)
        }]
    }
    /// Draws horizontal lines using the specified [NcChannels]s, interpolating
    /// between them as we go.
    ///
    /// All lines start at the current cursor position.
    ///
    /// The [NcEgc] at `egc` may not use more than one column.
    ///
    /// For a horizontal line, `len` cannot exceed the screen width minus the
    /// cursor's offset.
    ///
    /// *C style function: [ncdirect_hline_interp()][crate::ncdirect_hline_interp].*
    #[inline]
    pub fn hline_interp(
        &mut self,
        egc: &NcEgc,
        len: NcDim,
        h1: NcChannels,
        h2: NcChannels,
    ) -> NcResult<()> {
        // https://github.com/dankamongmen/notcurses/issues/1339
        #[cfg(any(target_arch = "x86_64", target_arch = "i686"))]
        let egc_ptr = &(*egc as i8);
        #[cfg(not(any(target_arch = "x86_64", target_arch = "i686")))]
        let egc_ptr = &(*egc as u8);

        error![unsafe { crate::ncdirect_hline_interp(self, egc_ptr, len as i32, h1, h2) }]
    }

    /// Draws horizontal lines using the specified [NcChannels]s, interpolating
    /// between them as we go.
    ///
    /// All lines start at the current cursor position.
    ///
    /// The [NcEgc] at `egc` may not use more than one column.
    ///
    /// For a vertical line, `len` may be as long as you'd like; the screen
    /// will scroll as necessary.
    ///
    /// *C style function: [ncdirect_vline_interp()][crate::ncdirect_vline_interp].*
    #[inline]
    pub fn vline_interp(
        &mut self,
        egc: &NcEgc,
        len: NcDim,
        h1: NcChannels,
        h2: NcChannels,
    ) -> NcResult<()> {
        // https://github.com/dankamongmen/notcurses/issues/1339
        #[cfg(any(target_arch = "x86_64", target_arch = "i686"))]
        let egc_ptr = &(*egc as i8);
        #[cfg(not(any(target_arch = "x86_64", target_arch = "i686")))]
        let egc_ptr = &(*egc as u8);

        error![unsafe { crate::ncdirect_vline_interp(self, egc_ptr, len as i32, h1, h2) }]
    }
}
