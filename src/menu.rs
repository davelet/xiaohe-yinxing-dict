#[cfg(target_os = "macos")]
use keyboard_types::Code;

// ============================================================================
// macOS ObjC 运行时 FFI 封装
// ============================================================================
//
// 笔记：为什么需要这个模块？
// ─────────────────────────
// 本应用的 GUI 框架（eframe/winit/muda）内部已经依赖 objc2 库，但版本不统一：
//   - winit 依赖 objc2 0.5.2
//   - muda  依赖 objc2 0.6.4
// 两个版本的内部类型不兼容，直接引入 objc2 容易引发链接冲突或 API 不对齐。
// 因此这里使用最原始的 ObjC Runtime（objc_getClass / sel_registerName /
// objc_msgSend）直接操作 NSObject，避免引入第三方 objc crate。
//
// 笔记：为什么 objc_msgSend 不能用 variadic FFI（fn(...)）声明？
// ───────────────────────────────────────────────────────────────
// 在 Apple Silicon（ARM64）上，C variadic 函数的参数通过栈传递，而
// objc_msgSend 的实际 ABI 要求参数放在寄存器（x0, x1, …）。两者
// 不一致会导致参数值错乱，启动即崩溃（panic_cannot_unwind / abort）。
//
// 正确的做法：把 objc_msgSend 声明为无参函数（仅拿地址），然后通过
// std::mem::transmute 转为精确签名的函数指针后再调用，这样编译器会
// 生成正确的寄存器传参代码。
#[cfg(target_os = "macos")]
mod macos_ffi {
    use std::os::raw::{c_char, c_void};

    unsafe extern "C" {
        pub fn objc_getClass(name: *const c_char) -> *const c_void;
        pub fn sel_registerName(name: *const c_char) -> *const c_void;
        /// 仅用于获取函数地址，禁止直接调用。
        /// 实际调用前通过 transmute 转为精确签名的函数指针。
        pub fn objc_msgSend();
    }

    /// 等价于 [obj sel] → id
    pub unsafe fn send0(obj: *const c_void, sel: *const c_void) -> *const c_void {
        let f: unsafe extern "C" fn(*const c_void, *const c_void) -> *const c_void =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { f(obj, sel) }
    }

    /// 等价于 [obj sel: arg] → id
    pub unsafe fn send1_ret(
        obj: *const c_void,
        sel: *const c_void,
        arg: *const c_void,
    ) -> *const c_void {
        let f: unsafe extern "C" fn(*const c_void, *const c_void, *const c_void) -> *const c_void =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { f(obj, sel, arg) }
    }

    /// 等价于 [obj sel: arg] → void
    #[allow(dead_code)]
    pub unsafe fn send1_void(obj: *const c_void, sel: *const c_void, arg: *const c_void) {
        let f: unsafe extern "C" fn(*const c_void, *const c_void, *const c_void) =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { f(obj, sel, arg) };
    }

    /// 等价于 [obj sel: arg1 arg2] → void
    /// 用于 NSMutableDictionary.setObject:forKey: 等双参无返回值方法
    pub unsafe fn send2_void(
        obj: *const c_void,
        sel: *const c_void,
        arg1: *const c_void,
        arg2: *const c_void,
    ) {
        let f: unsafe extern "C" fn(*const c_void, *const c_void, *const c_void, *const c_void) =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        unsafe { f(obj, sel, arg1, arg2) };
    }
}

/// 菜单事件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEvent {
    /// 打开帮助
    OpenHelp,
    /// 打开关于
    OpenAbout,
    /// 打开 AI 对话
    OpenAi,
    /// 打开 Rime 数据管理
    OpenRimeData,
}

/// 菜单项 ID
pub const MENU_HELP: &str = "menu_help";
pub const MENU_ABOUT: &str = "menu_about";
pub const MENU_AI: &str = "menu_ai";
pub const MENU_RIME: &str = "menu_rime";

