//! `NcPlane*` methods and associated functions.
use core::{
    ptr::{null, null_mut},
    slice::from_raw_parts_mut,
};

use crate::{
    c_api, cstring, error, error_ref, error_ref_mut, rstring_free, Nc, NcAlign, NcAlpha, NcBlitter,
    NcBoxMask, NcCell, NcChannel, NcChannels, NcComponent, NcDim, NcError, NcFadeCb, NcFile,
    NcIntResult, NcIntResultApi, NcOffset, NcPaletteIndex, NcPixelGeometry, NcPlane,
    NcPlaneOptions, NcResizeCb, NcResult, NcRgb, NcStyle, NcTime,
};

/// # NcPlaneOptions Constructors
impl NcPlaneOptions {
    /// New NcPlaneOptions using the horizontal x.
    pub fn new(y: NcOffset, x: NcOffset, rows: NcDim, cols: NcDim) -> Self {
        Self::with_flags(y, x, rows, cols, None, 0, 0, 0)
    }

    /// New NcPlaneOptions with horizontal alignment.
    pub fn new_aligned(y: NcOffset, align: NcAlign, rows: NcDim, cols: NcDim) -> Self {
        Self::with_flags_aligned(y, align, rows, cols, None, NcPlaneOptions::HORALIGNED)
    }

    /// New NcPlaneOptions, with flags.
    pub fn with_flags(
        y: NcOffset,
        x: NcOffset,
        rows: NcDim,
        cols: NcDim,
        resizecb: Option<NcResizeCb>,
        flags: u64,
        margin_b: NcOffset,
        margin_r: NcOffset,
    ) -> Self {
        NcPlaneOptions {
            y: y as i32,
            x: x as i32,
            rows,
            cols,
            userptr: null_mut(),
            name: null(),
            resizecb: c_api::ncresizecb_to_c(resizecb),
            flags,
            margin_b: margin_b as i32,
            margin_r: margin_r as i32,
        }
    }

    /// New NcPlaneOptions, with flags and horizontal alignment.
    ///
    /// Note: Already includes the
    /// [NcPlaneOptions::HORALIGNED][NcPlaneOptions#associatedconstant.HORALIGNED]
    /// flag.
    pub fn with_flags_aligned(
        y: NcOffset,
        align: NcAlign,
        rows: NcDim,
        cols: NcDim,
        resizecb: Option<NcResizeCb>,
        flags: u64,
    ) -> Self {
        let flags = NcPlaneOptions::HORALIGNED | flags;
        NcPlaneOptions {
            y: y as i32,
            x: align as i32,
            rows,
            cols,
            userptr: null_mut(),
            name: null(),
            resizecb: c_api::ncresizecb_to_c(resizecb),
            flags,
            margin_b: 0,
            margin_r: 0,
        }
    }
}

/// # NcPlane constructors & destructors
impl NcPlane {
    /// New `NcPlane`.
    ///
    /// The returned plane will be the top, bottom, and root of this new pile.
    ///
    /// *C style function: [ncpile_create()][c_api::ncpile_create].*
    pub fn new<'a>(
        nc: &mut Nc,
        y: NcOffset,
        x: NcOffset,
        rows: NcDim,
        cols: NcDim,
    ) -> NcResult<&'a mut NcPlane> {
        Self::with_options(nc, &NcPlaneOptions::new(y, x, rows, cols))
    }

    /// New `NcPlane`, expects an [NcPlaneOptions] struct.
    ///
    /// The returned plane will be the top, bottom, and root of this new pile.
    ///
    /// *C style function: [ncpile_create()][c_api::ncpile_create].*
    pub fn with_options<'a>(nc: &mut Nc, options: &NcPlaneOptions) -> NcResult<&'a mut NcPlane> {
        error_ref_mut![
            unsafe { c_api::ncpile_create(nc, options) },
            &format!["NcPlane::with_options(Nc, {:?})", options]
        ]
    }

    /// New `NcPlane`, bound to another NcPlane.
    ///
    /// *C style function: [ncplane_create()][c_api::ncplane_create].*
    pub fn new_bound<'a>(
        bound_to: &mut NcPlane,
        y: NcOffset,
        x: NcOffset,
        rows: NcDim,
        cols: NcDim,
    ) -> NcResult<&'a mut NcPlane> {
        Self::with_options_bound(bound_to, &NcPlaneOptions::new(y, x, rows, cols))
    }

    /// New `NcPlane`, bound to another plane, expects an [NcPlaneOptions] struct.
    ///
    /// *C style function: [ncplane_create()][c_api::ncplane_create].*
    pub fn with_options_bound<'a>(
        bound_to: &mut NcPlane,
        options: &NcPlaneOptions,
    ) -> NcResult<&'a mut NcPlane> {
        error_ref_mut![
            unsafe { c_api::ncplane_create(bound_to, options) },
            &format!("NcPlane::with_options_bound(NcPlane, {:?})", options)
        ]
    }

    /// New `NcPlane`, with the same dimensions of the terminal.
    ///
    /// The returned plane will be the top, bottom, and root of this new pile.
    ///
    /// *(No equivalent C style function)*
    pub fn with_termsize<'a>(nc: &mut Nc) -> NcResult<&'a mut NcPlane> {
        let (trows, tcols) = c_api::notcurses_term_dim_yx(nc);
        assert![(trows > 0) & (tcols > 0)];
        Self::with_options(
            nc,
            &NcPlaneOptions::new(0, 0, trows as NcDim, tcols as NcDim),
        )
    }

    /// Destroys this `NcPlane`.
    ///
    /// None of its contents will be visible after the next render call.
    /// It is an error to attempt to destroy the standard plane.
    ///
    /// *C style function: [ncplane_destroy()][c_api::ncplane_destroy].*
    pub fn destroy(&mut self) -> NcResult<()> {
        error![unsafe { c_api::ncplane_destroy(self) }, "NcPlane.destroy()"]
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: `NcAlpha`
impl NcPlane {
    /// Gets the foreground [`NcAlpha`] from this `NcPlane`, shifted to LSBs.
    ///
    /// *C style function: [ncplane_fg_alpha()][c_api::ncplane_fg_alpha].*
    #[inline]
    pub fn fg_alpha(&self) -> NcAlpha {
        c_api::ncchannels_fg_alpha(c_api::ncplane_channels(self))
    }

    /// Gets the background [`NcAlpha`] for this `NcPlane`, shifted to LSBs.
    ///
    /// *C style function: [ncplane_bg_alpha()][c_api::ncplane_bg_alpha].*
    #[inline]
    pub fn bg_alpha(&self) -> NcAlpha {
        c_api::ncchannels_bg_alpha(c_api::ncplane_channels(self))
    }

    /// Sets the foreground [`NcAlpha`] from this `NcPlane`.
    ///
    /// *C style function: [ncplane_set_fg_alpha()][c_api::ncplane_set_fg_alpha].*
    pub fn set_fg_alpha(&mut self, alpha: NcAlpha) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_set_fg_alpha(self, alpha as i32) },
            &format!("NcPlane.set_fg_alpha({:0X})", alpha)
        ]
    }

    /// Sets the background [`NcAlpha`] for this `NcPlane`.
    ///
    /// *C style function: [ncplane_set_bg_alpha()][c_api::ncplane_set_bg_alpha].*
    pub fn set_bg_alpha(&mut self, alpha: NcAlpha) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_set_bg_alpha(self, alpha as i32) },
            &format!("NcPlane.set_bg_alpha({:0X})", alpha)
        ]
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: `NcChannel`
impl NcPlane {
    /// Gets the current [`NcChannels`] from this `NcPlane`.
    ///
    /// *C style function: [ncplane_channels()][c_api::ncplane_channels].*
    pub fn channels(&self) -> NcChannels {
        c_api::ncplane_channels(self)
    }

    /// Sets the current [`NcChannels`] for this `NcPlane`.
    ///
    /// *C style function: [ncplane_set_channels()][c_api::ncplane_set_channels].*
    pub fn set_channels(&mut self, channels: NcChannels) {
        c_api::ncplane_set_channels(self, channels);
    }

    /// Gets the foreground [`NcChannel`] from an [NcPlane].
    ///
    /// *C style function: [ncplane_fchannel()][c_api::ncplane_fchannel].*
    #[inline]
    pub fn fchannel(&self) -> NcChannel {
        c_api::ncchannels_fchannel(c_api::ncplane_channels(self))
    }

    /// Gets the background [`NcChannel`] from an [NcPlane].
    ///
    /// *C style function: [ncplane_bchannel()][c_api::ncplane_bchannel].*
    #[inline]
    pub fn bchannel(&self) -> NcChannel {
        c_api::ncchannels_bchannel(c_api::ncplane_channels(self))
    }

    /// Sets the current foreground [`NcChannel`] for this `NcPlane`.
    /// Returns the updated [`NcChannels`].
    ///
    /// *C style function: [ncplane_set_fchannel()][c_api::ncplane_set_fchannel].*
    pub fn set_fchannel(&mut self, channel: NcChannel) -> NcChannels {
        c_api::ncplane_set_fchannel(self, channel)
    }

    /// Sets the current background [`NcChannel`] for this `NcPlane`.
    /// Returns the updated [`NcChannels`].
    ///
    /// *C style function: [ncplane_set_bchannel()][c_api::ncplane_set_bchannel].*
    pub fn set_bchannel(&mut self, channel: NcChannel) -> NcChannels {
        c_api::ncplane_set_bchannel(self, channel)
    }

