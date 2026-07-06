mod app;
mod components;
mod render;

pub(crate) use app::Message;

pub use app::run;

use std::borrow::Cow;

#[cfg(target_os = "android")]
static ANDROID_APP: std::sync::OnceLock<
    iced_winit::winit::platform::android::activity::AndroidApp,
> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
static ANDROID_TOP_SAFE_AREA: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

pub(crate) fn android_system_fonts() -> Vec<Cow<'static, [u8]>> {
    #[cfg(target_os = "android")]
    {
        return android_system_font_paths()
            .iter()
            .filter_map(|path| match std::fs::read(path) {
                Ok(bytes) => Some(Cow::Owned(bytes)),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                Err(error) => {
                    eprintln!("Skipping Android system font {path}: {error}");
                    None
                }
            })
            .collect();
    }

    #[cfg(not(target_os = "android"))]
    Vec::new()
}

pub(crate) fn android_top_safe_area() -> f32 {
    #[cfg(target_os = "android")]
    {
        return ANDROID_TOP_SAFE_AREA.load(std::sync::atomic::Ordering::Relaxed) as f32;
    }

    #[cfg(not(target_os = "android"))]
    0.0
}

pub(crate) fn sync_android_system_bars(dark_mode: bool, surface: iced::Color) {
    #[cfg(target_os = "android")]
    {
        let Some(app) = ANDROID_APP.get() else {
            return;
        };

        if let Err(error) = set_android_system_bars(app, dark_mode, surface) {
            eprintln!("Failed to update Android system bars: {error:?}");
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        let _ = (dark_mode, surface);
    }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: iced_winit::winit::platform::android::activity::AndroidApp) {
    let _ = ANDROID_APP.set(app.clone());
    set_android_top_safe_area(&app);
    iced_winit::set_android_app(app);

    if let Err(error) = run() {
        eprintln!("Rdict failed to start: {error:?}");
        std::process::exit(1);
    }
}

#[cfg(target_os = "android")]
fn set_android_top_safe_area(app: &iced_winit::winit::platform::android::activity::AndroidApp) {
    const FALLBACK_TOP_SAFE_AREA: u32 = 24;

    let top = android_status_bar_height(app).unwrap_or_else(|error| {
        eprintln!("Failed to read Android status bar height: {error:?}");
        FALLBACK_TOP_SAFE_AREA
    });

    ANDROID_TOP_SAFE_AREA.store(top, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(target_os = "android")]
fn android_system_font_paths() -> &'static [&'static str] {
    &[
        "/system/fonts/NotoSans-Regular.ttf",
        "/system/fonts/NotoSans-Bold.ttf",
        "/system/fonts/NotoSans-Regular.otf",
        "/system/fonts/NotoSansDisplay-Regular.ttf",
        "/system/fonts/NotoSerif-Regular.ttf",
        "/system/fonts/NotoSansSymbols-Regular-Subsetted.ttf",
        "/system/fonts/NotoSansSymbols2-Regular.ttf",
        "/system/fonts/Roboto-Regular.ttf",
        "/system/fonts/NotoSansCJK-Regular.ttc",
        "/system/fonts/NotoSansCJK-Bold.ttc",
        "/system/fonts/NotoSansSC-Regular.otf",
        "/system/fonts/NotoSansSC-Bold.otf",
        "/system/fonts/NotoSansJP-Regular.otf",
        "/system/fonts/NotoSansJP-Bold.otf",
        "/system/fonts/NotoSansKR-Regular.otf",
        "/system/fonts/NotoSansKR-Bold.otf",
        "/product/fonts/NotoSans-Regular.ttf",
        "/product/fonts/NotoSans-Bold.ttf",
        "/product/fonts/NotoSans-Regular.otf",
        "/product/fonts/NotoSansDisplay-Regular.ttf",
        "/product/fonts/NotoSerif-Regular.ttf",
        "/product/fonts/NotoSansSymbols-Regular-Subsetted.ttf",
        "/product/fonts/NotoSansSymbols2-Regular.ttf",
        "/product/fonts/Roboto-Regular.ttf",
        "/product/fonts/NotoSansCJK-Regular.ttc",
        "/product/fonts/NotoSansCJK-Bold.ttc",
        "/system_ext/fonts/NotoSans-Regular.ttf",
        "/system_ext/fonts/NotoSansDisplay-Regular.ttf",
        "/system_ext/fonts/NotoSerif-Regular.ttf",
        "/system_ext/fonts/NotoSansSymbols-Regular-Subsetted.ttf",
        "/system_ext/fonts/NotoSansSymbols2-Regular.ttf",
        "/system_ext/fonts/Roboto-Regular.ttf",
    ]
}

#[cfg(target_os = "android")]
fn android_status_bar_height(
    app: &iced_winit::winit::platform::android::activity::AndroidApp,
) -> jni::errors::Result<u32> {
    use jni::objects::{JObject, JValue};
    use jni::{JavaVM, jni_sig, jni_str};

    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    let activity_raw = app.activity_as_ptr() as jni::sys::jobject;

    vm.attach_current_thread(|env| -> jni::errors::Result<u32> {
        let activity = unsafe { env.as_cast_raw::<JObject>(&activity_raw)? };
        let resources = env
            .call_method(
                &activity,
                jni_str!("getResources"),
                jni_sig!("()Landroid/content/res/Resources;"),
                &[],
            )?
            .l()?;

        let name = env.new_string("status_bar_height")?;
        let def_type = env.new_string("dimen")?;
        let def_package = env.new_string("android")?;

        let resource_id = env
            .call_method(
                &resources,
                jni_str!("getIdentifier"),
                jni_sig!("(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)I"),
                &[
                    JValue::Object(&name),
                    JValue::Object(&def_type),
                    JValue::Object(&def_package),
                ],
            )?
            .i()?;

        if resource_id <= 0 {
            return Ok(24);
        }

        let status_bar_px = env
            .call_method(
                &resources,
                jni_str!("getDimensionPixelSize"),
                jni_sig!("(I)I"),
                &[JValue::Int(resource_id)],
            )?
            .i()?;

        let metrics = env
            .call_method(
                &resources,
                jni_str!("getDisplayMetrics"),
                jni_sig!("()Landroid/util/DisplayMetrics;"),
                &[],
            )?
            .l()?;
        let density = env
            .get_field(&metrics, jni_str!("density"), jni_sig!("F"))?
            .f()?;

        let logical_height = ((status_bar_px as f32) / density.max(1.0)).ceil();

        Ok(logical_height.max(24.0) as u32)
    })
}

#[cfg(target_os = "android")]
fn set_android_system_bars(
    app: &iced_winit::winit::platform::android::activity::AndroidApp,
    dark_mode: bool,
    surface: iced::Color,
) -> jni::errors::Result<()> {
    use jni::objects::{JObject, JValue};
    use jni::{JavaVM, jni_sig, jni_str};

    const SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR: i32 = 0x0000_0010;
    const SYSTEM_UI_FLAG_LIGHT_STATUS_BAR: i32 = 0x0000_2000;
    const APPEARANCE_LIGHT_STATUS_BARS: i32 = 0x0000_0008;
    const APPEARANCE_LIGHT_NAVIGATION_BARS: i32 = 0x0000_0010;

    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr().cast()) };
    let activity_raw = app.activity_as_ptr() as jni::sys::jobject;
    let color = android_color(surface);
    let light_bars = !dark_mode;

    vm.attach_current_thread(|env| -> jni::errors::Result<()> {
        let activity = unsafe { env.as_cast_raw::<JObject>(&activity_raw)? };
        let window = env
            .call_method(
                &activity,
                jni_str!("getWindow"),
                jni_sig!("()Landroid/view/Window;"),
                &[],
            )?
            .l()?;

        env.call_method(
            &window,
            jni_str!("setStatusBarColor"),
            jni_sig!("(I)V"),
            &[JValue::Int(color)],
        )?;
        env.call_method(
            &window,
            jni_str!("setNavigationBarColor"),
            jni_sig!("(I)V"),
            &[JValue::Int(color)],
        )?;

        let decor_view = env
            .call_method(
                &window,
                jni_str!("getDecorView"),
                jni_sig!("()Landroid/view/View;"),
                &[],
            )?
            .l()?;
        let current_visibility = env
            .call_method(
                &decor_view,
                jni_str!("getSystemUiVisibility"),
                jni_sig!("()I"),
                &[],
            )?
            .i()?;
        let light_flags = SYSTEM_UI_FLAG_LIGHT_STATUS_BAR | SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR;
        let visibility = if light_bars {
            current_visibility | light_flags
        } else {
            current_visibility & !light_flags
        };

        env.call_method(
            &decor_view,
            jni_str!("setSystemUiVisibility"),
            jni_sig!("(I)V"),
            &[JValue::Int(visibility)],
        )?;

        let sdk_int = env
            .get_static_field(
                jni_str!("android/os/Build$VERSION"),
                jni_str!("SDK_INT"),
                jni_sig!("I"),
            )?
            .i()?;

        if sdk_int >= 30 {
            let insets_controller = env
                .call_method(
                    &window,
                    jni_str!("getInsetsController"),
                    jni_sig!("()Landroid/view/WindowInsetsController;"),
                    &[],
                )?
                .l()?;

            if insets_controller.as_raw().is_null() {
                return Ok(());
            }

            let light_appearance = APPEARANCE_LIGHT_STATUS_BARS | APPEARANCE_LIGHT_NAVIGATION_BARS;
            let appearance = if light_bars { light_appearance } else { 0 };

            env.call_method(
                &insets_controller,
                jni_str!("setSystemBarsAppearance"),
                jni_sig!("(II)V"),
                &[JValue::Int(appearance), JValue::Int(light_appearance)],
            )?;
        }

        Ok(())
    })
}

#[cfg(target_os = "android")]
fn android_color(color: iced::Color) -> i32 {
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as i32;

    (channel(color.a) << 24) | (channel(color.r) << 16) | (channel(color.g) << 8) | channel(color.b)
}
