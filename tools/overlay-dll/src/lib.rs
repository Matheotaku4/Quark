//! Quark overlay DLL (Windows, Direct3D 11).
//!
//! This library is installed next to a game as `d3d11.dll`. It forwards the
//! three D3D11 entry points to the system DLL, then patches the `Present` slot
//! of the first swap chain it sees. A small layered GDI window draws a
//! Steam-like FPS counter in a corner of the game window.
//!
//! It deliberately does not depend on Quark or egui: the DLL runs inside the
//! game process, often under Proton/Wine, and must stay self-contained.

#![cfg(windows)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::c_void;
use std::mem::size_of;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicPtr, AtomicU32, Ordering};
use windows::core::{w, GUID, PCWSTR};
use windows::Win32::Foundation::{BOOL, COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, EndPaint, FillRect, InvalidateRect,
    SelectObject, SetBkMode, SetTextColor, TextOutW, CLIP_DEFAULT_PRECIS, DEFAULT_CHARSET,
    DEFAULT_QUALITY, FF_DONTCARE, FW_BOLD, HBRUSH, HGDIOBJ, OUT_DEFAULT_PRECIS, PAINTSTRUCT,
    TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress, LoadLibraryW};
use windows::Win32::System::Memory::{VirtualProtect, PAGE_PROTECTION_FLAGS, PAGE_READWRITE};
use windows::Win32::System::SystemInformation::GetSystemDirectoryW;
use windows::Win32::System::Threading::{CreateThread, GetCurrentProcessId, THREAD_CREATION_FLAGS};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetClientRect, GetForegroundWindow,
    GetMessageW, GetWindowRect, GetWindowThreadProcessId, RegisterClassW,
    SetLayeredWindowAttributes, SetTimer, SetWindowPos, ShowWindow, TranslateMessage, CS_HREDRAW,
    CS_VREDRAW, HWND_TOPMOST, LWA_ALPHA, MSG, SWP_NOACTIVATE, SWP_NOSIZE, SW_HIDE,
    SW_SHOWNOACTIVATE, WM_DESTROY, WM_PAINT, WM_TIMER, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
};

const TIMER_ID: usize = 0x51_4B;
const OVERLAY_WIDTH: i32 = 176;
const OVERLAY_HEIGHT: i32 = 42;

/// Number of `Present` calls since the last timer tick.
static FRAMES: AtomicU32 = AtomicU32::new(0);
/// Last measured value, read by the paint handler.
static FPS: AtomicU32 = AtomicU32::new(0);
static HOOKED: AtomicBool = AtomicBool::new(false);
static OVERLAY_HWND: AtomicIsize = AtomicIsize::new(0);
static LAST_GAME_HWND: AtomicIsize = AtomicIsize::new(0);
static ORIGINAL_PRESENT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static FACTORY_HOOKED: AtomicBool = AtomicBool::new(false);
static ORIGINAL_CREATE_SWAPCHAIN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_CREATE_SWAPCHAIN_FOR_HWND: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static ORIGINAL_CREATE_SWAPCHAIN_FOR_COMPOSITION: AtomicPtr<c_void> =
    AtomicPtr::new(std::ptr::null_mut());

type PresentFn = unsafe extern "system" fn(*mut c_void, u32, u32) -> i32;
type QueryInterfaceFn =
    unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32;
type GetAdapterFn = unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> i32;
type GetParentFn = unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> i32;
type CreateSwapChainFn =
    unsafe extern "system" fn(*mut c_void, *mut c_void, *const c_void, *mut *mut c_void) -> i32;