    /// Sets the given [`NcChannels`]s throughout the specified region,
    /// keeping content and attributes unchanged.
    ///
    /// Returns the number of cells set.
    ///
    /// It is an error for any coordinate to be outside the plane.
    ///
    /// *C style function: [ncplane_stain()][c_api::ncplane_stain].*
    pub fn stain(
        &mut self,
        y: Option<NcDim>,
        x: Option<NcDim>,
        y_stop: Option<NcDim>,
        x_stop: Option<NcDim>,
        ul: NcChannels,
        ur: NcChannels,
        ll: NcChannels,
        lr: NcChannels,
    ) -> NcResult<u32> {
        let res = unsafe {
            c_api::ncplane_stain(
                self,
                y.unwrap_or(NcDim::MAX) as i32,
                x.unwrap_or(NcDim::MAX) as i32,
                y_stop.unwrap_or(0),
                x_stop.unwrap_or(0),
                ul,
                ur,
                ll,
                lr,
            )
        };
        error![
            res,
            &format!(
                "NcPlane.stain({:?}, {:?}, {:?}, {:?}, {:0X}, {:0X}, {:0X}, {:0X})",
                y, x, y_stop, x_stop, ul, ur, ll, lr
            ),
            res as u32
        ]
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: `NcComponent`, `NcRgb` & default color
impl NcPlane {
    /// Gets the foreground RGB [`NcComponent`]s from this `NcPlane`.
    ///
    /// *C style function: [ncplane_fg_rgb8()][c_api::ncplane_fg_rgb8].*
    #[inline]
    pub fn fg_rgb8(&self) -> (NcComponent, NcComponent, NcComponent) {
        let (mut r, mut g, mut b) = (0, 0, 0);
        let _ = c_api::ncchannels_fg_rgb8(c_api::ncplane_channels(self), &mut r, &mut g, &mut b);
        (r, g, b)
    }

    /// Gets the background RGB [`NcComponent`]s from this `NcPlane`.
    ///
    /// *C style function: [ncplane_bg_rgb8()][c_api::ncplane_bg_rgb8].*
    #[inline]
    pub fn bg_rgb8(&self) -> (NcComponent, NcComponent, NcComponent) {
        let (mut r, mut g, mut b) = (0, 0, 0);
        let _ = c_api::ncchannels_bg_rgb8(c_api::ncplane_channels(self), &mut r, &mut g, &mut b);
        (r, g, b)
    }

    /// Sets the foreground RGB [`NcComponent`]s for this `NcPlane`.
    ///
    /// If the terminal does not support directly-specified 3x8b cells
    /// (24-bit "TrueColor", indicated by the "RGB" terminfo capability),
    /// the provided values will be interpreted in some lossy fashion.
    ///
    /// "HP-like" terminals require setting foreground and background at the same
    /// time using "color pairs"; notcurses will manage color pairs transparently.
    ///
    /// *C style function: [ncplane_set_fg_rgb8()][c_api::ncplane_set_fg_rgb8].*
    pub fn set_fg_rgb8(&mut self, red: NcComponent, green: NcComponent, blue: NcComponent) {
        unsafe {
            // Can't fail because of type enforcing.
            let _ = c_api::ncplane_set_fg_rgb8(self, red as u32, green as u32, blue as u32);
        }
    }

    /// Sets the background RGB [`NcComponent`]s for this `NcPlane`.
    ///
    /// If the terminal does not support directly-specified 3x8b cells
    /// (24-bit "TrueColor", indicated by the "RGB" terminfo capability),
    /// the provided values will be interpreted in some lossy fashion.
    ///
    /// "HP-like" terminals require setting foreground and background at the same
    /// time using "color pairs"; notcurses will manage color pairs transparently.
    ///
    /// *C style function: [ncplane_set_bg_rgb8()][c_api::ncplane_set_bg_rgb8].*
    pub fn set_bg_rgb8(&mut self, red: NcComponent, green: NcComponent, blue: NcComponent) {
        unsafe {
            // Can't fail because of type enforcing.
            let _ = c_api::ncplane_set_bg_rgb8(self, red as u32, green as u32, blue as u32);
        }
    }

    /// Gets the foreground [`NcRgb`] from this `NcPlane`, shifted to LSBs.
    ///
    /// *C style function: [ncplane_fg_rgb()][c_api::ncplane_fg_rgb].*
    #[inline]
    pub fn fg_rgb(&self) -> NcRgb {
        c_api::ncchannels_fg_rgb(c_api::ncplane_channels(self))
    }

    /// Gets the background [`NcRgb`] from this `NcPlane`, shifted to LSBs.
    ///
    /// *C style function: [ncplane_bg_rgb()][c_api::ncplane_bg_rgb].*
    #[inline]
    pub fn bg_rgb(&self) -> NcRgb {
        c_api::ncchannels_bg_rgb(c_api::ncplane_channels(self))
    }

    /// Sets the foreground [`NcRgb`] for this `NcPlane`.
    ///
    /// *C style function: [ncplane_set_fg_rgb()][c_api::ncplane_set_fg_rgb].*
    #[inline]
    pub fn set_fg_rgb(&mut self, rgb: NcRgb) {
        unsafe {
            c_api::ncplane_set_fg_rgb(self, rgb);
        }
    }

    /// Sets the background [`NcRgb`] for this `NcPlane`.
    ///
    /// *C style function: [ncplane_set_bg_rgb()][c_api::ncplane_set_bg_rgb].*
    #[inline]
    pub fn set_bg_rgb(&mut self, rgb: NcRgb) {
        unsafe {
            c_api::ncplane_set_bg_rgb(self, rgb);
        }
    }

    /// Is this `NcPlane`'s foreground using the "default foreground color"?
    ///
    /// *C style function: [ncplane_fg_default_p()][c_api::ncplane_fg_default_p].*
    #[inline]
    pub fn fg_default(&self) -> bool {
        c_api::ncchannels_fg_default_p(c_api::ncplane_channels(self))
    }

    /// Is this `NcPlane`'s background using the "default background color"?
    ///
    /// *C style function: [ncplane_bg_default_p()][c_api::ncplane_bg_default_p].*
    #[inline]
    pub fn bg_default(&self) -> bool {
        c_api::ncchannels_bg_default_p(c_api::ncplane_channels(self))
    }

    /// Uses the default color for the foreground.
    ///
    /// *C style function: [ncplane_set_fg_default()][c_api::ncplane_set_fg_default].*
    #[inline]
    pub fn set_fg_default(&mut self) {
        unsafe {
            c_api::ncplane_set_fg_default(self);
        }
    }

    /// Uses the default color for the background.
    ///
    /// *C style function: [ncplane_set_bg_default()][c_api::ncplane_set_bg_default].*
    #[inline]
    pub fn set_bg_default(&mut self) {
        unsafe {
            c_api::ncplane_set_bg_default(self);
        }
    }

    /// Marks the foreground as NOT using the default color.
    ///
    /// Returns the new [`NcChannels`].
    ///
    /// *C style function: [ncplane_set_fg_not_default()][c_api::ncplane_set_fg_not_default].*
    //
    // Not in the C API
    #[inline]
    pub fn set_fg_not_default(&mut self) -> NcChannels {
        c_api::ncplane_set_fg_not_default(self)
    }

    /// Marks the background as NOT using the default color.
    ///
    /// Returns the new [`NcChannels`].
    ///
    /// *C style function: [ncplane_set_bg_not_default()][c_api::ncplane_set_bg_not_default].*
    //
    // Not in the C API
    #[inline]
    pub fn set_bg_not_default(&mut self) -> NcChannels {
        c_api::ncplane_set_bg_not_default(self)
    }

    /// Marks both the foreground and background as using the default color.
    ///
    /// Returns the new [`NcChannels`].
    ///
    /// *C style function: [ncplane_set_default()][c_api::ncplane_set_default].*
    //
    // Not in the C API
    #[inline]
    pub fn set_default(&mut self) -> NcChannels {
        c_api::ncplane_set_default(self)
    }

    /// Marks both the foreground and background as NOT using the default color.
    ///
    /// Returns the new [`NcChannels`].
    ///
    /// *C style function: [ncplane_set_not_default()][c_api::ncplane_set_not_default].*
    //
    // Not in the C API
    #[inline]
    pub fn set_not_default(&mut self) -> NcChannels {
        c_api::ncplane_set_not_default(self)
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: `NcStyle` & `PaletteIndex`
impl NcPlane {
    /// Sets the given style throughout the specified region, keeping content
    /// and channels unchanged.
    ///
    /// The upper left corner is at `y`, `x`, and `None` may be specified to
    /// indicate the cursor's position in that dimension.
    ///
    /// The lower right corner is specified by `stop_y`, `stop_x`, and `None`
    /// may be specified to go through the boundary of the plane in that axis
    /// (same as `0`).
    ///
    /// It is an error for any coordinate to be outside the plane.
    ///
    /// Returns the number of cells set.
    ///
    /// *C style function: [ncplane_format()][c_api::ncplane_format].*
    pub fn format(
        &mut self,
        y: Option<NcDim>,
        x: Option<NcDim>,
        stop_y: Option<NcDim>,
        stop_x: Option<NcDim>,
        stylemask: NcStyle,
    ) -> NcResult<NcDim> {
        let res = unsafe {
            c_api::ncplane_format(
                self,
                y.unwrap_or(NcDim::MAX) as i32,
                x.unwrap_or(NcDim::MAX) as i32,
                stop_y.unwrap_or(0),
                stop_x.unwrap_or(0),
                stylemask,
            )
        };
        error![
            res,
            &format!(
                "NcPlane.format({:?}, {:?}, {:?}, {:?}, {:0X})",
                y, x, stop_y, stop_x, stylemask
            ),
            res as u32
        ]
    }

    /// Returns the current style for this `NcPlane`.
    ///
    /// *C style function: [ncplane_styles()][c_api::ncplane_styles].*
    pub fn styles(&self) -> NcStyle {
        unsafe { c_api::ncplane_styles(self) }
    }

    /// Removes the specified styles from this `NcPlane`'s existing spec.
    ///
    /// *C style function: [ncplane_off_styles()][c_api::ncplane_off_styles].*
    pub fn off_styles(&mut self, stylemask: NcStyle) {
        unsafe {
            c_api::ncplane_off_styles(self, stylemask as u32);
        }
    }

    /// Adds the specified styles to this `NcPlane`'s existing spec.
    ///
    /// *C style function: [ncplane_on_styles()][c_api::ncplane_on_styles].*
    pub fn on_styles(&mut self, stylemask: NcStyle) {
        unsafe {
            c_api::ncplane_on_styles(self, stylemask as u32);
        }
    }

    /// Sets just the specified styles for this `NcPlane`.
    ///
    /// *C style function: [ncplane_set_styles()][c_api::ncplane_set_styles].*
    pub fn set_styles(&mut self, stylemask: NcStyle) {
        unsafe {
            c_api::ncplane_set_styles(self, stylemask as u32);
        }
    }

    /// Sets this `NcPlane`'s foreground [NcPaletteIndex].
    ///
    /// Also sets the foreground palette index bit, sets it foreground-opaque,
    /// and clears the foreground default color bit.
    ///
    /// *C style function: [ncplane_set_fg_palindex()][c_api::ncplane_set_fg_palindex].*
    pub fn set_fg_palindex(&mut self, palindex: NcPaletteIndex) {
        unsafe {
            c_api::ncplane_set_fg_palindex(self, palindex as i32);
        }
    }

    /// Sets this `NcPlane`'s background [NcPaletteIndex].
    ///
    /// Also sets the background palette index bit, sets it background-opaque,
    /// and clears the background default color bit.
    ///
    /// *C style function: [ncplane_set_bg_palindex()][c_api::ncplane_set_bg_palindex].*
    pub fn set_bg_palindex(&mut self, palindex: NcPaletteIndex) {
        unsafe {
            c_api::ncplane_set_bg_palindex(self, palindex as i32);
        }
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: `NcCell` & strings
impl NcPlane {
    /// Retrieves the current contents of the [`NcCell`] under the cursor,
    /// returning the `EGC` and writing out the [`NcStyle`] and the [`NcChannels`].
    ///
    /// *C style function: [ncplane_at_cursor()][c_api::ncplane_at_cursor].*
    pub fn at_cursor(
        &mut self,
        stylemask: &mut NcStyle,
        channels: &mut NcChannels,
    ) -> NcResult<String> {
        let egc = unsafe { c_api::ncplane_at_cursor(self, stylemask, channels) };
        if egc.is_null() {
            return Err(NcError::with_msg(
                NcIntResult::ERR,
                &format!("NcPlane.at_cursor({:0X}, {:0X})", stylemask, channels),
            ));
        }
        Ok(rstring_free![egc])
    }

    /// Retrieves the current contents of the [`NcCell`] under the cursor
    /// into `cell`. Returns the number of bytes in the `EGC`.
    ///
    /// This NcCell is invalidated if the associated NcPlane is destroyed.
    ///
    /// *C style function: [ncplane_at_cursor_cell()][c_api::ncplane_at_cursor_cell].*
    #[inline]
    pub fn at_cursor_cell(&mut self, cell: &mut NcCell) -> NcResult<u32> {
        let bytes = unsafe { c_api::ncplane_at_cursor_cell(self, cell) };
        error![
            bytes,
            &format!("NcPlane.at_cursor_cell({:?})", cell),
            bytes as u32
        ]
    }

    /// Retrieves the current contents of the specified [`NcCell`], returning the
    /// `EGC` and writing out the [`NcStyle`] and the [`NcChannels`].
    ///
    /// *C style function: [ncplane_at_yx()][c_api::ncplane_at_yx].*
    pub fn at_yx(
        &mut self,
        y: NcDim,
        x: NcDim,
        stylemask: &mut NcStyle,
        channels: &mut NcChannels,
    ) -> NcResult<String> {
        let egc = unsafe { c_api::ncplane_at_yx(self, y as i32, x as i32, stylemask, channels) };
        if egc.is_null() {
            return Err(NcError::with_msg(
                NcIntResult::ERR,
                &format!(
                    "NcPlane.at_yx({}, {}, {:0X}, {:0X})",
                    y, x, stylemask, channels
                ),
            ));
        }
        Ok(rstring_free![egc])
    }

    /// Retrieves the current contents of the specified [`NcCell`] into `cell`.
    /// Returns the number of bytes in the `EGC`.
    ///
    /// This NcCell is invalidated if the associated plane is destroyed.
    ///
    /// *C style function: [ncplane_at_yx_cell()][c_api::ncplane_at_yx_cell].*
    #[inline]
    pub fn at_yx_cell(&mut self, y: NcDim, x: NcDim, cell: &mut NcCell) -> NcResult<u32> {
        let bytes = unsafe { c_api::ncplane_at_yx_cell(self, y as i32, x as i32, cell) };
        error![
            bytes,
            &format!("NcPlane.at_yx_cell({}, {}, {:?})", y, x, cell),
            bytes as u32
        ]
    }

    /// Extracts this `NcPlane`'s base [`NcCell`].
    ///
    /// The reference is invalidated if this `NcPlane` is destroyed.
    ///
    /// *C style function: [ncplane_base()][c_api::ncplane_base].*
    pub fn base(&mut self) -> NcResult<NcCell> {
        let mut cell = NcCell::new();
        let res = unsafe { c_api::ncplane_base(self, &mut cell) };
        error![res, "NcPlane.base()", cell]
    }

    /// Sets this `NcPlane`'s base [`NcCell`] from its components.
    ///
    /// Returns the number of bytes copied out of `egc` if succesful.
    ///
    /// It will be used for purposes of rendering anywhere that the `NcPlane`'s
    /// gcluster is 0.
    ///
    /// Note that erasing the `NcPlane` does not reset the base cell.
    ///
    /// *C style function: [ncplane_set_base()][c_api::ncplane_set_base].*
    // call stack:
    // - ncplane_set_base calls nccell_prime:
    //      return nccell_prime(ncp, &ncp->basecell, egc, stylemask, channels);
    // - nccell_prime calls notcurses.c/nccell_load:
    //      return nccell_load(n, c, gcluster);
    // - cell-load calls internal.h/pool load:
    //      return pool_load(&n->pool, c, gcluster);
    pub fn set_base(
        &mut self,
        egc: &str,
        stylemask: NcStyle,
        channels: NcChannels,
    ) -> NcResult<u32> {
        let res = unsafe { c_api::ncplane_set_base(self, cstring![egc], stylemask, channels) };
        error![
            res,
            &format!(
                "NcPlane.set_base({:?}, {:0X}, {:0X})",
                egc, stylemask, channels
            ),
            res as u32
        ]
    }

    /// Sets this `NcPlane`'s base [`NcCell`].
    ///
    /// It will be used for purposes of rendering anywhere that the `NcPlane`'s
    /// gcluster is 0.
    ///
    /// Note that erasing the `NcPlane` does not reset the base cell.
    ///
    /// *C style function: [ncplane_set_base_cell()][c_api::ncplane_set_base_cell].*
    pub fn set_base_cell(&mut self, cell: &NcCell) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_set_base_cell(self, cell) },
            &format!("NcPlane.base({:?})", cell)
        ]
    }

    /// Creates a flat string from the `EGC`'s of the selected region of the
    /// `NcPlane`.
    ///
    /// Starts at the plane's `beg_y` * `beg_x` coordinates (which must lie on
    /// the plane), continuing for `len_y` x `len_x` cells.
    ///
    /// Use `None` for either or all of `beg_y` and `beg_x` in order to
    /// use the current cursor position along that axis.
    ///
    /// Use `None` for either or both of `len_y` and `len_x` in order to
    /// go through the boundary of the plane in that axis (same as `0`).
    ///
    /// *C style function: [ncplane_contents()][c_api::ncplane_contents].*
    pub fn contents(
        &mut self,
        beg_y: Option<NcDim>,
        beg_x: Option<NcDim>,
        len_y: Option<NcDim>,
        len_x: Option<NcDim>,
    ) -> String {
        rstring_free![c_api::ncplane_contents(
            self,
            beg_y.unwrap_or(NcDim::MAX) as i32,
            beg_x.unwrap_or(NcDim::MAX) as i32,
            len_y.unwrap_or(0),
            len_x.unwrap_or(0)
        )]
    }

    /// Erases every [`NcCell`] in this `NcPlane`, resetting all attributes to
    /// normal, all colors to the default color, and all cells to undrawn.
    ///
    /// All cells associated with this `NcPlane` are invalidated, and must not
    /// be used after the call, excluding the base cell. The cursor is homed.
    ///
    /// *C style function: [ncplane_erase()][c_api::ncplane_erase].*
    pub fn erase(&mut self) {
        unsafe {
            c_api::ncplane_erase(self);
        }
    }

    /// Erases every cell in the region beginning at (`beg_y`, `beg_x`) and
    /// having a size (`len_y` × `len_x`) for non-zero lengths.
    ///
    /// If `beg_y` and/or `beg_x` are `None`, the current cursor position
    /// along that axis is used.
    ///
    /// A negative `len_` means to move up from the origin, and a negative
    /// `len_x` means to move left from the origin. A positive `len_y` moves down,
    /// and a positive `len_x` moves right.
    ///
    /// A value of `0` for the length erases everything along that dimension.
    ///
    /// It is an error if the starting coordinate is not in the plane,
    /// but the ending coordinate may be outside the plane.
    ///
    /// ```ignore
    /// // For example, on a plane of 20 rows and 10 columns, with the cursor at
    /// // row 10 and column 5, the following would hold:
    ///
    /// (None, None, 0, 1) // clears the column to the right of the cursor (col 6)
    /// (None, None, 0, -1) // clears the column to the left of the cursor (col 4)
    /// (None, None, i32::MAX, 0) // clears all rows with or below the cursor (rows 10..19)
    /// (None, None, i32::MIN, 0) // clears all rows with or above the cursor (rows 0..10)
    /// (None, 4, 3, 3) // clears from [row 5, col 4] through [row 7, col 6]
    /// (None, 4, -3, -3) // clears from [row 5, col 4] through [row 3, col 2]
    /// (4, None, 0, 3) // clears columns 5, 6, and 7
    /// (None, None, 0, 0) // clears the plane *if the cursor is in a legal position*
    /// (0, 0, 0, 0) // clears the plane in all cases
    /// ```
    /// See also the [`erase_region` example][0].
    ///
    /// [0]: https://github.com/dankamongmen/libnotcurses-sys/blob/main/examples/erase_region.rs
    ///
    /// *C style function: [ncplane_erase_region()][c_api::ncplane_erase_region].*
    pub fn erase_region(
        &mut self,
        beg_y: Option<NcDim>,
        beg_x: Option<NcDim>,
        len_y: NcOffset,
        len_x: NcOffset,
    ) -> NcResult<()> {
        error![
            unsafe {
                c_api::ncplane_erase_region(
                    self,
                    beg_y.unwrap_or(u32::MAX) as i32, // unwrap_or(-1)
                    beg_x.unwrap_or(u32::MAX) as i32, // unwrap_or(-1)
                    len_y,
                    len_x,
                )
            },
            &format!(
                "NcPlane.erase_region({:?}, {:?}, {}, {})",
                beg_y, beg_x, len_y, len_x
            )
        ]
    }

    /// Replaces the `NcCell` at the **specified** coordinates with the provided
    /// `NcCell`, advancing the cursor by its width (but not past the end of
    /// the plane).
    ///
    /// The new `NcCell` must already be associated with this `NcPlane`.
    ///
    /// On success, returns the number of columns the cursor was advanced.
    ///
    /// If the glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_putc_yx()][c_api::ncplane_putc_yx].*
    pub fn putc_yx(&mut self, y: NcDim, x: NcDim, cell: &NcCell) -> NcResult<NcDim> {
        let res = unsafe { c_api::ncplane_putc_yx(self, y as i32, x as i32, cell) };
        error![
            res,
            &format!("NcPlane.putc_yx({}, {}, {:?})", y, x, cell),
            res as NcDim
        ]
    }

    /// Replaces the [`NcCell`] at the **current** coordinates with the provided
    /// `NcCell`, advancing the cursor by its width (but not past the end of
    /// the plane).
    ///
    /// The new `NcCell` must already be associated with the `NcPlane`.
    ///
    /// On success, returns the number of columns the cursor was advanced.
    ///
    /// If the glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_putc()][c_api::ncplane_putc].*
    pub fn putc(&mut self, cell: &NcCell) -> NcResult<NcDim> {
        let res = c_api::ncplane_putc(self, cell);
        error![res, &format!("NcPlane.putc({:?})", cell), res as NcDim]
    }

    /// Calls [`putchar_yx`][NcPlane#method.putchar_yx] at the current cursor
    /// location.
    ///
    /// On success, returns the number of columns the cursor was advanced.
    ///
    /// If the glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// NOTE: unlike the original C function, this one accepts any 4-byte `char`.
    ///
    /// *C style function: [ncplane_putchar()][c_api::ncplane_putchar].*
    pub fn putchar(&mut self, ch: char) -> NcResult<NcDim> {
        let res = c_api::ncplane_putchar(self, ch);
        error![res, &format!("NcPlane.putchar({:?})", ch), res as NcDim]
    }

    /// Replaces the [`NcCell`] at the current location with the provided `char`,
    /// while retaining the previous style.
    ///
    /// On success, returns the number of columns the cursor was advanced.
    ///
    /// If the glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// NOTE: unlike the original C function, this one accepts any 4-byte `char`.
    ///
    /// *C style function: [ncplane_putchar_stained()][c_api::ncplane_putchar_stained].*
    // WIP
    pub fn putchar_stained(&mut self, ch: char) -> NcResult<NcDim> {
        let res = c_api::ncplane_putchar_stained(self, ch);
        error![
            res,
            &format!("NcPlane.putchar_stained({:?})", ch),
            res as NcDim
        ]
    }

    /// Replaces the [`NcCell`] at the specified coordinates with the provided
    /// [`char`], using the current style.
    ///
    /// On success, returns the number of columns the cursor was advanced.
    ///
    /// If the glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// NOTE: unlike the original C function, this one accepts any 4-byte `char`.
    ///
    /// *C style function: [ncplane_putchar_yx()][c_api::ncplane_putchar_yx].*
    pub fn putchar_yx(&mut self, y: NcDim, x: NcDim, ch: char) -> NcResult<NcDim> {
        let res = c_api::ncplane_putchar_yx(self, y, x, ch);
        error![
            res,
            &format!("NcPlane.putchar_yx({}, {}, {:?})", y, x, ch),
            res as NcDim
        ]
    }

    /// Replaces the [`NcCell`] at the current location with the provided `egc`,
    /// using the current style.
    ///
    /// Advances the cursor by the width of the cluster (but not past the end of
    /// the the plane), and this number is returned on success.
    ///
    /// The number of bytes converted from the `egc` can be optionally written
    /// to `sbytes`.
    ///
    /// If the glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// NOTE: unlike the original C function, this one accepts any 4-byte `char`.
    ///
    /// *C style function: [ncplane_putegc()][c_api::ncplane_putegc].*
    pub fn putegc(&mut self, egc: &str, sbytes: Option<&mut usize>) -> NcResult<NcDim> {
        let res = c_api::ncplane_putegc(self, egc, sbytes);
        error![res, &format!("NcPlane.putegc({:?}, …)", egc), res as NcDim]
    }

    /// Replaces the [`NcCell`] at the specified coordinates with the provided
    /// `egc`, using the current style.
    ///
    /// Advances the cursor by the width of the cluster (but not past the end of
    /// the the plane), and this number is returned on success.
    ///
    /// The number of bytes converted from the `egc` can be optionally written
    /// to `sbytes`.
    ///
    /// If the glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// NOTE: unlike the original C function, this one accepts any 4-byte `char`.
    ///
    /// *C style function: [ncplane_putegc_yx()][c_api::ncplane_putegc_yx].*
    pub fn putegc_yx(
        &mut self,
        y: Option<NcDim>,
        x: Option<NcDim>,
        egc: &str,
        sbytes: Option<&mut usize>,
    ) -> NcResult<NcDim> {
        let res = c_api::ncplane_putegc_yx(self, y, x, egc, sbytes);
        error![
            res,
            &format!("NcPlane.putegc_yx({:?}, {:?}, {:?}, …)", y, x, egc),
            res as NcDim
        ]
    }

    /// Replaces the [`NcCell`] at the current location with the provided `egc`,
    /// while retaining the previous style.
    ///
    /// Advances the cursor by the width of the cluster (but not past the end of
    /// the the plane), and this number is returned on success.
    ///
    /// The number of bytes converted from the `egc` can be optionally written
    /// to `sbytes`.
    ///
    /// If the glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// NOTE: unlike the original C function, this one accepts any 4-byte `char`.
    ///
    /// *C style function: [ncplane_putegc_stained()][c_api::ncplane_putegc_stained].*
    pub fn putegc_stained(&mut self, egc: &str, sbytes: Option<&mut usize>) -> NcResult<NcDim> {
        let res = c_api::ncplane_putegc_stained(self, egc, sbytes);
        error![
            res,
            &format!("NcPlane.putegc_stained({:?}, …)", egc),
            res as NcDim
        ]
    }

    /// Write the specified text to the plane, breaking lines sensibly,
    /// beginning at the specified line.
    ///
    /// Returns the number of columns written, including the cleared columns.
    ///
    /// When breaking a line, the line will be cleared to the end of the plane
    /// (the last line will *not* be so cleared).
    //
    // MAYBE:
    // The number of bytes written from the input is written to '*bytes'
    // if it is not NULL.
    //
    // Cleared columns are included in the return value, but *not* included in
    // the number of bytes written.
    //
    /// Leaves the cursor at the end of output. A partial write will be
    /// accomplished as far as it can;
    //
    // determine whether the write completed by inspecting '*bytes'.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_puttext()][c_api::ncplane_puttext].*
    pub fn puttext(&mut self, y: NcDim, align: NcAlign, string: &str) -> NcResult<NcDim> {
        let res =
            unsafe { c_api::ncplane_puttext(self, y as i32, align, cstring![string], null_mut()) };
        error![res, &format!("NcPlane.puttext({:?})", string), res as NcDim]
    }

    /// Writes a string to the current location, using the current style.
    ///
    /// Advances the cursor by some positive number of columns (though not
    /// beyond the end of the plane), and this number is returned on success.
    ///
    /// On error, a non-positive number is returned, indicating
    /// the number of columns which were written before the error.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_putstr()][c_api::ncplane_putstr].*
    #[inline]
    pub fn putstr(&mut self, string: &str) -> NcResult<NcDim> {
        let res = c_api::ncplane_putstr(self, string);
        error![res, &format!("NcPlane.putstr({:?})", string), res as NcDim]
    }

    /// Same as [`putstr`][NcPlane#method.putstr], but it also puts a newline
    /// character at the end.
    ///
    /// This will only work if scrolling is enabled in the plane.
    ///
    /// *(No equivalent C style function)*
    pub fn putstrln(&mut self, string: &str) -> NcResult<NcDim> {
        let mut cols = self.putstr(string)?;
        cols += self.putstr("\n")?;
        Ok(cols)
    }

    /// Prints a new line character.
    ///
    /// This will only work if scrolling is enabled in the plane.
    ///
    /// *(No equivalent C style function)*
    pub fn putln(&mut self) -> NcResult<NcDim> {
        let cols = self.putstr("\n")?;
        Ok(cols)
    }

    /// Writes a string to the current location, retaining the previous style.
    ///
    /// Advances the cursor by some positive number of columns (though not
    /// beyond the end of the plane); this number is returned on success.
    ///
    /// On error, a non-positive number is returned, indicating the number of
    /// columns which were written before the error.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_putstr_stained()][c_api::ncplane_putstr_stained].*
    pub fn putstr_stained(&mut self, string: &str) -> NcResult<NcDim> {
        let res = c_api::ncplane_putstr_stained(self, string);
        error![
            res,
            &format!("NcPlane.putstr_stained({:?})", string),
            res as NcDim
        ]
    }

    /// Writes a string to the provided location, using the current style
    /// and [`NcAlign`]ed on *x*.
    ///
    /// Advances the cursor by some positive number of columns (though not
    /// beyond the end of the plane); this number is returned on success.
    ///
    /// On error, a non-positive number is returned, indicating the number of
    /// columns which were written before the error.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_putstr_aligned()][c_api::ncplane_putstr_aligned].*
    pub fn putstr_aligned(
        &mut self,
        y: Option<NcDim>,
        align: NcAlign,
        string: &str,
    ) -> NcResult<NcDim> {
        let res = c_api::ncplane_putstr_aligned(self, y, align, string);
        error![
            res,
            &format!("NcPlane.putstr_aligned({:?}, {}, {:?})", y, align, string),
            res as NcDim
        ]
    }

    /// Writes a string to the provided location, using the current style.
    ///
    /// Advances the cursor by some positive number of columns (though not
    /// beyond the end of the plane); this number is returned on success.
    ///
    /// On error, a non-positive number is returned, indicating the number of
    /// columns which were written before the error.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_putstr_yx()][c_api::ncplane_putstr_yx].*
    pub fn putstr_yx(
        &mut self,
        y: Option<NcDim>,
        x: Option<NcDim>,
        string: &str,
    ) -> NcResult<NcDim> {
        let res = c_api::ncplane_putstr_yx(self, y, x, string);
        error![
            res,
            &format!("NcPlane.putstr_yx({:?}, {:?}, {:?})", y, x, string),
            res as NcDim
        ]
    }

    /// Writes a string to the provided location, [`NcAlign`]ed on *x*
    /// and retaining the previous style.
    ///
    /// Advances the cursor by some positive number of columns (though not
    /// beyond the end of the plane); this number is returned on success.
    ///
    /// On error, a non-positive number is returned, indicating the number of
    /// columns which were written before the error.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *(No equivalent C style function)*
    pub fn putstr_aligned_stained(
        &mut self,
        y: NcDim,
        align: NcAlign,
        string: &str,
    ) -> NcResult<NcDim> {
        let width = string.chars().count() as u32;
        let xpos = self.halign(align, width)?;
        self.cursor_move_yx(y, xpos)?;
        let res = c_api::ncplane_putstr_stained(self, string);
        error![
            res,
            &format!(
                "NcPlane.putstr_aligned_stained({}, {}, {:?})",
                y, align, string
            ),
            res as NcDim
        ]
    }

    /// Writes a string to the provided location, while retaining the previous
    /// style.
    ///
    /// Advances the cursor by some positive number of columns (though not
    /// beyond the end of the plane); this number is returned on success.
    ///
    /// On error, a non-positive number is returned, indicating the number of
    /// columns which were written before the error.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *(No equivalent C style function)*
    pub fn putstr_yx_stained(&mut self, y: NcDim, x: NcDim, string: &str) -> NcResult<NcDim> {
        self.cursor_move_yx(y, x)?;
        let res = c_api::ncplane_putstr_stained(self, string);
        error![
            res,
            &format!("NcPlane.putstr_yx_stained({}, {}, {:?})", y, x, string),
            res as NcDim
        ]
    }

    /// Writes a string to the current location, using the current style,
    /// and no more than `num_bytes` bytes will be written.
    ///
    /// Advances the cursor by some positive number of columns (though not
    /// beyond the end of the plane), and this number is returned on success.
    ///
    /// On error, a non-positive number is returned, indicating the number of
    /// columns which were written before the error.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_putnstr()][c_api::ncplane_putnstr].*
    #[inline]
    pub fn putnstr(&mut self, num_bytes: usize, string: &str) -> NcResult<NcDim> {
        let res = c_api::ncplane_putnstr(self, num_bytes, string);
        error![
            res,
            &format!("NcPlane.puntstr({}, {:?})", num_bytes, string),
            res as NcDim
        ]
    }

    /// Writes a string to the provided location, using the current style,
    /// [`NcAlign`]ed on *x*, and no more than `num_bytes` bytes will be written.
    ///
    /// Advances the cursor by some positive number of columns (though not
    /// beyond the end of the plane), and this number is returned on success.
    ///
    /// On error, a non-positive number is returned, indicating the number of
    /// columns which were written before the error.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_putnstr_aligned()][c_api::ncplane_putnstr_aligned].*
    pub fn putnstr_aligned(
        &mut self,
        y: NcDim,
        align: NcAlign,
        num_bytes: usize,
        string: &str,
    ) -> NcResult<NcDim> {
        let res = unsafe {
            c_api::ncplane_putnstr_aligned(self, y as i32, align, num_bytes, cstring![string])
        };
        error![
            res,
            &format!(
                "NcPlane.putnstr_aligned({}, {}, {}, {:?})",
                y, align, num_bytes, string
            ),
            res as NcDim
        ]
    }

    /// Writes a string to the provided location, using the current style,
    /// and no more than `num_bytes` bytes will be written.
    ///
    /// Advances the cursor by some positive number of columns (though not
    /// beyond the end of the plane); this number is returned on success.
    ///
    /// On error, a non-positive number is returned, indicating the number of
    /// columns which were written before the error.
    ///
    /// If a glyph can not fit in the current line, it is an error, unless
    /// scrolling is enabled.
    ///
    /// *C style function: [ncplane_putnstr_yx()][c_api::ncplane_putnstr_yx].*
    pub fn putnstr_yx(
        &mut self,
        y: Option<NcDim>,
        x: Option<NcDim>,
        num_bytes: usize,
        string: &str,
    ) -> NcResult<NcDim> {
        let res = c_api::ncplane_putnstr_yx(self, y, x, num_bytes, string);
        error![
            res,
            &format!(
                "NcPlane.putnstr_yx({:?}, {:?}, {}, {:?})",
                y, x, num_bytes, string
            ),
            res as NcDim
        ]
    }

    /// Considers the glyph at `y`,`x` coordinates as the fill target,
    /// and copies `cell` to it and to all cardinally-connect cells.
    ///
    /// Returns the number of cells polyfilled.
    ///
    /// An invalid initial `y`, `x` is an error.
    ///
    /// *C style function: [ncplane_putnstr_yx()][c_api::ncplane_putnstr_yx].*
    pub fn polyfill_yx(&mut self, y: NcDim, x: NcDim, cell: &NcCell) -> NcResult<usize> {
        let res = unsafe { c_api::ncplane_polyfill_yx(self, y as i32, x as i32, cell) };
        error![
            res,
            &format!("NcPlane.polyfill_yx({}, {}, {:?})", y, x, cell),
            res as usize
        ]
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: `NcPlane` & `Nc`
impl NcPlane {
    /// Gets the origin of this plane relative to its pile.
    ///
    /// *C style function: [ncplane_abs_yx()][c_api::ncplane_abs_yx].*
    pub fn abs_yx(&self) -> (NcDim, NcDim) {
        let mut y = 0;
        let mut x = 0;
        unsafe {
            c_api::ncplane_abs_yx(self, &mut y, &mut x);
        }
        (y as NcDim, x as NcDim)
    }

    /// Gets the origin of this plane relative to its pile, in the y axis.
    ///
    /// *C style function: [ncplane_abs_y()][c_api::ncplane_abs_y].*
    pub fn abs_y(&self) -> NcDim {
        unsafe { c_api::ncplane_abs_y(self) as NcDim }
    }

    /// Gets the origin of this plane relative to its pile, in the x axis.
    ///
    /// *C style function: [ncplane_abs_x()][c_api::ncplane_abs_x].*
    pub fn abs_x(&self) -> NcDim {
        unsafe { c_api::ncplane_abs_x(self) as NcDim }
    }

    /// Duplicates this `NcPlane`.
    ///
    /// The new NcPlane will have the same geometry, the same rendering state,
    /// and all the same duplicated content.
    ///
    /// The new plane will be immediately above the old one on the z axis,
    /// and will be bound to the same parent. Bound planes are not duplicated;
    /// the new plane is bound to the current parent, but has no bound planes.
    ///
    /// *C style function: [ncplane_dup()][c_api::ncplane_dup].*
    //
    // TODO: deal with the opaque field that is stored in NcPlaneOptions.userptr
    pub fn dup(&mut self) -> &mut NcPlane {
        unsafe { &mut *c_api::ncplane_dup(self, null_mut()) }
    }

    /// Returns the topmost `NcPlane` of the current pile.
    ///
    /// *C style function: [ncpile_top()][c_api::ncpile_top].*
    pub fn top(&mut self) -> &mut NcPlane {
        unsafe { &mut *c_api::ncpile_top(self) }
    }

    /// Returns the bottommost `NcPlane` of the current pile.
    ///
    /// *C style function: [ncpile_bottom()][c_api::ncpile_bottom].*
    pub fn bottom<'a>(&mut self) -> &'a mut NcPlane {
        unsafe { &mut *c_api::ncpile_bottom(self) }
    }

    /// Relocates this `NcPlane` at the bottom of the z-buffer.
    ///
    /// *C style function: [ncplane_move_bottom()][c_api::ncplane_move_bottom].*
    pub fn move_bottom(&mut self) {
        c_api::ncplane_move_bottom(self);
    }

    /// Relocates this `NcPlane` at the top of the z-buffer.
    ///
    /// *C style function: [ncplane_move_top()][c_api::ncplane_move_top].*
    pub fn move_top(&mut self) {
        c_api::ncplane_move_top(self);
    }

    /// Moves this `NcPlane` relative to the standard plane, or the plane to
    /// which it is bound.
    ///
    /// It is an error to attempt to move the standard plane.
    ///
    /// *C style function: [ncplane_move_yx()][c_api::ncplane_move_yx].*
    pub fn move_yx(&mut self, y: NcOffset, x: NcOffset) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_move_yx(self, y, x) },
            &format!("NcPlane.move_yx({}, {})", y, x)
        ]
    }

    /// Moves this `NcPlane` relative to its current location.
    ///
    /// Negative values move up and left, respectively.
    /// Pass 0 to hold an axis constant.
    ///
    /// It is an error to attempt to move the standard plane.
    ///
    /// *C style function: [ncplane_moverel()][c_api::ncplane_moverel].*
    pub fn move_rel(&mut self, rows: NcOffset, cols: NcOffset) -> NcResult<()> {
        error![
            c_api::ncplane_moverel(self, rows, cols),
            &format!("NcPlane.move_rel({}, {})", rows, cols)
        ]
    }

    /// Returns the `NcPlane` above this one, or None if already at the top.
    ///
    /// *C style function: [ncplane_above()][c_api::ncplane_above].*
    pub fn above(&mut self) -> Option<&mut NcPlane> {
        let ptr = unsafe { c_api::ncplane_above(self) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *ptr })
        }
    }

