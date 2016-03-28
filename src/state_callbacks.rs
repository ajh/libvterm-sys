#![allow(unused_variables)]

use libc::{c_int, c_void};
use std::sync::mpsc::Sender;
use std::ffi::CStr;

use super::*;

// int (*putglyph)(VTermGlyphInfo *info, VTermPos pos, void *user);
pub extern "C" fn put_glyph(info: *mut ffi::VTermGlyphInfo,
                        pos: ffi::VTermPos,
                        vterm: *mut c_void)
                        -> c_int {
    cast_vterm(vterm, |vterm, tx| {
        let event = StateEvent::PutGlyph(PutGlyphEvent {
            glyph_info: ::GlyphInfo::from_ptr(info),
            pos: pos.as_pos(),
        });

        match tx.send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

// int (*movecursor)(VTermPos pos, VTermPos oldpos, int visible, void *user);
pub extern "C" fn move_cursor(new: ffi::VTermPos,
                          old: ffi::VTermPos,
                          visible: c_int,
                          vterm: *mut c_void)
                          -> c_int {
    cast_vterm(vterm, |vterm, tx| {
        let event = StateEvent::MoveCursor(MoveCursorEvent {
            new: new.as_pos(),
            old: old.as_pos(),
            is_visible: int_to_bool(visible),
        });

        match tx.send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

// int (*scrollrect)(VTermRect rect, int downward, int rightward, void *user);
pub extern "C" fn scroll_rect(rect: ffi::VTermRect,
                          downward: c_int,
                          rightward: c_int,
                          vterm: *mut c_void)
                          -> c_int {
    cast_vterm(vterm, |vterm, tx| {
        let event = StateEvent::ScrollRect(ScrollRectEvent {
            rect: rect.as_rect(),
            downward: downward as isize,
            rightward: rightward as isize,
        });

        match tx.send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

// int (*moverect)(VTermRect dest, VTermRect src, void *user);
pub extern "C" fn move_rect(dest: ffi::VTermRect, src: ffi::VTermRect, vterm: *mut c_void) -> c_int {
    cast_vterm(vterm, |vterm, tx| {
        let event = StateEvent::MoveRect(MoveRectEvent {
            src: src.as_rect(),
            dest: dest.as_rect(),
        });

        match tx.send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

// int (*erase)(VTermRect rect, int selective, void *user);
pub extern "C" fn erase(rect: ffi::VTermRect, selective: c_int, vterm: *mut c_void) -> c_int {
    cast_vterm(vterm, |vterm, tx| {
        let event = StateEvent::Erase(EraseEvent {
            rect: rect.as_rect(),
            is_selective: int_to_bool(selective),
        });

        match tx.send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

// int (*initpen)(void *user);
pub extern "C" fn init_pen(vterm: *mut c_void) -> c_int {
    cast_vterm(vterm, |vterm, tx| {
        let event = StateEvent::InitPen(InitPenEvent);
        match tx.send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

// int (*setpenattr)(VTermAttr attr, VTermValue *val, void *user);
pub extern "C" fn set_pen_attr(attr: ffi::VTermAttr,
                           val: *mut ffi::VTermValue,
                           vterm: *mut c_void)
                           -> c_int {
    println!("set_pen_attr {:?}", attr);
    cast_vterm(vterm, |vterm, tx| {
        let event: StateEvent = match attr {
            ffi::VTermAttr::Bold => {
                let val = unsafe { int_to_bool(ffi::vterm_value_get_boolean(val)).clone() };
                StateEvent::PenBold(PenBoldEvent { is_true: val })
            }
            ffi::VTermAttr::Background => {
                let rgb: ColorRGB = unsafe { ffi::vterm_value_get_color(val).as_color_rgb() };
                let palette = vterm.state_get_palette_color_from_rgb(&rgb);
                StateEvent::PenBackground(PenBackgroundEvent {
                    rgb: rgb, palette: palette })
            }
            ffi::VTermAttr::Blink => {
                let val = unsafe { int_to_bool(ffi::vterm_value_get_boolean(val)).clone() };
                StateEvent::PenBlink(PenBlinkEvent { is_true: val })
            }
            ffi::VTermAttr::Font => {
                let val = unsafe { ffi::vterm_value_get_number(val).clone() };
                StateEvent::PenFont(PenFontEvent { value: val })
            }
            ffi::VTermAttr::Foreground => {
                let rgb: ColorRGB = unsafe { ffi::vterm_value_get_color(val).as_color_rgb() };
                let palette = vterm.state_get_palette_color_from_rgb(&rgb);
                StateEvent::PenForeground(PenForegroundEvent {
                    rgb: rgb, palette: palette })
            }
            ffi::VTermAttr::Italic => {
                let val = unsafe { int_to_bool(ffi::vterm_value_get_boolean(val)).clone() };
                StateEvent::PenItalic(PenItalicEvent { is_true: val })
            }
            ffi::VTermAttr::Reverse => {
                let val = unsafe { int_to_bool(ffi::vterm_value_get_boolean(val)).clone() };
                StateEvent::PenReverse(PenReverseEvent { is_true: val })
            }
            ffi::VTermAttr::Strike => {
                let val = unsafe { int_to_bool(ffi::vterm_value_get_boolean(val)).clone() };
                StateEvent::PenStrike(PenStrikeEvent { is_true: val })
            }
            ffi::VTermAttr::Underline => {
                let val = unsafe { Underline::from_i32(ffi::vterm_value_get_number(val).clone()) };
                StateEvent::PenUnderline(PenUnderlineEvent { value: val })
            }
        };

        match tx.send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

// int (*settermprop)(VTermProp prop, VTermValue *val, void *user);
pub extern "C" fn set_term_prop(prop: ffi::VTermProp,
                            val: *mut ffi::VTermValue,
                            vterm: *mut c_void)
                            -> c_int {
    cast_vterm(vterm, |vterm, tx| {
        let event: StateEvent = match prop {
            ffi::VTermProp::VTermPropCursorVisible => {
                let val = unsafe { int_to_bool(ffi::vterm_value_get_boolean(val)).clone() };
                StateEvent::CursorVisible(CursorVisibleEvent { is_true: val })
            }

            ffi::VTermProp::VTermPropAltscreen => {
                let val = unsafe { int_to_bool(ffi::vterm_value_get_boolean(val)).clone() };
                StateEvent::AltScreen(AltScreenEvent { is_true: val })
            }

            ffi::VTermProp::VTermPropCursorBlink => {
                let val = unsafe { int_to_bool(ffi::vterm_value_get_boolean(val)).clone() };
                StateEvent::CursorBlink(CursorBlinkEvent { is_true: val })
            }

            ffi::VTermProp::VTermPropCursorShape => {
                let val: CursorShape = unsafe { CursorShape::from_i32(ffi::vterm_value_get_number(val)) };
                StateEvent::CursorShape(CursorShapeEvent { shape: val })
            }

            ffi::VTermProp::VTermPropIconName => {
                let val: String = unsafe { CStr::from_ptr(ffi::vterm_value_get_string(val)).to_string_lossy().into_owned() };
                StateEvent::IconName(IconNameEvent { text: val })
            }

            ffi::VTermProp::VTermPropMouse => {
                let val: MouseMode = unsafe { MouseMode::from_i32(ffi::vterm_value_get_number(val)) };
                StateEvent::Mouse(MouseEvent { mode: val })
            },

            ffi::VTermProp::VTermPropReverse => {
                let val = unsafe { int_to_bool(ffi::vterm_value_get_boolean(val)).clone() };
                StateEvent::Reverse(ReverseEvent { is_true: val })
            },

            ffi::VTermProp::VTermPropTitle => {
                let val: String = unsafe { CStr::from_ptr(ffi::vterm_value_get_string(val)).to_string_lossy().into_owned() };
                StateEvent::Title(TitleEvent { text: val })
            }
        };

        match tx.send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

// int (*bell)(void *user);
pub extern "C" fn bell(vterm: *mut c_void) -> c_int {
    cast_vterm(vterm, |vterm, tx| {
        let event = StateEvent::Bell(BellEvent);
        match tx.send(event) {
            Ok(_) => 1,
            Err(_) => 0,
        }
    })
}

// int (*resize)(int rows, int cols, VTermPos *delta, void *user);
pub extern "C" fn resize(rows: c_int,
                     cols: c_int,
                     delta: *mut ffi::VTermPos,
                     vterm: *mut c_void)
                     -> c_int {
    0
}
// int (*setlineinfo)(int row, const VTermLineInfo *newinfo, const VTermLineInfo *oldinfo, void *user);
pub extern "C" fn set_line_info(row: c_int,
                            new: *const ffi::VTermLineInfo,
                            old: *const ffi::VTermLineInfo,
                            vterm: *mut c_void)
                            -> c_int {
    0
}

/// Call the given closure with the vterms sender, if it exists.
fn cast_vterm<F>(vterm: *mut c_void, closure: F) -> c_int
    where F: Fn(&VTerm, &Sender<StateEvent>) -> c_int
{
    let vterm: &VTerm = unsafe { &mut *(vterm as *mut VTerm) };
    match vterm.state_event_tx.as_ref() {
        Some(tx) => closure(vterm, tx),
        None => 0,
    }
}
