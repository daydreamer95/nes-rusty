pub mod cartridge;
pub mod joypad;
pub mod opharn;
pub mod ppu;
pub mod render;
pub mod virtual_nes;

use std::cell::RefCell;
use std::rc::Rc;

// use sdl2::EventPump;
// use sdl2::event::Event;
// use sdl2::keyboard::Keycode;
// use sdl2::pixels::Color;
// use sdl2::pixels::PixelFormatEnum;

use crate::joypad::JoypadButton;
use crate::ppu::Interface as PpuInterface;
use crate::render::Frame;
use crate::virtual_nes::Interface;
use mos6502::cpu::Context as CpuContext;

use leptos::logging::log; // <--- Import from core leptos crate
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    pub static ref SNAKE_GAME_CODE: Vec<u8> = vec![
        0x20, 0x06, 0x06, 0x20, 0x38, 0x06, 0x20, 0x0d, 0x06, 0x20, 0x2a, 0x06, 0x60, 0xa9, 0x02,
        0x85, 0x02, 0xa9, 0x04, 0x85, 0x03, 0xa9, 0x11, 0x85, 0x10, 0xa9, 0x10, 0x85, 0x12, 0xa9,
        0x0f, 0x85, 0x14, 0xa9, 0x04, 0x85, 0x11, 0x85, 0x13, 0x85, 0x15, 0x60, 0xa5, 0xfe, 0x85,
        0x00, 0xa5, 0xfe, 0x29, 0x03, 0x18, 0x69, 0x02, 0x85, 0x01, 0x60, 0x20, 0x4d, 0x06, 0x20,
        0x8d, 0x06, 0x20, 0xc3, 0x06, 0x20, 0x19, 0x07, 0x20, 0x20, 0x07, 0x20, 0x2d, 0x07, 0x4c,
        0x38, 0x06, 0xa5, 0xff, 0xc9, 0x77, 0xf0, 0x0d, 0xc9, 0x64, 0xf0, 0x14, 0xc9, 0x73, 0xf0,
        0x1b, 0xc9, 0x61, 0xf0, 0x22, 0x60, 0xa9, 0x04, 0x24, 0x02, 0xd0, 0x26, 0xa9, 0x01, 0x85,
        0x02, 0x60, 0xa9, 0x08, 0x24, 0x02, 0xd0, 0x1b, 0xa9, 0x02, 0x85, 0x02, 0x60, 0xa9, 0x01,
        0x24, 0x02, 0xd0, 0x10, 0xa9, 0x04, 0x85, 0x02, 0x60, 0xa9, 0x02, 0x24, 0x02, 0xd0, 0x05,
        0xa9, 0x08, 0x85, 0x02, 0x60, 0x60, 0x20, 0x94, 0x06, 0x20, 0xa8, 0x06, 0x60, 0xa5, 0x00,
        0xc5, 0x10, 0xd0, 0x0d, 0xa5, 0x01, 0xc5, 0x11, 0xd0, 0x07, 0xe6, 0x03, 0xe6, 0x03, 0x20,
        0x2a, 0x06, 0x60, 0xa2, 0x02, 0xb5, 0x10, 0xc5, 0x10, 0xd0, 0x06, 0xb5, 0x11, 0xc5, 0x11,
        0xf0, 0x09, 0xe8, 0xe8, 0xe4, 0x03, 0xf0, 0x06, 0x4c, 0xaa, 0x06, 0x4c, 0x35, 0x07, 0x60,
        0xa6, 0x03, 0xca, 0x8a, 0xb5, 0x10, 0x95, 0x12, 0xca, 0x10, 0xf9, 0xa5, 0x02, 0x4a, 0xb0,
        0x09, 0x4a, 0xb0, 0x19, 0x4a, 0xb0, 0x1f, 0x4a, 0xb0, 0x2f, 0xa5, 0x10, 0x38, 0xe9, 0x20,
        0x85, 0x10, 0x90, 0x01, 0x60, 0xc6, 0x11, 0xa9, 0x01, 0xc5, 0x11, 0xf0, 0x28, 0x60, 0xe6,
        0x10, 0xa9, 0x1f, 0x24, 0x10, 0xf0, 0x1f, 0x60, 0xa5, 0x10, 0x18, 0x69, 0x20, 0x85, 0x10,
        0xb0, 0x01, 0x60, 0xe6, 0x11, 0xa9, 0x06, 0xc5, 0x11, 0xf0, 0x0c, 0x60, 0xc6, 0x10, 0xa5,
        0x10, 0x29, 0x1f, 0xc9, 0x1f, 0xf0, 0x01, 0x60, 0x4c, 0x35, 0x07, 0xa0, 0x00, 0xa5, 0xfe,
        0x91, 0x00, 0x60, 0xa6, 0x03, 0xa9, 0x00, 0x81, 0x10, 0xa2, 0x00, 0xa9, 0x01, 0x81, 0x10,
        0x60, 0xa6, 0xff, 0xea, 0xea, 0xca, 0xd0, 0xfb, 0x60,
    ];
    pub static ref TEST_ASSEMBLY_CODE: Vec<u8> = vec![
        0xa9, 0xc1, 0x8d, 0x00, 0x02, 0xa9, 0xa5, 0x8d, 0xff, 0x02, 0xa9, 0xf8, 0x8d, 0xff, 0x05,
    ];
}