    /// Returns the `NcPlane` below this one, or None if already at the bottom.
    ///
    /// *C style function: [ncplane_below()][c_api::ncplane_below].*
    pub fn below(&mut self) -> Option<&mut NcPlane> {
        let ptr = unsafe { c_api::ncplane_below(self) };
        if ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *ptr })
        }
    }

    /// Relocates this `NcPlane` above the `above` NcPlane, in the z-buffer.
    ///
    /// Returns [`NcIntResult::ERR`][NcIntResult#associatedconstant.ERR] if
    /// the current plane is already in the desired location.
    /// Both planes must not be the same.
    ///
    /// *C style function: [ncplane_move_above()][c_api::ncplane_move_above].*
    pub fn move_above(&mut self, above: &mut NcPlane) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_move_above(self, above) },
            "NcPlane.move_above()"
        ]
    }

    /// Relocates this `NcPlane` below the `below` NcPlane, in the z-buffer.
    ///
    /// Returns [`NcIntResult::ERR`][NcIntResult#associatedconstant.ERR] if
    /// the current plane is already in the desired location.
    /// Both planes must not be the same.
    ///
    /// *C style function: [ncplane_move_below()][c_api::ncplane_move_below].*
    pub fn move_below(&mut self, below: &mut NcPlane) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_move_below(self, below) },
            "NcPlane.move_below()"
        ]
    }

    /// Splices this plane and its bound planes out of the z-buffer,
    /// and reinserts them at the bottom.
    ///
    /// Relative order will be maintained between the reinserted planes.
    ///
    /// For a plane E bound to C, with z-ordering A B C D E, moving the C family
    /// to the bottom results in A B D C E.
    ///
    /// *C style function: [ncplane_move_family_bottom()][c_api::ncplane_move_family_bottom].*
    pub fn move_family_bottom(&mut self) {
        c_api::ncplane_move_family_bottom(self)
    }

    /// Splices this plane and its bound planes out of the z-buffer,
    /// and reinserts them at the top.
    ///
    /// Relative order will be maintained between the reinserted planes.
    ///
    /// For a plane E bound to C, with z-ordering A B C D E, moving the C family
    /// to the top results in C E A B D.
    ///
    /// *C style function: [ncplane_move_family_top()][c_api::ncplane_move_family_top].*
    pub fn move_family_top(&mut self) {
        c_api::ncplane_move_family_top(self)
    }

    /// Splices this plane and its bound planes out of the z-buffer,
    /// and reinserts them above the `above` plane.
    ///
    /// Relative order will be maintained between the reinserted planes.
    ///
    /// For a plane E bound to C, with z-ordering A B C D E, moving the C family
    /// to the top results in C E A B D.
    ///
    /// *C style function: [ncplane_move_family_below()][c_api::ncplane_move_family_below].*
    pub fn move_family_above(&mut self, above: &mut NcPlane) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_move_family_above(self, above) },
            "NcPlane.move_family_above()"
        ]
    }

    /// Splices this plane and its bound planes out of the z-buffer,
    /// and reinserts them below the `below` plane.
    ///
    /// Relative order will be maintained between the reinserted planes.
    ///
    /// For a plane E bound to C, with z-ordering A B C D E, moving the C family
    /// to the bottom results in A B D C E.
    ///
    /// *C style function: [ncplane_move_family_below()][c_api::ncplane_move_family_below].*
    pub fn move_family_below(&mut self, below: &mut NcPlane) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_move_family_below(self, below) },
            "NcPlane.move_family_below()"
        ]
    }

    /// Merges the `NcPlane` `source` down onto the current `NcPlane` (`self`).
    ///
    /// This is most rigorously defined as "write to `self` the frame that would
    /// be rendered were the entire stack made up only of the specified subregion
    /// of `source` and, below it, the subregion of `self` having the specified
    /// origin.
    ///
    /// Use `None` for either or all of `beg_src_y` and `beg_src_x` in order to
    /// use the current cursor position along that axis.
    ///
    /// Use `None` for either or both of `len_y` and `len_x` in order to
    /// go through the boundary of the plane in that axis (same as `0`).
    ///
    /// Merging is independent of the position of both planes on the z-axis.
    ///
    /// It is an error to define a subregion that is not entirely contained
    /// within `source`.
    ///
    /// It is an error to define a target origin such that the projected
    /// subregion is not entirely contained within `self`.
    ///
    /// Behavior is undefined if both planes are equivalent.
    ///
    /// `self` is modified, but `source` remains unchanged.
    ///
    /// Neither `source` nor `self` may have sprixels.
    ///
    /// *C style function: [ncplane_mergedown()][c_api::ncplane_mergedown].*
    pub fn mergedown(
        &mut self,
        source: &mut NcPlane,
        beg_src_y: Option<NcDim>,
        beg_src_x: Option<NcDim>,
        len_y: Option<NcDim>,
        len_x: Option<NcDim>,
        dst_y: Option<NcDim>,
        dst_x: Option<NcDim>,
    ) -> NcResult<()> {
        error![
            unsafe {
                c_api::ncplane_mergedown(
                    source,
                    self,
                    beg_src_y.unwrap_or(NcDim::MAX) as i32,
                    beg_src_x.unwrap_or(NcDim::MAX) as i32,
                    len_y.unwrap_or(0),
                    len_x.unwrap_or(0),
                    dst_y.unwrap_or(NcDim::MAX) as i32,
                    dst_x.unwrap_or(NcDim::MAX) as i32,
                )
            },
            &format!(
                "NcPlane.mergedown(NcPlane, {:?}, {:?}, {:?}, {:?}, {:?}, {:?})",
                beg_src_y, beg_src_x, len_y, len_x, dst_y, dst_x
            )
        ]
    }

    /// Merges `source` down onto this `NcPlane`.
    ///
    /// If `source` does not intersect, this plane will not be changed,
    /// but it is not an error.
    ///
    /// See [`mergedown`][NcPlane#method.mergedown]
    /// for more information.
    ///
    /// *C style function: [ncplane_mergedown_simple()][c_api::ncplane_mergedown_simple].*
    //
    // TODO: maybe create a reversed method, and/or an associated function,
    // for `mergedown` too.
    pub fn mergedown_simple(&mut self, source: &mut NcPlane) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_mergedown_simple(source, self) },
            "NcPlane.mergedown_simple(NcPlane)"
        ]
    }

    /// Gets the parent to which this `NcPlane` is bound, if any.
    ///
    /// *C style function: [ncplane_parent()][c_api::ncplane_parent].*
    //
    // TODO: CHECK: what happens when it's bound to itself.
    pub fn parent(&mut self) -> NcResult<&mut NcPlane> {
        error_ref_mut![unsafe { c_api::ncplane_parent(self) }, "NcPlane.parent()"]
    }

    /// Gets the parent to which this `NcPlane` is bound, if any.
    ///
    /// *C style function: [ncplane_parent_const()][c_api::ncplane_parent_const].*
    //
    // CHECK: what happens when it's bound to itself.
    pub fn parent_const(&self) -> NcResult<&NcPlane> {
        error_ref![
            unsafe { c_api::ncplane_parent_const(self) },
            "NcPlane.parent_const()"
        ]
    }

    /// Unbounds this `NcPlane` from its parent, makes it a bound child of
    /// 'newparent', and returns itself.
    ///
    /// Any planes bound to this `NcPlane` are reparented to the previous parent.
    ///
    /// If this `NcPlane` is equal to `newparent`, then becomes the root of a new
    /// pile, unless it is already the root of a pile, in which case this is a
    /// no-op.
    ///
    /// The standard plane cannot be reparented.
    ///
    /// *C style function: [ncplane_reparent()][c_api::ncplane_reparent].*
    pub fn reparent<'a>(&mut self, newparent: &'a mut NcPlane) -> NcResult<&'a mut NcPlane> {
        error_ref_mut![
            unsafe { c_api::ncplane_reparent(self, newparent) },
            "NcPlane.reparent(NcPlane)"
        ]
    }

    /// Like [`reparent`][NcPlane#method.reparent], except any bound
    /// planes comes along with this `NcPlane` to its new destination.
    ///
    /// Their z-order is maintained.
    ///
    /// *C style function: [ncplane_reparent_family()][c_api::ncplane_reparent_family].*
    //
    // TODO:CHECK: If 'newparent' is an ancestor, NULL is returned & no changes're made.
    pub fn reparent_family<'a>(&mut self, newparent: &'a mut NcPlane) -> NcResult<&'a mut NcPlane> {
        error_ref_mut![
            unsafe { c_api::ncplane_reparent_family(self, newparent) },
            "NcPlane.reparent_family(NcPlane)"
        ]
    }

    /// Makes the physical screen match the last rendered frame from the pile of
    /// which this `NcPlane` is a part.
    ///
    /// This is a blocking call. Don't call this before the pile has been
    /// rendered (doing so will likely result in a blank screen).
    ///
    /// *C style function: [ncpile_rasterize()][c_api::ncpile_rasterize].*
    pub fn rasterize(&mut self) -> NcResult<()> {
        error![
            unsafe { c_api::ncpile_rasterize(self) },
            "NcPlane.rasterize()"
        ]
    }

    /// Renders the pile of which this `NcPlane` is a part.
    /// Rendering this pile again will blow away the render.
    /// To actually write out the render, call ncpile_rasterize().
    ///
    /// *C style function: [ncpile_render()][c_api::ncpile_render].*
    pub fn render(&mut self) -> NcResult<()> {
        error![unsafe { c_api::ncpile_render(self) }, "NcPlane.render()"]
    }

    /// Renders and rasterizes the pile of which this `NcPlane` is a part.
    ///
    /// *(No equivalent C style function)*
    pub fn render2(&mut self) -> NcResult<()> {
        self.render()?;
        self.rasterize()?;
        Ok(())
    }

    /// Performs the rendering and rasterization portion of
    /// [`render`][NcPlane#method.render] and [`rasterize`][NcPlane#method.rasterize]
    /// but doe not write the resulting buffer out to the terminal.
    ///
    /// Using this function, the user can control the writeout process.
    /// The returned buffer must be freed by the caller.
    ///
    /// *C style function: [ncpile_render_to_buffer()][c_api::ncpile_render_to_buffer].*
    // CHECK this works
    pub fn render_to_buffer(&mut self, buffer: &mut Vec<u8>) -> NcResult<()> {
        let len = buffer.len() as u32;

        // https://github.com/dankamongmen/notcurses/issues/1339
        #[cfg(any(target_arch = "x86_64", target_arch = "i686", target_arch = "x86"))]
        let mut buf = buffer.as_mut_ptr() as *mut i8;
        #[cfg(not(any(target_arch = "x86_64", target_arch = "i686", target_arch = "x86")))]
        let mut buf = buffer.as_mut_ptr() as *mut u8;

        error![
            unsafe { c_api::ncpile_render_to_buffer(self, &mut buf, &mut len.try_into().unwrap()) },
            &format!["NcPlane.render_to_buffer(buffer, {})", len]
        ]
    }

    /// Writes the last rendered frame, in its entirety, to `fp`.
    ///
    /// If a frame has not yet been rendered, nothing will be written.
    ///
    /// *C style function: [ncpile_render_to_file()][c_api::ncpile_render_to_file].*
    pub fn render_to_file(&mut self, fp: &mut NcFile) -> NcResult<()> {
        error![unsafe { c_api::ncpile_render_to_file(self, fp.as_nc_ptr()) }]
    }
    /// Gets a mutable reference to the [`Nc`] context of this `NcPlane`.
    ///
    /// *C style function: [ncplane_notcurses()][c_api::ncplane_notcurses].*
    pub fn notcurses<'a>(&self) -> NcResult<&'a mut Nc> {
        error_ref_mut![
            unsafe { c_api::ncplane_notcurses(self) },
            "NcPlane.notcurses()"
        ]
    }

    /// Gets an immutable reference to the [`Nc`] context of this `NcPlane`.
    ///
    /// *C style function: [ncplane_notcurses_const()][c_api::ncplane_notcurses_const].*
    pub fn notcurses_const<'a>(&self) -> NcResult<&'a Nc> {
        error_ref![
            unsafe { c_api::ncplane_notcurses_const(self) },
            "NcPlane.notcurses()"
        ]
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: cursor
impl NcPlane {
    /// Moves the cursor to 0, 0.
    ///
    /// *C style function: [ncplane_home()][c_api::ncplane_home].*
    pub fn cursor_home(&mut self) {
        unsafe {
            c_api::ncplane_home(self);
        }
    }

    /// Returns the current position of the cursor within this `NcPlane`.
    ///
    /// *C style function: [ncplane_cursor_yx()][c_api::ncplane_cursor_yx].*
    //
    // NOTE: y and/or x may be NULL.
    // check for null and return NcResult
    pub fn cursor_yx(&self) -> (NcDim, NcDim) {
        let (mut y, mut x) = (0, 0);
        unsafe { c_api::ncplane_cursor_yx(self, &mut y, &mut x) };
        (y as NcDim, x as NcDim)
    }

    /// Returns the current row of the cursor within this `NcPlane`.
    ///
    /// *(No equivalent C style function)*
    pub fn cursor_y(&self) -> NcDim {
        self.cursor_yx().0
    }

    /// Returns the current column of the cursor within this `NcPlane`.
    ///
    /// *(No equivalent C style function)*
    pub fn cursor_x(&self) -> NcDim {
        self.cursor_yx().1
    }

    /// Moves the cursor to the specified position within this `NcPlane`.
    ///
    /// The cursor doesn't need to be visible.
    ///
    /// Parameters exceeding the plane's dimensions will result in an error,
    /// and the cursor position will remain unchanged.
    ///
    /// *C style function: [ncplane_cursor_move_yx()][c_api::ncplane_cursor_move_yx].*
    pub fn cursor_move_yx(&mut self, y: NcDim, x: NcDim) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_cursor_move_yx(self, y as i32, x as i32) },
            &format!("NcPlane.move_yx({}, {})", y, x)
        ]
    }

    /// Moves the cursor to the specified row within this `NcPlane`.
    ///
    /// *(No equivalent C style function)*
    pub fn cursor_move_y(&mut self, y: NcDim) -> NcResult<()> {
        let x = self.cursor_x();
        error![
            unsafe { c_api::ncplane_cursor_move_yx(self, y as i32, x as i32) },
            &format!("NcPlane.move_y({})", y)
        ]
    }

    /// Moves the cursor to the specified column within this `NcPlane`.
    ///
    /// *(No equivalent C style function)*
    pub fn cursor_move_x(&mut self, x: NcDim) -> NcResult<()> {
        let y = self.cursor_y();
        error![
            unsafe { c_api::ncplane_cursor_move_yx(self, y as i32, x as i32) },
            &format!("NcPlane.move_x({})", x)
        ]
    }

    /// Moves the cursor the number of rows specified (forward or backwards).
    ///
    /// It will error if the target row exceeds the plane dimensions.
    ///
    /// *(No equivalent C style function)*
    pub fn cursor_move_rows(&mut self, rows: NcOffset) -> NcResult<()> {
        let (y, x) = self.cursor_yx();
        self.cursor_move_yx((y as NcOffset + rows) as NcDim, x)
    }

    /// Moves the cursor the number of columns specified (forward or backwards).
    ///
    /// It will error if the target column exceeds the plane dimensions.
    ///
    /// *(No equivalent C style function)*
    pub fn cursor_move_cols(&mut self, cols: NcOffset) -> NcResult<()> {
        let (y, x) = self.cursor_yx();
        self.cursor_move_yx(y, (x as NcOffset + cols) as NcDim)
    }

    /// Moves the cursor relatively, the number of rows and columns specified
    /// (forward or backwards).
    ///
    /// It will error if the target row or column exceeds the plane dimensions.
    ///
    /// *C style function: [ncplane_cursor_move_rel()][c_api::ncplane_cursor_move_rel].*
    pub fn cursor_move_rel(&mut self, rows: NcOffset, cols: NcOffset) -> NcResult<()> {
        self.cursor_move_rows(rows)?;
        self.cursor_move_cols(cols)?;
        Ok(())
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: size, position & alignment
impl NcPlane {
    /// Returns the column at which `numcols` columns ought start in order to be
    /// aligned according to `align` within this plane.
    ///
    /// Returns `-`[NcIntResult::MAX][crate::NcIntResult::MAX] if
    /// [NcAlign::UNALIGNED][NcAlign#associatedconstant.UNALIGNED]
    /// or invalid [`NcAlign`].
    ///
    /// *C style function: [ncplane_halign()][c_api::ncplane_halign].*
    #[inline]
    pub fn halign(&mut self, align: NcAlign, numcols: NcDim) -> NcResult<NcDim> {
        let res = c_api::ncplane_halign(self, align, numcols);
        error![
            res,
            &format!("NcPlane.halign({:?}, {})", align, numcols),
            res as NcDim
        ]
    }

    /// Returns the row at which `rows` rows ought start in order to be
    /// aligned according to `align` within this plane.
    ///
    /// Returns `-`[NcIntResult::MAX][crate::NcIntResult::MAX] if
    /// [NcAlign::UNALIGNED][NcAlign#associatedconstant.UNALIGNED]
    /// or invalid [`NcAlign`].
    ///
    /// *C style function: [ncplane_valign()][c_api::ncplane_valign].*
    #[inline]
    pub fn valign(&mut self, align: NcAlign, numrows: NcDim) -> NcResult<()> {
        error![
            c_api::ncplane_valign(self, align, numrows),
            &format!("NcPlane.valign({:?}, {})", align, numrows)
        ]
    }

    /// Finds the center coordinate of a plane.
    ///
    /// In the case of an even number of rows/columns the top/left is preferred
    /// (in such a case, there will be one more cell to the bottom/right
    /// of the center than the top/left).
    /// The center is then modified relative to the plane's origin.
    ///
    /// *C style function: [ncplane_center_abs()][c_api::ncplane_center_abs].*
    pub fn center_abs(&self) -> (NcDim, NcDim) {
        let (mut y, mut x) = (0, 0);
        unsafe {
            c_api::ncplane_center_abs(self, &mut y, &mut x);
        }
        (y as NcDim, x as NcDim)
    }

    /// Returns the dimensions of this `NcPlane`.
    ///
    /// *C style function: [ncplane_dim_yx()][c_api::ncplane_dim_yx].*
    pub fn dim_yx(&self) -> (NcDim, NcDim) {
        let (mut y, mut x) = (0, 0);
        unsafe { c_api::ncplane_dim_yx(self, &mut y, &mut x) };
        (y as NcDim, x as NcDim)
    }

    /// Return the rows of this `NcPlane`.
    ///
    /// *C style function: [ncplane_dim_y()][c_api::ncplane_dim_y].*
    #[inline]
    pub fn dim_y(&self) -> NcDim {
        self.dim_yx().0
    }

    /// Return the columns of this `NcPlane`.
    ///
    /// *C style function: [ncplane_dim_x()][c_api::ncplane_dim_x].*
    #[inline]
    pub fn dim_x(&self) -> NcDim {
        self.dim_yx().1
    }

    /// Return the rows of this `NcPlane`.
    ///
    /// Alias of [dim_y][NcPlane#method.dim_y]
    ///
    /// *C style function: [ncplane_dim_y()][c_api::ncplane_dim_y].*
    #[inline]
    pub fn rows(&self) -> NcDim {
        self.dim_yx().0
    }

    /// Return the cols of this `NcPlane`.
    ///
    /// Alias of [dim_x][NcPlane#method.dim_x]
    ///
    /// *C style function: [ncplane_dim_x()][c_api::ncplane_dim_x].*
    #[inline]
    pub fn cols(&self) -> NcDim {
        self.dim_yx().1
    }

    /// Resizes this `NcPlane`.
    ///
    /// The four parameters `keep_y`, `keep_x`, `keep_len_y`, and `keep_len_x`
    /// defines a subset of this `NcPlane` to keep unchanged. This may be a section
    /// of size 0.
    ///
    /// `keep_x` and `keep_y` are relative to this `NcPlane`. They must specify a
    /// coordinate within the ncplane's totality. If either of `keep_len_y` or
    /// `keep_len_x` is non-zero, both must be non-zero.
    ///
    /// `y_off` and `x_off` are relative to `keep_y` and `keep_x`, and place the
    /// upper-left corner of the resized NcPlane.
    ///
    /// `y_len` and `x_len` are the dimensions of this `NcPlane` after resizing.
    /// `y_len` must be greater than or equal to `keep_len_y`,
    /// and `x_len` must be greater than or equal to `keeplenx`.
    ///
    /// It is an error to attempt to resize the standard plane.
    ///
    /// *C style function: [ncplane_resize()][c_api::ncplane_resize].*
    pub fn resize(
        &mut self,
        keep_y: NcDim,
        keep_x: NcDim,
        keep_len_y: NcDim,
        keep_len_x: NcDim,
        y_off: NcOffset,
        x_off: NcOffset,
        y_len: NcDim,
        x_len: NcDim,
    ) -> NcResult<()> {
        error![
            unsafe {
                c_api::ncplane_resize(
                    self,
                    keep_y as i32,
                    keep_x as i32,
                    keep_len_y,
                    keep_len_x,
                    y_off as i32,
                    x_off as i32,
                    y_len,
                    x_len,
                )
            },
            &format!(
                "NcPlane.resize({}, {}, {}, {}, {}, {}, {}, {})",
                keep_y, keep_x, keep_len_y, keep_len_x, y_off, x_off, y_len, x_len
            )
        ]
    }

    /// Suitable for use as a 'resizecb' with planes created with
    /// [`NcPlaneOptions::MARGINALIZED`][NcPlaneOptions#associatedconstant.MARGINALIZED].
    ///
    /// This will resize this plane against its parent, attempting to enforce
    /// the supplied margins.
    ///
    /// *C style function: [ncplane_resize_marginalized()][c_api::ncplane_resize_marginalized].*
    pub fn resize_marginalized(&mut self) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_resize_marginalized(self) },
            "NcPlane.resize_marginalized()"
        ]
    }

    /// Suitable for use as a 'resizecb', this will resize the plane
    /// to the visual region's size. It is used for the standard plane.
    ///
    /// *C style function: [ncplane_resize_maximize()][c_api::ncplane_resize_maximize].*
    pub fn resize_maximize(&mut self) -> NcResult<()> {
        error![
            unsafe { c_api::ncplane_resize_maximize(self) },
            "NcPlane.resize_maximize()"
        ]
    }

    /// Creates an RGBA flat array from the selected region of the plane.
    ///
    /// Begins at the plane's `beg_y`x`beg_x` coordinate (which must lie on the
    /// plane), continuing for `len_y`x`len_x` cells.
    ///
    /// Use `None` for either or both of `beg_y` and `beg_x` in order to
    /// use the current cursor position along that axis.
    ///
    /// Use `None` for either or both of `len_y` and `len_x` in order to
    /// go through the boundary of the plane in that axis (same as `0`).
    ///
    /// Only glyphs from the specified blitset may be present.
    ///
    /// *C style function: [ncplane_as_rgba()][c_api::ncplane_as_rgba].*
    pub fn as_rgba(
        &mut self,
        blitter: NcBlitter,
        beg_y: Option<NcDim>,
        beg_x: Option<NcDim>,
        len_y: Option<NcDim>,
        len_x: Option<NcDim>,
    ) -> NcResult<&mut [u32]> {
        // pixel geometry
        let mut pxdim_y = 0;
        let mut pxdim_x = 0;

        let res_array = unsafe {
            c_api::ncplane_as_rgba(
                self,
                blitter,
                beg_y.unwrap_or(NcDim::MAX) as i32,
                beg_x.unwrap_or(NcDim::MAX) as i32,
                len_y.unwrap_or(0),
                len_x.unwrap_or(0),
                &mut pxdim_y,
                &mut pxdim_x,
            )
        };

        error_ref_mut![
            res_array,
            &format![
                "NcPlane.rgba({}, {:?}, {:?}, {:?}, {:?})",
                blitter, beg_y, beg_x, len_y, len_x
            ],
            from_raw_parts_mut(res_array, (pxdim_y * pxdim_x) as usize)
        ]
    }

    /// Returns an [NcPixelGeometry] structure filled with pixel geometry for
    /// the display region, each cell, and the maximum displayable bitmap.
    ///
    /// This function calls
    /// [notcurses_check_pixel_support][c_api::notcurses_check_pixel_support],
    /// possibly leading to an interrogation of the terminal.
    ///
    /// *C style function: [ncplane_pixel_geom()][c_api::ncplane_pixel_geom].*
    pub fn pixel_geom(&self) -> NcPixelGeometry {
        let mut pxy = 0;
        let mut pxx = 0;
        let mut celldimy = 0;
        let mut celldimx = 0;
        let mut maxbmapy = 0;
        let mut maxbmapx = 0;
        unsafe {
            c_api::ncplane_pixel_geom(
                self,
                &mut pxy,
                &mut pxx,
                &mut celldimy,
                &mut celldimx,
                &mut maxbmapy,
                &mut maxbmapx,
            );
        }
        NcPixelGeometry {
            term_y: pxy as NcDim,
            term_x: pxx as NcDim,
            cell_y: celldimy as NcDim,
            cell_x: celldimx as NcDim,
            max_bitmap_y: maxbmapy as NcDim,
            max_bitmap_x: maxbmapx as NcDim,
        }
    }

    /// Realigns this `NcPlane` against its parent, using the alignment specified
    /// at creation time.
    ///
    /// Suitable for use as an [NcResizeCb].
    ///
    /// *C style function: [ncplane_resize_realign()][c_api::ncplane_resize_realign].*
    //
    // TODO: suitable for use as an NcResizeCb?
    pub fn resize_realign(&mut self) -> NcResult<()> {
        error![unsafe { c_api::ncplane_resize_realign(self) }]
    }

    /// Resizes this `NcPlane`, retaining what data we can (everything, unless we're
    /// shrinking in some dimension). Keeps the origin where it is.
    ///
    /// *C style function: [ncplane_resize_simple()][c_api::ncplane_resize_simple].*
    #[inline]
    pub fn resize_simple(&mut self, y_len: NcDim, x_len: NcDim) -> NcResult<()> {
        error![c_api::ncplane_resize_simple(
            self,
            y_len as u32,
            x_len as u32
        )]
    }

    /// Returns this `NcPlane`'s current resize callback.
    ///
    /// *C style function: [ncplane_resizecb()][c_api::ncplane_resizecb].*
    pub fn resizecb(&self) -> Option<NcResizeCb> {
        unsafe { c_api::ncresizecb_to_rust(c_api::ncplane_resizecb(self)) }
    }

    /// Replaces this `NcPlane`'s existing resize callback (which may be [None]).
    ///
    /// The standard plane's resizecb may not be changed.
    ///
    /// *C style function: [ncplane_set_resizecb()][c_api::ncplane_set_resizecb].*
    pub fn set_resizecb(&mut self, resizecb: Option<NcResizeCb>) {
        unsafe { c_api::ncplane_set_resizecb(self, c_api::ncresizecb_to_c(resizecb)) }
    }

    /// Rotate the plane π/2 radians (90°) clockwise.
    ///
    /// This cannot be performed on arbitrary planes, because glyphs cannot be
    /// arbitrarily rotated.
    ///
    /// The glyphs which can be rotated are limited: line-drawing characters,
    /// spaces, half blocks, and full blocks.
    ///
    /// The plane must have an even number of columns.
    ///
    /// Use the ncvisual rotation for a more flexible approach.
    ///
    /// *C style function: [ncplane_rotate_cw()][c_api::ncplane_rotate_cw].*
    pub fn rotate_cw(&mut self) -> NcResult<()> {
        error![unsafe { c_api::ncplane_rotate_cw(self) }]
    }

    /// Rotate the plane π/2 radians (90°) counter-clockwise.
    ///
    /// See [`rotate_cw`][NcPlane#method.rotate_cw]
    /// for more information.
    ///
    /// *C style function: [ncplane_rotate_ccw()][c_api::ncplane_rotate_ccw].*
    pub fn rotate_ccw(&mut self) -> NcResult<()> {
        error![unsafe { c_api::ncplane_rotate_ccw(self) }]
    }

    /// Maps the specified coordinates relative to the origin of this `NcPlane`,
    /// to the same absolute coordinates relative to the origin of `target`.
    ///
    /// *C style function: [ncplane_translate()][c_api::ncplane_translate].*
    //
    // TODO: API change, return the coordinates as a tuple instead of being &mut
    pub fn translate(&self, target: &NcPlane, y: &mut NcDim, x: &mut NcDim) {
        unsafe { c_api::ncplane_translate(self, target, &mut (*y as i32), &mut (*x as i32)) }
    }

    /// Returns true if the provided absolute `y`/`x` coordinates are within
    /// this `NcPlane`, or false otherwise.
    ///
    /// Either way, translates the absolute coordinates relative to this `NcPlane`.
    ///
    /// *C style function: [ncplane_translate_abs()][c_api::ncplane_translate_abs].*
    //
    // TODO: API change, return a tuple (y,x,bool)
    pub fn translate_abs(&self, y: &mut NcDim, x: &mut NcDim) -> bool {
        unsafe { c_api::ncplane_translate_abs(self, &mut (*y as i32), &mut (*x as i32)) }
    }

    /// Gets the `y`, `x` origin of this `NcPlane` relative to the standard plane,
    /// or the `NcPlane` to which it is bound.
    ///
    /// *C style function: [ncplane_yx()][c_api::ncplane_yx].*
    //
    // CHECK: negative offsets
    pub fn yx(&self) -> (NcOffset, NcOffset) {
        let (mut y, mut x) = (0, 0);
        unsafe { c_api::ncplane_yx(self, &mut y, &mut x) };
        (y as NcOffset, x as NcOffset)
    }

    /// Gets the `x` origin of this `NcPlane` relative to the standard plane,
    /// or the `NcPlane` to which it is bound.
    ///
    /// *C style function: [ncplane_x()][c_api::ncplane_x].*
    pub fn x(&self) -> NcOffset {
        unsafe { c_api::ncplane_x(self) as NcOffset }
    }

    /// Gets the `y` origin of this `NcPlane` relative to the standard plane,
    /// or the `NcPlane` to which it is bound.
    ///
    /// *C style function: [ncplane_y()][c_api::ncplane_y].*
    pub fn y(&self) -> NcOffset {
        unsafe { c_api::ncplane_y(self) as NcOffset }
    }

    /// Returns `true` if this `NcPlane` has scrolling enabled, or `false` otherwise.
    pub fn scrolling_p(&self) -> bool {
        unsafe { c_api::ncplane_scrolling_p(self) }
    }

    /// Sets the scrolling behaviour of the plane, and
    /// returns true if scrolling was previously enabled, of false, if disabled.
    ///
    /// All planes are created with scrolling disabled. Attempting to print past
    /// the end of a line will stop at the plane boundary, and indicate an error.
    ///
    /// On a plane 10 columns wide and two rows high, printing "0123456789"
    /// at the origin should succeed, but printing "01234567890" will by default
    /// fail at the eleventh character. In either case, the cursor will be left
    /// at location 0x10; it must be moved before further printing can take place. I
    ///
    /// *C style function: [ncplane_set_scrolling()][c_api::ncplane_set_scrolling].*
    pub fn set_scrolling(&mut self, scroll: bool) -> bool {
        unsafe { c_api::ncplane_set_scrolling(self, scroll) }
    }

    /// Scrolls down the current plane `r` times.
    ///
    /// Returns an error if current plane is not a scrolling plane,
    /// and otherwise returns the number of lines scrolled.
    ///
    /// *C style function: [ncplane_scrollup()][c_api::ncplane_scrollup].*
    pub fn scrollup(&mut self, r: NcDim) -> NcResult<NcDim> {
        let res = unsafe { c_api::ncplane_scrollup(self, r as i32) };
        error![res, "", res as NcDim]
    }

    /// Scrolls down the current plane until `child` is no longer hidden beneath it.
    ///
    /// Returns an error if `child` is not a child of this plane, or if this
    /// plane is not scrolling, or `child` is fixed.
    ///
    /// Returns the number of scrolling events otherwise (might be 0).
    ///
    /// *C style function: [ncplane_scrollup_child()][c_api::ncplane_scrollup_child].*
    pub fn scrollup_child(&mut self, child: &NcPlane) -> NcResult<NcDim> {
        let res = unsafe { c_api::ncplane_scrollup_child(self, child) };
        error![res, "", res as NcDim]
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: boxes & perimeters
impl NcPlane {
    /// Draws a box with its upper-left corner at the current cursor position,
    /// and its lower-right corner at `y_stop` * `x_stop`.
    ///
    /// The 6 cells provided are used to draw the upper-left, ur, ll, and lr corners,
    /// then the horizontal and vertical lines.
    ///
    /// See [NcBoxMask] for information about the border and gradient masks,
    /// and the drawing of corners.
    ///
    /// If the gradient bit is not set, the style from the hline/vlline cells
    /// is used for the horizontal and vertical lines, respectively.
    ///
    /// If the gradient bit is set, the color is linearly interpolated between
    /// the two relevant corner cells.
    ///
    /// *C style function: [ncplane_box()][c_api::ncplane_box].*
    pub fn r#box(
        &mut self,
        ul: &NcCell,
        ur: &NcCell,
        ll: &NcCell,
        lr: &NcCell,
        hline: &NcCell,
        vline: &NcCell,
        y_stop: NcDim,
        x_stop: NcDim,
        boxmask: NcBoxMask,
    ) -> NcResult<()> {
        error![unsafe {
            c_api::ncplane_box(self, ul, ur, ll, lr, hline, vline, y_stop, x_stop, boxmask)
        }]
    }

    /// Draws a box with its upper-left corner at the current cursor position,
    /// having dimensions `y_len` * `x_len`.
    /// The minimum box size is 2x2, and it cannot be drawn off-screen.
    ///
    /// See the [`box`][NcPlane#method.box] method for more information.
    ///
    /// *C style function: [ncplane_box_sized()][c_api::ncplane_box_sized].*
    #[inline]
    pub fn box_sized(
        &mut self,
        ul: &NcCell,
        ur: &NcCell,
        ll: &NcCell,
        lr: &NcCell,
        hline: &NcCell,
        vline: &NcCell,
        y_len: NcDim,
        x_len: NcDim,
        boxmask: NcBoxMask,
    ) -> NcResult<()> {
        error![c_api::ncplane_box_sized(
            self, ul, ur, ll, lr, hline, vline, y_len, x_len, boxmask
        )]
    }

    /// NcPlane.[box()][NcPlane#method.box] with the double box-drawing characters.
    ///
    /// *C style function: [ncplane_double_box()][c_api::ncplane_double_box].*
    #[inline]
    pub fn double_box(
        &mut self,
        stylemask: NcStyle,
        channels: NcChannels,
        y_stop: NcDim,
        x_stop: NcDim,
        boxmask: NcBoxMask,
    ) -> NcResult<()> {
        error![c_api::ncplane_double_box(
            self, stylemask, channels, y_stop, x_stop, boxmask
        )]
    }

    ///
    ///
    /// *C style function: [ncplane_double_box_sized()][c_api::ncplane_double_box_sized].*
    #[inline]
    pub fn double_box_sized(
        &mut self,
        stylemask: NcStyle,
        channels: NcChannels,
        y_len: NcDim,
        x_len: NcDim,
        boxmask: NcBoxMask,
    ) -> NcResult<()> {
        error![c_api::ncplane_double_box(
            self, stylemask, channels, y_len, x_len, boxmask
        )]
    }

    /// Draws the perimeter around this `NcPlane`.
    ///
    /// *C style function: [ncplane_perimeter()][c_api::ncplane_perimeter].*
    #[inline]
    pub fn perimeter(
        &mut self,
        ul: &NcCell,
        ur: &NcCell,
        ll: &NcCell,
        lr: &NcCell,
        hline: &NcCell,
        vline: &NcCell,
        boxmask: NcBoxMask,
    ) -> NcResult<()> {
        error![c_api::ncplane_perimeter(
            self, ul, ur, ll, lr, hline, vline, boxmask
        )]
    }

    /// NcPlane.[perimeter()][NcPlane#method.perimeter] with the double box-drawing characters.

    ///
    /// *C style function: [ncplane_perimeter_double()][c_api::ncplane_perimeter_double].*
    #[inline]
    pub fn perimeter_double(
        &mut self,
        stylemask: NcStyle,
        channels: NcChannels,
        boxmask: NcBoxMask,
    ) -> NcResult<()> {
        error![c_api::ncplane_perimeter_double(
            self, stylemask, channels, boxmask
        )]
    }

    /// NcPlane.[perimeter()][NcPlane#method.perimeter] with the rounded box-drawing characters.
    ///
    ///
    /// *C style function: [ncplane_perimeter_rounded()][c_api::ncplane_perimeter_rounded].*
    #[inline]
    pub fn perimeter_rounded(
        &mut self,
        stylemask: NcStyle,
        channels: NcChannels,
        boxmask: NcBoxMask,
    ) -> NcResult<()> {
        error![c_api::ncplane_perimeter_rounded(
            self, stylemask, channels, boxmask
        )]
    }
}