/// 创建并初始化 macOS 原生菜单
pub fn setup_menu() {
    #[cfg(target_os = "macos")]
    {
        use muda::{
            Menu, MenuItem, PredefinedMenuItem, Submenu,
            accelerator::{Accelerator, Modifiers},
        };

        let menu = Menu::new();

        // 1. 应用菜单（macOS 菜单栏的首个菜单必须是 App Submenu）
        let app_menu = Submenu::new("小鹤音形词典", true);
        menu.append(&app_menu).unwrap();

        let open_about_app = MenuItem::with_id(MENU_ABOUT, "关于小鹤音形词典", true, None);
        app_menu.append(&open_about_app).unwrap();
        app_menu.append(&PredefinedMenuItem::separator()).unwrap();
        app_menu
            .append(&PredefinedMenuItem::hide(Some("隐藏小鹤音形词典")))
            .unwrap();
        app_menu
            .append(&PredefinedMenuItem::hide_others(Some("隐藏其他")))
            .unwrap();
        app_menu
            .append(&PredefinedMenuItem::show_all(Some("全部显示")))
            .unwrap();
        app_menu.append(&PredefinedMenuItem::separator()).unwrap();
        app_menu
            .append(&PredefinedMenuItem::quit(Some("退出小鹤音形词典")))
            .unwrap();

        // 2. 视图菜单
        let view_menu = Submenu::new("视图", true);
        menu.append(&view_menu).unwrap();

        let open_ai = MenuItem::with_id(
            MENU_AI,
            "AI 助手",
            true,
            Some(Accelerator::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyA,
            )),
        );
        view_menu.append(&open_ai).unwrap();

        let open_rime = MenuItem::with_id(
            MENU_RIME,
            "Rime 数据管理",
            true,
            Some(Accelerator::new(
                Some(Modifiers::SUPER | Modifiers::SHIFT),
                Code::KeyR,
            )),
        );
        view_menu.append(&open_rime).unwrap();

        // 3. 帮助菜单
        let help_menu = Submenu::new("帮助", true);
        menu.append(&help_menu).unwrap();

        let open_help = MenuItem::with_id(
            MENU_HELP,
            "打开帮助文档",
            true,
            Some(Accelerator::new(None, Code::F1)),
        );
        help_menu.append(&open_help).unwrap();

        // ================================================================
        // 笔记：让 macOS App 菜单显示中文名"小鹤音形词典"
        // ================================================================
        //
        // 问题：muda 已经创建了 title 为"小鹤音形词典"的 App Submenu，但
        // macOS 在调用 [NSApp setMainMenu:] 时，会从 NSBundle.mainBundle
        // .infoDictionary 中读取 CFBundleName / CFBundleDisplayName，并
        // 用它覆盖 App 菜单第一个 item 的 submenu 标题。因此菜单栏始终显
        // 示进程名（如 xiaohe-yinxing-dict）或 Info.plist 中注册的英文名，
        // 而不是 muda 传入的中文。
        //
        // 尝试过的无效方案：
        //   1. 在 init_for_nsapp() 之后用 objc_msgSend 调 NSMenu.setTitle:
        //      → setTitle: 不影响 macOS App 菜单的显示标题，系统会继续用
        //        CFBundleName 覆盖。
        //   2. 用 [NSApplication setApplicationName:]
        //      → 同样会被后续的 setMainMenu: 内部逻辑覆盖。
        //
        // 正确方案：
        //   在 init_for_nsapp() 之前，直接修改 NSBundle.mainBundle
        //   .infoDictionary 里的 CFBundleName 和 CFBundleDisplayName。
        //   这样当 init_for_nsapp → setMainMenu: 内部读取 bundle 信息时，
        //   拿到的就是我们预先注入的中文名。
        //
        // 等价的 ObjC 代码：
        //   NSMutableDictionary *info = [[[NSBundle mainBundle] infoDictionary] mutableCopy];
        //   info[@"CFBundleName"] = @"小鹤音形词典";
        //   info[@"CFBundleDisplayName"] = @"小鹤音形词典";
        // 注意：由于 infoDictionary 实际返回的是
        //   __NSDictionaryM（内部可变），可以直接 setObject:forKey:，
        //   不需要 mutableCopy。
        //
        // ================================================================
        #[cfg(target_os = "macos")]
        unsafe {
            use macos_ffi::*;
            use std::ffi::CString;
            use std::os::raw::c_void;

            let cls_bundle = CString::new("NSBundle").unwrap();
            let cls_nsstr = CString::new("NSString").unwrap();
            let sel_main_bundle = CString::new("mainBundle").unwrap();
            let sel_info_dict = CString::new("infoDictionary").unwrap();
            let sel_str = CString::new("stringWithUTF8String:").unwrap();
            let sel_set_obj = CString::new("setObject:forKey:").unwrap();

            let bundle_cls = objc_getClass(cls_bundle.as_ptr());
            let bundle = send0(bundle_cls, sel_registerName(sel_main_bundle.as_ptr()));
            let info_dict = send0(bundle, sel_registerName(sel_info_dict.as_ptr()));

            if !info_dict.is_null() {
                let ns_cls = objc_getClass(cls_nsstr.as_ptr());

                let name_c = CString::new("小鹤音形词典").unwrap();
                let ns_name = send1_ret(
                    ns_cls,
                    sel_registerName(sel_str.as_ptr()),
                    name_c.as_ptr() as *const c_void,
                );

                // 设置 CFBundleName —— setMainMenu: 时会读取此项作为 App 菜单标题
                let key_c = CString::new("CFBundleName").unwrap();
                let ns_key = send1_ret(
                    ns_cls,
                    sel_registerName(sel_str.as_ptr()),
                    key_c.as_ptr() as *const c_void,
                );
                send2_void(
                    info_dict,
                    sel_registerName(sel_set_obj.as_ptr()),
                    ns_name,
                    ns_key,
                );

                // 也设置 CFBundleDisplayName（某些系统场景会读此项）
                let display_key_c = CString::new("CFBundleDisplayName").unwrap();
                let ns_display_key = send1_ret(
                    ns_cls,
                    sel_registerName(sel_str.as_ptr()),
                    display_key_c.as_ptr() as *const c_void,
                );
                send2_void(
                    info_dict,
                    sel_registerName(sel_set_obj.as_ptr()),
                    ns_name,
                    ns_display_key,
                );
            }
        }

        // init_for_nsapp 内部调用 [NSApp setMainMenu:]，
        // 此时系统读取 CFBundleName → 获得"小鹤音形词典"。
        menu.init_for_nsapp();

        // 必须通过 Box::leak 保持 menu 及其内部菜单项在程序生命周期内存活。
        // 若 menu 在 setup_menu 结束时被释放，muda 会销毁内部 Rust 对象（MenuChild），
        // 导致 macOS 原生 NSMenuItem 点击回调时访问悬空指针（EXC_BAD_ACCESS）并引发崩溃。
        Box::leak(Box::new(menu));
    }
}

