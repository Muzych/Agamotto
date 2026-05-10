use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Window, WindowBounds,
    WindowOptions,
};

/// 🔮 Agamotto — the Eye that sees through all code illusions.
/// A GPU-accelerated visual interface for coding agents, powered by gpui.
struct AgamottoApp;

impl Render for AgamottoApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0x0d1117))
            .size_full()
            .text_color(rgb(0xc9d1d9))
            .font_family("JetBrains Mono")
            // --- titlebar ---
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .h_8()
                    .bg(rgb(0x161b22))
                    .border_b_1()
                    .border_color(rgb(0x30363d))
                    .text_sm()
                    .child("🔮 Agamotto"),
            )
            // --- main area ---
            .child(
                div()
                    .flex()
                    .flex_1()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_4()
                    .child("Reality is but an illusion...")
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x8b949e))
                            .child("— Strange Supreme"),
                    ),
            )
            // --- input bar ---
            .child(
                div()
                    .flex()
                    .h_10()
                    .border_t_1()
                    .border_color(rgb(0x30363d))
                    .bg(rgb(0x161b22))
                    .px_2()
                    .items_center()
                    .text_sm()
                    .text_color(rgb(0x8b949e))
                    .child("> _"),
            )
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(900.0), px(650.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| AgamottoApp),
        )
        .unwrap();
        cx.activate(true);
    });
}