// -----------------------------------------------------------------------------
/// ## NcPlane methods: fading, gradients & greyscale
impl NcPlane {
    /// Fades this `NcPlane` in, over the specified time, calling 'fader' at
    /// each iteration.
    ///
    /// Usage:
    /// 1. Load this `NcPlane` with the target cells without rendering.
    /// 2. call this function.
    ///
    /// When it's done, the `NcPlane` will have reached the target levels,
    /// starting from zeroes.
    ///
    /// *C style function: [ncplane_fadein()][c_api::ncplane_fadein].*
    pub fn fadein(&mut self, time: &NcTime, fader: NcFadeCb) -> NcResult<()> {
        error![unsafe { c_api::ncplane_fadein(self, time, fader, null_mut()) }]
    }

    /// Fades in through 'iter' iterations,
    /// where 'iter' < 'ncfadectx_iterations(nctx)'.
    ///
    /// *C style function: [ncplane_fadein_iteration()][c_api::ncplane_fadein_iteration].*
    pub fn fadein_iteration(&mut self, time: &NcTime, fader: NcFadeCb) -> NcResult<()> {
        error![unsafe { c_api::ncplane_fadein(self, time, fader, null_mut()) }]
    }

    /// Fades this `NcPlane` out, over the specified time, calling 'fader' at
    /// each iteration.
    ///
    /// Requires a terminal which supports truecolor, or at least palette
    /// modification (if the terminal uses a palette, our ability to fade planes
    /// is limited, and affected by the complexity of the rest of the screen).
    ///
    /// *C style function: [ncplane_fadeout()][c_api::ncplane_fadeout].*
    pub fn fadeout(&mut self, time: &NcTime, fader: NcFadeCb) -> NcResult<()> {
        error![unsafe { c_api::ncplane_fadeout(self, time, fader, null_mut()) }]
    }

