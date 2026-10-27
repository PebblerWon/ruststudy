use gpui_kit::{Context, Hsla, Render, rgb};

use gpui_kit::component::ActiveTheme as _;

#[derive(Debug, Clone, Copy)]
pub struct AppThemePalette {
    pub bg_primary: Hsla,
    pub bg_secondary: Hsla,
    pub bg_tertiary: Hsla,
    pub bg_elevated: Hsla,
    pub bg_overlay: Hsla,
    pub bg_glass: Hsla,

    pub border_primary: Hsla,
    pub border_secondary: Hsla,
    pub border_strong: Hsla,
    pub border_focus: Hsla,

    pub text_primary: Hsla,
    pub text_secondary: Hsla,
    pub text_tertiary: Hsla,
    pub text_muted: Hsla,

    pub color_accent: Hsla,
    pub color_accent_hover: Hsla,
    pub color_accent_bg: Hsla,

    pub color_warning: Hsla,
    pub color_warning_bg: Hsla,

    pub color_error: Hsla,
    pub color_success: Hsla,

    pub color_fall: Hsla,
    pub color_rise: Hsla,
}

impl AppThemePalette {
    #[inline(always)]
    pub fn light() -> Self {
        Self {
            // bg
            bg_primary: rgb(0xfafafa).into(),
            bg_secondary: rgb(0xffffff).into(),
            bg_tertiary: rgb(0xf4f4f5).into(),
            bg_elevated: rgb(0xe4e4e7).into(),
            bg_overlay: rgb(0xfafafa).into(),
            bg_glass: rgb(0xffffff).into(),

            // border
            border_primary: rgb(0xe4e4e7).into(),
            border_secondary: rgb(0xf4f4f5).into(),
            border_strong: rgb(0xd4d4d8).into(),
            border_focus: rgb(0x18181b).into(),

            // text
            text_primary: rgb(0x09090b).into(),
            text_secondary: rgb(0x27272a).into(),
            text_tertiary: rgb(0x71717a).into(),
            text_muted: rgb(0xa1a1aa).into(),

            // accent
            color_accent: rgb(0x2563eb).into(),
            color_accent_hover: rgb(0x1d4ed8).into(),
            color_accent_bg: rgb(0x2563eb).alpha(0.08).into(),

            // warning
            color_warning: rgb(0xd97706).into(),
            color_warning_bg: rgb(0xd97706).alpha(0.08).into(),

            // status
            color_error: rgb(0xdc2626).into(),
            color_success: rgb(0x16a34a).into(),

            color_fall: rgb(0x10b981).into(),
            color_rise: rgb(0xef4444).into(),
        }
    }

    // 这里后面你补 dark() 深色版本即可
    #[inline(always)]
    pub fn dark() -> Self {
        // todo: 后续补充深色色值
        Self::light()
    }

    #[inline(always)]
    pub fn for_appearance(is_dark: bool) -> Self {
        match is_dark {
            true => Self::dark(),
            false => Self::light(),
        }
    }
}

// 快捷入口，render顶层调用一次
#[inline(always)]
pub fn theme_palette(cx: &mut Context<impl Render>) -> AppThemePalette {
    AppThemePalette::for_appearance(cx.theme().is_dark())
}