type CreateSwapChainForHwndFn = unsafe extern "system" fn(
    *mut c_void,
    *mut c_void,
    *mut c_void,
    *const c_void,
    *const c_void,
    *mut c_void,
    *mut *mut c_void,
) -> i32;
type CreateSwapChainForCompositionFn = unsafe extern "system" fn(
    *mut c_void,
    *mut c_void,
    *const c_void,
    *mut c_void,
    *mut *mut c_void,
) -> i32;
type CreateDeviceFn = unsafe extern "system" fn(
    *mut c_void,
    i32,
    *mut c_void,
    u32,
    *const i32,
    u32,
    u32,
    *mut *mut c_void,
    *mut i32,
    *mut *mut c_void,
) -> i32;
type CreateDeviceAndSwapChainFn = unsafe extern "system" fn(
    *mut c_void,
    i32,
    *mut c_void,
    u32,
    *const i32,
    u32,
    u32,
    *const c_void,
    *mut *mut c_void,
    *mut *mut c_void,
    *mut i32,
    *mut *mut c_void,
) -> i32;

/// IID_IDXGIDevice et IID_IDXGIFactory : les seules interfaces nécessaires pour
/// remonter du device D3D11 à la fabrique DXGI. Les GUID sont stables.
const IID_IDXGIDEVICE: GUID = GUID::from_u128(0x54ec77fa_1377_44e6_8c32_88fd5f44c84c);
const IID_IDXGIFACTORY: GUID = GUID::from_u128(0x7b7166ec_21c7_44ae_b21a_ae5f5b1f1e7e);

#[repr(C)]
struct FactoryVtbl {
    query_interface: *mut c_void,
    add_ref: *mut c_void,
    release: *mut c_void,
    set_private_data: *mut c_void,
    set_private_data_interface: *mut c_void,
    get_private_data: *mut c_void,
    get_parent: *mut c_void,
    enum_adapters: *mut c_void,
    make_window_association: *mut c_void,
    get_window_association: *mut c_void,
    create_swap_chain: CreateSwapChainFn,
    create_software_adapter: *mut c_void,
    enum_adapters1: *mut c_void,
    is_current: *mut c_void,
    is_windowed_stereo_enabled: *mut c_void,
    create_swap_chain_for_hwnd: CreateSwapChainForHwndFn,
    create_swap_chain_for_core_window: *mut c_void,
    create_swap_chain_for_composition: CreateSwapChainForCompositionFn,
}

#[repr(C)]
struct SwapChainVtbl {
    query_interface: *mut c_void,
    add_ref: *mut c_void,
    release: *mut c_void,
    // IDXGIObject
    set_private_data: *mut c_void,
    set_private_data_interface: *mut c_void,
    get_private_data: *mut c_void,
    get_parent: *mut c_void,
    // IDXGISwapChain : Present est le 18e pointeur de la vtable.
    get_device: *mut c_void,
    get_buffer: *mut c_void,
    set_fullscreen_state: *mut c_void,
    get_fullscreen_state: *mut c_void,
    get_desc: *mut c_void,
    resize_buffers: *mut c_void,
    resize_target: *mut c_void,
    get_containing_output: *mut c_void,
    get_frame_statistics: *mut c_void,
    get_last_present_count: *mut c_void,
    present: PresentFn,
}

/// Loads the real system `d3d11.dll`, never this proxy. `C:\Windows\System32`
/// is also valid under Wine/Proton, where it points inside the prefix.
unsafe fn system_module() -> Option<windows::Win32::Foundation::HMODULE> {
    let mut path = [0u16; 260];
    let len = GetSystemDirectoryW(Some(&mut path)) as usize;
    if len == 0 || len >= path.len() {
        return None;
    }
    let suffix: Vec<u16> = "\\d3d11.dll"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    if len + suffix.len() > path.len() {
        return None;
    }
    path[len..len + suffix.len()].copy_from_slice(&suffix);
    LoadLibraryW(PCWSTR(path.as_ptr())).ok()
}

unsafe fn real_proc(name: &[u8]) -> Option<*mut c_void> {
    let module = system_module()?;
    let address = GetProcAddress(module, windows::core::PCSTR(name.as_ptr()));
    address.map(|proc| proc as *mut c_void)
}

