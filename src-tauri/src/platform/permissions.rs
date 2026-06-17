// platform/permissions.rs — macOS 辅助功能权限检查（真实的，不是 Enigo::new() 探测）。
//
// AXIsProcessTrusted 是 ApplicationServices 框架的 API：返回当前进程是否
// 在 系统设置 → 隐私与安全 → 辅助功能 中被授权了 模拟键盘事件。
//
// 没授权时 enigo 的 Cmd+V 会被静默丢弃 —— 这是 "已复制但没注入" 的根因。

#[cfg(target_os = "macos")]
mod imp {
    use core::ffi::c_void;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    }

    pub fn is_trusted() -> bool {
        unsafe { AXIsProcessTrusted() }
    }

    /// 调一次系统弹窗：如果未授权，会弹出"打开系统设置"的引导。
    /// 返回值与 is_trusted() 相同；只是顺便触发引导。
    pub fn prompt_trust() -> bool {
        // 构造 CFDictionary { kAXTrustedCheckOptionPrompt: kCFBooleanTrue }
        // 这里通过 core-foundation 类型，但避免引入额外 crate；用 FFI 写死。
        use std::ptr;

        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            fn CFDictionaryCreate(
                allocator: *const c_void,
                keys: *const *const c_void,
                values: *const *const c_void,
                num_values: isize,
                key_callbacks: *const c_void,
                value_callbacks: *const c_void,
            ) -> *const c_void;
            fn CFRelease(cf: *const c_void);
        }

        #[link(name = "ApplicationServices", kind = "framework")]
        extern "C" {
            static kAXTrustedCheckOptionPrompt: *const c_void;
            static kCFBooleanTrue: *const c_void;
            static kCFTypeDictionaryKeyCallBacks: c_void;
            static kCFTypeDictionaryValueCallBacks: c_void;
        }

        unsafe {
            let key = kAXTrustedCheckOptionPrompt;
            let val = kCFBooleanTrue;
            let dict = CFDictionaryCreate(
                ptr::null(),
                &key as *const _ as *const *const c_void,
                &val as *const _ as *const *const c_void,
                1,
                &kCFTypeDictionaryKeyCallBacks as *const _ as *const c_void,
                &kCFTypeDictionaryValueCallBacks as *const _ as *const c_void,
            );
            let trusted = AXIsProcessTrustedWithOptions(dict);
            if !dict.is_null() {
                CFRelease(dict);
            }
            trusted
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod imp {
    pub fn is_trusted() -> bool {
        true
    }
    pub fn prompt_trust() -> bool {
        true
    }
}

pub fn is_trusted() -> bool {
    imp::is_trusted()
}
pub fn prompt_trust() -> bool {
    imp::prompt_trust()
}