/// 处理菜单事件，更新应用状态。
///
/// 每个菜单动作先清理互斥的视图状态，保证切换后目标视图可见：
///   - Help 和 Chat 互相排斥（都独占 CentralPanel）
///   - Manager 和 Chat 互相排斥（Chat 的 show_viewport 优先级高于 current_view）
pub(crate) fn handle_menu_event(app: &mut crate::DictApp, event: MenuEvent) {
    match event {
        MenuEvent::OpenHelp => {
            let was_open = app.show_help_panel;
            app.show_help_panel = !app.show_help_panel;
            if app.show_help_panel {
                // 打开帮助时退出 Chat，避免帮助关闭后意外回到 Chat
                if !was_open {
                    app.chat.show_viewport = false;
                }
                if app.selected_help_chapter.is_none() {
                    app.selected_help_chapter = Some("readme".to_string());
                }
            }
        }
        MenuEvent::OpenAbout => {
            app.show_about_dialog = true;
        }
        MenuEvent::OpenAi => {
            app.chat.show_viewport = !app.chat.show_viewport;
            if app.chat.show_viewport {
                // 打开 Chat 时退出帮助，否则帮助盖住 Chat 不可见
                app.show_help_panel = false;
                app.current_view = crate::types::ViewMode::Chat;
            } else {
                // 关闭 Chat 时恢复到之前的视图（Dict 作为默认兜底）
                if app.current_view == crate::types::ViewMode::Chat {
                    app.current_view = crate::types::ViewMode::Dict;
                }
                app.search_auto_focus = true;
            }
        }
        MenuEvent::OpenRimeData => {
            // 退出 Chat 和帮助，保证 Manager 可见
            app.chat.show_viewport = false;
            app.show_help_panel = false;
            app.current_view = crate::types::ViewMode::Manager;
        }
    }
}

/// 轮询菜单事件
pub fn poll_menu_events() -> Option<MenuEvent> {
    #[cfg(target_os = "macos")]
    {
        use muda::MenuEvent as MudaMenuEvent;
        if let Ok(event) = MudaMenuEvent::receiver().try_recv() {
            let id = event.id().as_ref();
            match id {
                MENU_HELP => Some(MenuEvent::OpenHelp),
                MENU_ABOUT => Some(MenuEvent::OpenAbout),
                MENU_AI => Some(MenuEvent::OpenAi),
                MENU_RIME => Some(MenuEvent::OpenRimeData),
                _ => None,
            }
        } else {
            None
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        None
    }
}