/// Patch une entrée de vtable en restaurant la protection d'origine ensuite.
unsafe fn patch<T: Copy>(slot: *mut T, replacement: T, original: &AtomicPtr<c_void>) -> bool {
    let address = slot as *mut c_void;
    let mut old = PAGE_PROTECTION_FLAGS(0);
    if VirtualProtect(address, size_of::<T>(), PAGE_READWRITE, &mut old).is_err() {
        return false;
    }
    let previous: *mut c_void = std::mem::transmute_copy(&*slot);
    original.store(previous, Ordering::SeqCst);
    *slot = replacement;
    let _ = VirtualProtect(address, size_of::<T>(), old, &mut old);
    true
}

/// Remonte `ID3D11Device` → `IDXGIDevice` → adapter → `IDXGIFactory`, puis
/// accroche les trois façons de créer un swap chain. Les jeux qui appellent
/// `D3D11CreateDevice` puis une fabrique DXGI sont ainsi couverts, pas
/// seulement `D3D11CreateDeviceAndSwapChain`.
unsafe fn hook_factory(device: *mut c_void) {
    if device.is_null() || FACTORY_HOOKED.load(Ordering::SeqCst) {
        return;
    }
    let device_vtable = *(device as *const *const *mut c_void);
    if device_vtable.is_null() {
        return;
    }
    let query_interface: QueryInterfaceFn = std::mem::transmute(*device_vtable);
    let mut dxgi_device = std::ptr::null_mut();
    if query_interface(device, &IID_IDXGIDEVICE, &mut dxgi_device) < 0 || dxgi_device.is_null() {
        return;
    }
    let dxgi_vtable = *(dxgi_device as *const *const *mut c_void);
    if dxgi_vtable.is_null() {
        return;
    }
    let get_adapter: GetAdapterFn = std::mem::transmute(*dxgi_vtable.add(7));
    let mut adapter = std::ptr::null_mut();
    if get_adapter(dxgi_device, &mut adapter) < 0 || adapter.is_null() {
        return;
    }
    let adapter_vtable = *(adapter as *const *const *mut c_void);
    if adapter_vtable.is_null() {
        return;
    }
    let get_parent: GetParentFn = std::mem::transmute(*adapter_vtable.add(6));
    let mut factory = std::ptr::null_mut();
    if get_parent(adapter, &IID_IDXGIFACTORY, &mut factory) < 0 || factory.is_null() {
        return;
    }
    let factory_vtable = *(factory as *const *mut FactoryVtbl);
    if factory_vtable.is_null() {
        return;
    }
    let mut patched = false;
    patched |= patch(
        &mut (*factory_vtable).create_swap_chain,
        create_swapchain_hook,
        &ORIGINAL_CREATE_SWAPCHAIN,
    );
    patched |= patch(
        &mut (*factory_vtable).create_swap_chain_for_hwnd,
        create_swapchain_for_hwnd_hook,
        &ORIGINAL_CREATE_SWAPCHAIN_FOR_HWND,
    );
    patched |= patch(
        &mut (*factory_vtable).create_swap_chain_for_composition,
        create_swapchain_for_composition_hook,
        &ORIGINAL_CREATE_SWAPCHAIN_FOR_COMPOSITION,
    );
    FACTORY_HOOKED.store(patched, Ordering::SeqCst);
}

unsafe extern "system" fn create_swapchain_hook(
    factory: *mut c_void,
    device: *mut c_void,
    desc: *const c_void,
    swapchain: *mut *mut c_void,
) -> i32 {
    let original = ORIGINAL_CREATE_SWAPCHAIN.load(Ordering::Relaxed);
    if original.is_null() {
        return -2005270483;
    }
    let original: CreateSwapChainFn = std::mem::transmute(original);
    let result = original(factory, device, desc, swapchain);
    if result >= 0 && !swapchain.is_null() {
        hook_swapchain(*swapchain);
    }
    result
}