fn main() {
    // init sdl2
    // let sdl_context = sdl2::init().unwrap();
    // let video_subsystem = sdl_context.video().unwrap();
    // let window = video_subsystem
    //     .window("Untitle", (256.0 * 3.0) as u32, (240.0 * 3.0) as u32)
    //     .position_centered()
    //     .build()
    //     .unwrap();
    //
    // let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    // let mut event_pump = sdl_context.event_pump().unwrap();
    // canvas.set_scale(3.0, 3.0).unwrap();
    //...

    //try leptops
    leptos::mount::mount_to_body(App);

    //...
    // let creator = canvas.texture_creator();
    // let mut texture = creator
    //     .create_texture_target(PixelFormatEnum::RGB24, 256, 240)
    //     .unwrap();
    //...//
    // let mut emulator =
    //     virtual_nes::Emulator::new("/Users/huy/Source/snes-rusty/Super_mario_bros.nes".to_string());
    //
    // virtual_nes::Interface::reset(&mut emulator);
    // key map
    // let mut key_map = HashMap::new();
    // key_map.insert(Keycode::S, joypad::JoypadButton::DOWN);
    // key_map.insert(Keycode::W, joypad::JoypadButton::UP);
    // key_map.insert(Keycode::A, joypad::JoypadButton::LEFT);
    // key_map.insert(Keycode::D, joypad::JoypadButton::RIGHT);
    // key_map.insert(Keycode::Return, joypad::JoypadButton::START);
    // key_map.insert(Keycode::Space, joypad::JoypadButton::SELECT);
    // key_map.insert(Keycode::J, joypad::JoypadButton::BUTTON_A);
    // key_map.insert(Keycode::K, joypad::JoypadButton::BUTTON_B);

    // let mut frame = Frame::new();
    // virtual_nes::Interface::run_with_callback(&mut emulator, move |emulator| {
    //     if !emulator.ppu_state.frame_completed {
    //         return;
    //     }
    //
    //     emulator.ppu_state.frame_completed = false;
    //     render::render(&mut emulator.ppu_state, &mut frame);
    //     texture.update(None, &frame.data, 256 * 3).unwrap();
    //
    //     canvas.copy(&texture, None, None).unwrap();
    //
    //     canvas.present();
    //     for event in event_pump.poll_iter() {
    //         match event {
    //             Event::Quit { .. }
    //             | Event::KeyDown {
    //                 keycode: Some(Keycode::Escape),
    //                 ..
    //             } => std::process::exit(0),
    //             Event::KeyDown { keycode, .. } => {
    //                 if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
    //                     emulator.joypad1.set_button_pressed_status(*key, true);
    //                 }
    //             }
    //             Event::KeyUp { keycode, .. } => {
    //                 if let Some(key) = key_map.get(&keycode.unwrap_or(Keycode::Ampersand)) {
    //                     emulator.joypad1.set_button_pressed_status(*key, false);
    //                 }
    //             }
    //
    //             _ => { /* do nothing */ }
    //         }
    //     }
    // });
}