    /// Fades out through 'iter' iterations,
    /// where 'iter' < 'ncfadectx_iterations(nctx)'.
    ///
    /// *C style function: [ncplane_fadeout_iteration()][c_api::ncplane_fadeout_iteration].*
    pub fn fadeout_iteration(&mut self, time: &NcTime, fader: NcFadeCb) -> NcResult<()> {
        error![unsafe { c_api::ncplane_fadeout(self, time, fader, null_mut()) }]
    }

    /// Pulses this `NcPlane` in and out until the callback returns non-zero,
    /// relying on the callback 'fader' to initiate rendering.
    ///
    /// `time` defines the half-period (i.e. the transition from black to full
    /// brightness, or back again).
    ///
    /// Proper use involves preparing (but not rendering) the `NcPlane`,
    /// then calling this method, which will fade in from black to the
    /// specified colors.
    ///
    /// *C style function: [ncplane_pulse()][c_api::ncplane_pulse].*
    pub fn pulse(&mut self, time: &NcTime, fader: NcFadeCb) -> NcResult<()> {
        error![unsafe { c_api::ncplane_pulse(self, time, fader, null_mut()) }]
    }

    /// Draws a gradient with its upper-left corner at the current cursor
    /// position, stopping at `y_stop` * `xstop`.
    ///
    /// Returns the number of cells filled on success,
    /// or [`NcIntResult::ERR`][NcIntResult#associatedconstant.ERR] on error.
    ///
    /// The glyph composed of `egc` and `stylemask` is used for all cells.
    /// The channels specified by `ul`, `ur`, `ll`, and `lr` are composed into
    /// foreground and background gradients.
    ///
    /// To do a vertical gradient, `ul` ought equal `ur` and `ll` ought equal
    /// `lr`. To do a horizontal gradient, `ul` ought equal `ll` and `ur` ought
    /// equal `ul`.
    ///
    /// To color everything the same, all four channels should be equivalent.
    /// The resulting alpha values are equal to incoming alpha values.
    ///
    /// Palette-indexed color is not supported.
    ///
    /// Preconditions for gradient operations (error otherwise):
    ///
    /// all: only RGB colors, unless all four channels match as default
    /// all: all alpha values must be the same
    /// 1x1: all four colors must be the same
    /// 1xN: both top and both bottom colors must be the same (vertical gradient)
    /// Nx1: both left and both right colors must be the same (horizontal gradient)
    ///
    /// *C style function: [ncplane_gradient()][c_api::ncplane_gradient].*
    pub fn gradient(
        &mut self,
        y: Option<NcDim>,
        x: Option<NcDim>,
        y_stop: Option<NcDim>,
        x_stop: Option<NcDim>,
        egc: &str,
        stylemask: NcStyle,
        ul: NcChannels,
        ur: NcChannels,
        ll: NcChannels,
        lr: NcChannels,
    ) -> NcResult<NcDim> {
        let res =
            c_api::ncplane_gradient(self, y, x, y_stop, x_stop, egc, stylemask, ul, ur, ll, lr);
        error![res, "", res as NcDim]
    }