unsafe extern "system" fn create_swapchain_for_hwnd_hook(
    factory: *mut c_void,
    device: *mut c_void,
    hwnd: *mut c_void,
    desc: *const c_void,
    fullscreen_desc: *const c_void,
    restrict_to_output: *mut c_void,
    swapchain: *mut *mut c_void,
) -> i32 {
    let original = ORIGINAL_CREATE_SWAPCHAIN_FOR_HWND.load(Ordering::Relaxed);
    if original.is_null() {
        return -2005270483;
    }
    let original: CreateSwapChainForHwndFn = std::mem::transmute(original);
    let result = original(
        factory,
        device,
        hwnd,
        desc,
        fullscreen_desc,
        restrict_to_output,
        swapchain,
    );
    if result >= 0 && !swapchain.is_null() {
        hook_swapchain(*swapchain);
    }
    result
}

unsafe extern "system" fn create_swapchain_for_composition_hook(
    factory: *mut c_void,
    device: *mut c_void,
    desc: *const c_void,
    restrict_to_output: *mut c_void,
    swapchain: *mut *mut c_void,
) -> i32 {
    let original = ORIGINAL_CREATE_SWAPCHAIN_FOR_COMPOSITION.load(Ordering::Relaxed);
    if original.is_null() {
        return -2005270483;
    }
    let original: CreateSwapChainForCompositionFn = std::mem::transmute(original);
    let result = original(factory, device, desc, restrict_to_output, swapchain);
    if result >= 0 && !swapchain.is_null() {
        hook_swapchain(*swapchain);
    }
    result
}

/// Patches the `Present` entry of a swap-chain vtable. Vtables are shared by
/// all swap chains of the same class in the process, so this only runs once.
unsafe fn hook_swapchain(swapchain: *mut c_void) {
    if swapchain.is_null() || HOOKED.swap(true, Ordering::SeqCst) {
        return;
    }
    let vtable = *(swapchain as *const *mut SwapChainVtbl);
    if vtable.is_null() {
        HOOKED.store(false, Ordering::SeqCst);
        return;
    }
    let present = (*vtable).present;
    ORIGINAL_PRESENT.store(present as *mut c_void, Ordering::SeqCst);

    let slot = &mut (*vtable).present as *mut PresentFn as *mut c_void;
    let mut old = PAGE_PROTECTION_FLAGS(0);
    if VirtualProtect(slot, size_of::<PresentFn>(), PAGE_READWRITE, &mut old).is_err() {
        ORIGINAL_PRESENT.store(std::ptr::null_mut(), Ordering::SeqCst);
        HOOKED.store(false, Ordering::SeqCst);
        return;
    }
    *slot.cast::<PresentFn>() = present_hook;
    let _ = VirtualProtect(slot, size_of::<PresentFn>(), old, &mut old);
}

unsafe extern "system" fn present_hook(
    swapchain: *mut c_void,
    sync_interval: u32,
    flags: u32,
) -> i32 {
    FRAMES.fetch_add(1, Ordering::Relaxed);
    let original = ORIGINAL_PRESENT.load(Ordering::Relaxed);
    if original.is_null() {
        return 0;
    }
    let original: PresentFn = std::mem::transmute(original);
    original(swapchain, sync_interval, flags)
}

/// Forwarded proxy entry point. A game that imports `d3d11.dll` loads this
/// module from its own folder; the call is sent to the system implementation.
#[no_mangle]
pub unsafe extern "system" fn D3D11CreateDevice(
    adapter: *mut c_void,
    driver_type: i32,
    software: *mut c_void,
    flags: u32,
    feature_levels: *const i32,
    level_count: u32,
    sdk_version: u32,
    device: *mut *mut c_void,
    feature_level: *mut i32,
    context: *mut *mut c_void,
) -> i32 {
    let Some(proc) = real_proc(b"D3D11CreateDevice\0") else {
        return -2005270483; // E_FAIL
    };
    let create: CreateDeviceFn = std::mem::transmute(proc);
    let result = create(
        adapter,
        driver_type,
        software,
        flags,
        feature_levels,
        level_count,
        sdk_version,
        device,
        feature_level,
        context,
    );
    if result >= 0 && !device.is_null() {
        hook_factory(*device);
    }
    result
}