#[component]
fn App() -> impl IntoView {
    view! {
        <GameCanvas/>
        <p>
            "This might be the end of my passion for code"
        </p>
    }
}

// 1. Compile the local file directly into the WebAssembly binary.
// Path is relative to this source file (.rs).
const ROM_BINARY_DATA: &[u8] = include_bytes!("../Super_mario_bros.nes");

#[component]
fn GameCanvas() -> impl IntoView {
    // 1. Define dimensions
    let native_width = 256;
    let native_height = 240;

    // 1.1. Scale factor
    let scale = 2;
    let display_width = native_width * scale; // 768px
    let display_height = native_height * scale; // 720px

    // 2. Create a reference for the canvas element
    let canvas_ref = NodeRef::<leptos::html::Canvas>::new();

    log!("Init loading emulator");
    let emulator = Rc::new(RefCell::new(virtual_nes::Emulator::new_with_gamecodes(
        ROM_BINARY_DATA.to_vec(),
    )));
    virtual_nes::Interface::reset(&mut *emulator.borrow_mut());

    let frame = Rc::new(RefCell::new(Frame::new()));
    let canvas_buffer = Rc::new(RefCell::new(vec![0u8; native_width * native_height * 4]));
    let context_rc: Rc<RefCell<Option<Rc<CanvasRenderingContext2d>>>> = Rc::new(RefCell::new(None));
    let animation_handle: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let input_listeners_ready = Rc::new(RefCell::new(false));

    {
        let canvas_ref = canvas_ref.clone();
        let emulator = emulator.clone();
        let frame = frame.clone();
        let canvas_buffer = canvas_buffer.clone();
        let context_rc = context_rc.clone();
        let animation_handle_inner = animation_handle.clone();
        let input_listeners_ready = input_listeners_ready.clone();
        let window = web_sys::window().expect("window not available");
        let window_clone = window.clone();

        let closure = Closure::wrap(Box::new(move || {
            if !*input_listeners_ready.borrow() {
                let window = web_sys::window().expect("window not available");
                let emulator_for_keydown = emulator.clone();
                let keydown = Closure::wrap(Box::new(move |event: web_sys::Event| {
                    let event: web_sys::KeyboardEvent = event.unchecked_into();
                    let key = event.key();
                    let code = event.code();
                    let button = match key.as_str() {
                        "ArrowDown" | "s" | "S" => Some(JoypadButton::DOWN),
                        "ArrowUp" | "w" | "W" => Some(JoypadButton::UP),
                        "ArrowLeft" | "a" | "A" => Some(JoypadButton::LEFT),
                        "ArrowRight" | "d" | "D" => Some(JoypadButton::RIGHT),
                        "Enter" => Some(JoypadButton::START),
                        " " | "Space" | "Spacebar" => Some(JoypadButton::SELECT),
                        "j" | "J" => Some(JoypadButton::BUTTON_A),
                        "k" | "K" => Some(JoypadButton::BUTTON_B),
                        _ => match code.as_str() {
                            "ArrowDown" => Some(JoypadButton::DOWN),
                            "ArrowUp" => Some(JoypadButton::UP),
                            "ArrowLeft" => Some(JoypadButton::LEFT),
                            "ArrowRight" => Some(JoypadButton::RIGHT),
                            "Enter" => Some(JoypadButton::START),
                            "Space" => Some(JoypadButton::SELECT),
                            _ => None,
                        },
                    };
                    if let Some(button) = button {
                        event.prevent_default();
                        emulator_for_keydown
                            .borrow_mut()
                            .joypad1
                            .set_button_pressed_status(button, true);
                    }
                }) as Box<dyn FnMut(_)>);
                window
                    .add_event_listener_with_callback("keydown", keydown.as_ref().unchecked_ref())
                    .unwrap();
                keydown.forget();

                let emulator_for_keyup = emulator.clone();
                let keyup = Closure::wrap(Box::new(move |event: web_sys::Event| {
                    let event: web_sys::KeyboardEvent = event.unchecked_into();
                    let key = event.key();
                    let code = event.code();
                    let button = match key.as_str() {
                        "ArrowDown" | "s" | "S" => Some(JoypadButton::DOWN),
                        "ArrowUp" | "w" | "W" => Some(JoypadButton::UP),
                        "ArrowLeft" | "a" | "A" => Some(JoypadButton::LEFT),
                        "ArrowRight" | "d" | "D" => Some(JoypadButton::RIGHT),
                        "Enter" => Some(JoypadButton::START),
                        " " | "Space" | "Spacebar" => Some(JoypadButton::SELECT),
                        "j" | "J" => Some(JoypadButton::BUTTON_A),
                        "k" | "K" => Some(JoypadButton::BUTTON_B),
                        _ => match code.as_str() {
                            "ArrowDown" => Some(JoypadButton::DOWN),
                            "ArrowUp" => Some(JoypadButton::UP),
                            "ArrowLeft" => Some(JoypadButton::LEFT),
                            "ArrowRight" => Some(JoypadButton::RIGHT),
                            "Enter" => Some(JoypadButton::START),
                            "Space" => Some(JoypadButton::SELECT),
                            _ => None,
                        },
                    };
                    if let Some(button) = button {
                        event.prevent_default();
                        emulator_for_keyup
                            .borrow_mut()
                            .joypad1
                            .set_button_pressed_status(button, false);
                    }
                }) as Box<dyn FnMut(_)>);
                window
                    .add_event_listener_with_callback("keyup", keyup.as_ref().unchecked_ref())
                    .unwrap();
                keyup.forget();
                *input_listeners_ready.borrow_mut() = true;
            }

            let context = if let Some(context) = context_rc.borrow().clone() {
                context
            } else if let Some(canvas_node) = canvas_ref.get() {
                let canvas: HtmlCanvasElement = canvas_node.unchecked_into();
                let ctx: CanvasRenderingContext2d =
                    canvas.get_context("2d").unwrap().unwrap().unchecked_into();
                let ctx = Rc::new(ctx);
                *context_rc.borrow_mut() = Some(ctx.clone());
                ctx
            } else {
                if let Some(callback) = animation_handle_inner.borrow().as_ref() {
                    window_clone
                        .request_animation_frame(callback.as_ref().unchecked_ref())
                        .unwrap();
                }
                return;
            };

            let mut emulator = emulator.borrow_mut();
            let mut frame = frame.borrow_mut();
            let mut canvas_buffer = canvas_buffer.borrow_mut();

            (&mut *emulator).run_with_callback_until(
                |emulator| emulator.ppu_state.frame_completed,
                |emulator| ppu::Interface::poll_nmi_interrupt(emulator.newtype_mut()),
                |emulator, cycles| {
                    CpuContext::tick(emulator.newtype_mut(), cycles);
                    PpuInterface::tick(emulator.newtype_mut(), cycles * 3);
                },
            );

            emulator.ppu_state.frame_completed = false;
            render::render(&mut emulator.ppu_state, &mut frame);

            for y in 0..native_height {
                for x in 0..native_width {
                    let rgb_base = (y * native_width + x) * 3;
                    let rgba_base = (y * native_width + x) * 4;

                    canvas_buffer[rgba_base] = frame.data[rgb_base];
                    canvas_buffer[rgba_base + 1] = frame.data[rgb_base + 1];
                    canvas_buffer[rgba_base + 2] = frame.data[rgb_base + 2];
                    canvas_buffer[rgba_base + 3] = 255;
                }
            }

            let image_data = ImageData::new_with_u8_clamped_array_and_sh(
                Clamped(&canvas_buffer),
                native_width as u32,
                native_height as u32,
            )
            .unwrap();
            context.put_image_data(&image_data, 0.0, 0.0).unwrap();

            if let Some(callback) = animation_handle_inner.borrow().as_ref() {
                window_clone
                    .request_animation_frame(callback.as_ref().unchecked_ref())
                    .unwrap();
            }
        }) as Box<dyn FnMut()>);

        *animation_handle.borrow_mut() = Some(closure);
        window
            .request_animation_frame(
                animation_handle
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
            )
            .unwrap();
    }

    view! {
        <div style="display: flex; flex-direction: column; align-items: center; background: #111; padding: 20px; color: #f5f5f5;">
            <div style="display: flex; justify-content: center; align-items: center;">
                <canvas
                    node_ref=canvas_ref
                    width=native_width
                    height=native_height
                    style=format!(
                        "width: {}px; height: {}px; image-rendering: pixelated; image-rendering: crisp-edges; border: 4px solid #333;",
                        display_width,
                        display_height
                    )
                />
            </div>

            <div style="margin-top: 16px; width: min(100%, 620px); padding: 12px 14px; border: 1px solid #2a2a2a; border-radius: 12px; background: linear-gradient(135deg, #1b1b1b, #171717); box-shadow: 0 6px 20px rgba(0, 0, 0, 0.25);">
                <div style="font-size: 12px; font-weight: 700; letter-spacing: 0.12em; text-transform: uppercase; color: #9fb2c9; margin-bottom: 10px;">Controls</div>

                <div style="display: flex; flex-wrap: wrap; gap: 10px;">
                    <div style="flex: 1 1 180px; padding: 10px; border: 1px solid #2b2b2b; border-radius: 10px; background: rgba(255,255,255,0.03);">
                        <div style="font-size: 11px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: #8ea6c6; margin-bottom: 8px;">Direction</div>
                        <div style="display: flex; flex-wrap: wrap; gap: 8px;">
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="min-width: 36px; padding: 4px 8px; border-radius: 8px; border: 1px solid #4a4a4a; background: #252525; font-size: 12px; font-weight: 700; text-align: center;">A</span>
                                <span style="font-size: 13px; color: #e6e6e6;">Left</span>
                            </div>
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="min-width: 36px; padding: 4px 8px; border-radius: 8px; border: 1px solid #4a4a4a; background: #252525; font-size: 12px; font-weight: 700; text-align: center;">D</span>
                                <span style="font-size: 13px; color: #e6e6e6;">Right</span>
                            </div>
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="min-width: 36px; padding: 4px 8px; border-radius: 8px; border: 1px solid #4a4a4a; background: #252525; font-size: 12px; font-weight: 700; text-align: center;">W</span>
                                <span style="font-size: 13px; color: #e6e6e6;">Up</span>
                            </div>
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="min-width: 36px; padding: 4px 8px; border-radius: 8px; border: 1px solid #4a4a4a; background: #252525; font-size: 12px; font-weight: 700; text-align: center;">S</span>
                                <span style="font-size: 13px; color: #e6e6e6;">Down</span>
                            </div>
                        </div>
                    </div>

                    <div style="flex: 1 1 180px; padding: 10px; border: 1px solid #2b2b2b; border-radius: 10px; background: rgba(255,255,255,0.03);">
                        <div style="font-size: 11px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: #8ea6c6; margin-bottom: 8px;">A / B</div>
                        <div style="display: flex; flex-wrap: wrap; gap: 8px;">
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="min-width: 36px; padding: 4px 8px; border-radius: 8px; border: 1px solid #4a4a4a; background: #252525; font-size: 12px; font-weight: 700; text-align: center;">J</span>
                                <span style="font-size: 13px; color: #e6e6e6;">Button A</span>
                            </div>
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="min-width: 36px; padding: 4px 8px; border-radius: 8px; border: 1px solid #4a4a4a; background: #252525; font-size: 12px; font-weight: 700; text-align: center;">K</span>
                                <span style="font-size: 13px; color: #e6e6e6;">Button B</span>
                            </div>
                        </div>
                    </div>

                    <div style="flex: 1 1 180px; padding: 10px; border: 1px solid #2b2b2b; border-radius: 10px; background: rgba(255,255,255,0.03);">
                        <div style="font-size: 11px; font-weight: 700; letter-spacing: 0.1em; text-transform: uppercase; color: #8ea6c6; margin-bottom: 8px;">Start / Pause</div>
                        <div style="display: flex; flex-wrap: wrap; gap: 8px;">
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="min-width: 36px; padding: 4px 8px; border-radius: 8px; border: 1px solid #4a4a4a; background: #252525; font-size: 12px; font-weight: 700; text-align: center;">Enter</span>
                                <span style="font-size: 13px; color: #e6e6e6;">Start</span>
                            </div>
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <span style="min-width: 36px; padding: 4px 8px; border-radius: 8px; border: 1px solid #4a4a4a; background: #252525; font-size: 12px; font-weight: 700; text-align: center;">Space</span>
                                <span style="font-size: 13px; color: #e6e6e6;">Pause / Select</span>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