    /// Does a high-resolution gradient using upper blocks and synced backgrounds.
    ///
    /// This doubles the number of vertical gradations, but restricts you to
    /// half blocks (appearing to be full blocks).
    ///
    /// Returns the number of cells filled on success.
    ///
    /// Use `None` for either or all of `beg_y` and `beg_x` in order to
    /// use the current cursor position along that axis.
    ///
    /// Use `None` for either or both of `len_y` and `len_x` in order to
    /// go through the boundary of the plane in that axis (same as `0`).
    ///
    /// *C style function: [ncplane_gradient2x1()][c_api::ncplane_gradient2x1].*
    pub fn gradient2x1(
        &mut self,
        y: Option<NcDim>,
        x: Option<NcDim>,
        len_y: Option<NcDim>,
        len_x: Option<NcDim>,
        ul: NcChannel,
        ur: NcChannel,
        ll: NcChannel,
        lr: NcChannel,
    ) -> NcResult<NcDim> {
        let res = unsafe {
            c_api::ncplane_gradient2x1(
                self,
                y.unwrap_or(NcDim::MAX) as i32,
                x.unwrap_or(NcDim::MAX) as i32,
                len_y.unwrap_or(0),
                len_x.unwrap_or(0),
                ul,
                ur,
                ll,
                lr,
            )
        };
        error![res, "", res as NcDim]
    }

    /// Converts this `NcPlane`'s content to greyscale.
    ///
    /// *C style function: [ncplane_greyscale()][c_api::ncplane_greyscale].*
    pub fn greyscale(&mut self) {
        unsafe {
            c_api::ncplane_greyscale(self);
        }
    }
}