/// Forwarded proxy entry point plus swap-chain hook installation.
#[no_mangle]
pub unsafe extern "system" fn D3D11CreateDeviceAndSwapChain(
    adapter: *mut c_void,
    driver_type: i32,
    software: *mut c_void,
    flags: u32,
    feature_levels: *const i32,
    level_count: u32,
    sdk_version: u32,
    swapchain_desc: *const c_void,
    swapchain: *mut *mut c_void,
    device: *mut *mut c_void,
    feature_level: *mut i32,
    context: *mut *mut c_void,
) -> i32 {
    let Some(proc) = real_proc(b"D3D11CreateDeviceAndSwapChain\0") else {
        return -2005270483;
    };
    let create: CreateDeviceAndSwapChainFn = std::mem::transmute(proc);
    let result = create(
        adapter,
        driver_type,
        software,
        flags,
        feature_levels,
        level_count,
        sdk_version,
        swapchain_desc,
        swapchain,
        device,
        feature_level,
        context,
    );
    if result >= 0 && !swapchain.is_null() {
        hook_swapchain(*swapchain);
    }
    if result >= 0 && !device.is_null() {
        hook_factory(*device);
    }
    result
}

/// Minimal Direct3D 11 proxy export used by some D3D12 games.
#[no_mangle]
pub unsafe extern "system" fn D3D11On12CreateDevice(
    device: *mut c_void,
    flags: u32,
    feature_levels: *const i32,
    level_count: u32,
    queues: *const *mut c_void,
    queue_count: u32,
    node_mask: u32,
    d3d11_device: *mut *mut c_void,
    context: *mut *mut c_void,
    feature_level: *mut i32,
) -> i32 {
    let Some(proc) = real_proc(b"D3D11On12CreateDevice\0") else {
        return -2005270483;
    };
    let create: unsafe extern "system" fn(
        *mut c_void,
        u32,
        *const i32,
        u32,
        *const *mut c_void,
        u32,
        u32,
        *mut *mut c_void,
        *mut *mut c_void,
        *mut i32,
    ) -> i32 = std::mem::transmute(proc);
    create(
        device,
        flags,
        feature_levels,
        level_count,
        queues,
        queue_count,
        node_mask,
        d3d11_device,
        context,
        feature_level,
    )
}

/// True when the game requested the FPS overlay. Quark sets this variable on
/// launch; the DLL remains inert (but forwards D3D11) when it is absent.
fn fps_enabled() -> bool {
    std::env::var("QUARK_OVERLAY_FPS")
        .map(|value| value != "0" && !value.is_empty())
        .unwrap_or(false)
}

fn position_overlay(hwnd: HWND) {
    unsafe {
        let foreground = GetForegroundWindow();
        let overlay = HWND(OVERLAY_HWND.load(Ordering::Relaxed) as *mut c_void);
        let mut process_id = 0u32;
        let _ = GetWindowThreadProcessId(foreground, Some(&mut process_id));
        let foreground_is_game = foreground.0 != overlay.0
            && !foreground.0.is_null()
            && process_id == GetCurrentProcessId();
        if foreground_is_game {
            LAST_GAME_HWND.store(foreground.0 as isize, Ordering::Relaxed);
        } else if foreground.0 != overlay.0 {
            // Quark, le bureau ou une autre application : l'overlay se cache au
            // lieu de rester au-dessus de tout l'écran.
            let _: BOOL = ShowWindow(hwnd, SW_HIDE);
            return;
        }
        let game = HWND(LAST_GAME_HWND.load(Ordering::Relaxed) as *mut c_void);
        let mut rect = RECT::default();
        if game.0.is_null() || GetWindowRect(game, &mut rect).is_err() {
            return;
        }
        let width = rect.right - rect.left;
        if width < 320 {
            return;
        }
        let right = std::env::var("QUARK_OVERLAY_CORNER")
            .map(|corner| corner.eq_ignore_ascii_case("top-right"))
            .unwrap_or(false);
        let x = if right {
            rect.right - OVERLAY_WIDTH - 16
        } else {
            rect.left + 16
        };
        let _ = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            x,
            rect.top + 16,
            0,
            0,
            SWP_NOSIZE | SWP_NOACTIVATE,
        );
        let _: BOOL = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
}

unsafe extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_TIMER if wparam.0 == TIMER_ID => {
            let frames = FRAMES.swap(0, Ordering::Relaxed);
            FPS.store(frames.saturating_mul(4), Ordering::Relaxed);
            position_overlay(hwnd);
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut paint);
            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);
            let brush: HBRUSH = CreateSolidBrush(COLORREF(0x0010_1010));
            FillRect(hdc, &rect, brush);
            let _ = DeleteObject(HGDIOBJ(brush.0));

            let font = CreateFontW(
                -20,
                0,
                0,
                0,
                FW_BOLD.0 as i32,
                0,
                0,
                0,
                DEFAULT_CHARSET,
                OUT_DEFAULT_PRECIS,
                CLIP_DEFAULT_PRECIS,
                DEFAULT_QUALITY,
                FF_DONTCARE.0 as u32,
                w!("Segoe UI"),
            );
            let old_font = SelectObject(hdc, HGDIOBJ(font.0));
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, COLORREF(0x0078_FF78));
            let text = format!("{} FPS", FPS.load(Ordering::Relaxed));
            let utf16: Vec<u16> = text.encode_utf16().collect();
            let _ = TextOutW(hdc, 14, 10, &utf16);
            SelectObject(hdc, old_font);
            let _ = DeleteObject(HGDIOBJ(font.0));
            let _ = EndPaint(hwnd, &paint);
            LRESULT(0)
        }
        WM_DESTROY => {
            windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

/// Creates the layered, click-through FPS window and runs its message loop.
unsafe fn overlay_thread() {
    let instance = match GetModuleHandleW(None) {
        Ok(instance) => HINSTANCE(instance.0),
        Err(_) => return,
    };
    let class_name = w!("QuarkOverlayWindowD3D11");
    let wc = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(wndproc),
        hInstance: instance,
        lpszClassName: class_name,
        ..Default::default()
    };
    if RegisterClassW(&wc) == 0 {
        return;
    }
    let ex_style =
        WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE;
    let hwnd = match CreateWindowExW(
        ex_style,
        class_name,
        w!("Quark FPS"),
        WS_POPUP,
        0,
        0,
        OVERLAY_WIDTH,
        OVERLAY_HEIGHT,
        None,
        None,
        Some(instance),
        None,
    ) {
        Ok(hwnd) => hwnd,
        Err(_) => return,
    };
    OVERLAY_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
    let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 218, LWA_ALPHA);
    // La fenêtre reste cachée jusqu'au premier tick, qui la place au-dessus
    // du jeu : sans cela elle clignote brièvement dans le coin (0, 0).
    let _ = SetTimer(Some(hwnd), TIMER_ID, 250, None);
    let mut message = MSG::default();
    while GetMessageW(&mut message, None, 0, 0).0 > 0 {
        let _ = TranslateMessage(&message);
        DispatchMessageW(&message);
    }
}

unsafe extern "system" fn init_thread(_: *mut c_void) -> u32 {
    if fps_enabled() {
        overlay_thread();
    }
    0
}

/// DLL entry point. A thread is created instead of doing work under the loader
/// lock; the D3D11 exports still forward correctly even when FPS is disabled.
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    instance: *mut c_void,
    reason: u32,
    _reserved: *mut c_void,
) -> BOOL {
    if reason == 1 {
        // DLL_PROCESS_ATTACH
        let _ = CreateThread(
            None,
            0,
            Some(init_thread),
            None,
            THREAD_CREATION_FLAGS(0),
            None,
        );
    }
    let _ = instance;
    true.into()
}
