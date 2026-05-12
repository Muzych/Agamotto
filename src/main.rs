// =============================================================================
// 🔮 Agamotto — pi-agent GUI
// A GPU-accelerated desktop GUI for the pi coding agent, powered by gpui-component.
// =============================================================================

use gpui::{
    div, prelude::*, px, rgb, size, AnyElement, Context, Entity,
    IntoElement, ParentElement, Render, SharedString,
    Styled, Window, WindowBounds, WindowOptions,
};
use gpui_component::{
    button::{Button, ButtonVariants},
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    sidebar::{
        Sidebar, SidebarCollapsible, SidebarFooter, SidebarGroup, SidebarHeader, SidebarMenu,
        SidebarMenuItem,
    },
    text,
    ActiveTheme, Disableable, Icon, IconName, Root, StyledExt,
    h_flex, v_flex,
};
use gpui_component_assets::Assets;
use smol::process::Command;

// ---------------------------------------------------------------------------
// Data models
// ---------------------------------------------------------------------------

#[derive(Clone, PartialEq)]
enum Role {
    User,
    Assistant,
    System,
}

#[derive(Clone)]
struct Message {
    role: Role,
    content: String,
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

struct AgamottoApp {
    messages: Vec<Message>,
    input_state: Entity<InputState>,
    is_loading: bool,
    pi_available: bool,
    active_session: SharedString,
    sessions: Vec<SharedString>,
    _subscriptions: Vec<gpui::Subscription>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn check_pi() -> bool {
    std::process::Command::new("which")
        .arg("pi")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// AgamottoApp impl
// ---------------------------------------------------------------------------

impl AgamottoApp {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            InputState::new(window, cx).placeholder("Ask pi-agent... (Enter to send)")
        });

        let pi_available = check_pi();

        let mut messages = Vec::new();
        if pi_available {
            messages.push(Message {
                role: Role::System,
                content: concat!(
                    "🔮 **Agamotto pi-agent GUI** ready.\n\n",
                    "Type your question below and press **Enter** to send. ",
                    "pi-agent will respond with code, explanations, and tool results."
                )
                .into(),
            });
        } else {
            messages.push(Message {
                role: Role::System,
                content: concat!(
                    "⚠️ **pi-agent not found** on your PATH.\n\n",
                    "Install it with:\n\n```bash\nnpm install -g @earendil-works/pi-coding-agent\n```\n\n",
                    "Then restart Agamotto."
                )
                .into(),
            });
        }

        let sessions = vec![
            SharedString::from("🔮 Default"),
            SharedString::from("🐛 Bug fixes"),
            SharedString::from("📦 Feature: auth"),
        ];

        let _subscriptions = vec![cx.subscribe_in(
            &input_state,
            window,
            {
                let input_state = input_state.clone();
                move |this: &mut AgamottoApp,
                      _,
                      ev: &InputEvent,
                      window: &mut Window,
                      cx: &mut Context<AgamottoApp>| {
                    if matches!(ev, InputEvent::PressEnter { .. }) {
                        let value = input_state.read(cx).value().to_string().trim().to_string();
                        if !value.is_empty() {
                            this.send_message(value, window, cx);
                        }
                    }
                }
            },
        )];

        Self {
            messages,
            input_state,
            is_loading: false,
            pi_available,
            active_session: "🔮 Default".into(),
            sessions,
            _subscriptions,
        }
    }

    fn send_message(&mut self, content: String, window: &mut Window, cx: &mut Context<Self>) {
        // Clear input
        self.input_state.update(cx, |state, cx| {
            state.set_value("", window, cx);
        });

        // Add user message
        self.messages.push(Message {
            role: Role::User,
            content: content.clone(),
        });

        // Loading placeholder
        self.messages.push(Message {
            role: Role::Assistant,
            content: "⏳ Thinking...".into(),
        });

        self.is_loading = true;
        cx.notify();

        // Spawn pi in background
        let content = content.clone();
        cx.spawn(async move |this, cx| {
            let result = ask_pi(content).await;
            let _ = this.update(cx, move |this: &mut AgamottoApp, cx| {
                let loading_text = "⏳ Thinking...";
                if let Some(msg) = this.messages.iter_mut().find(|m| m.content == loading_text) {
                    match result {
                        Ok(response) => {
                            msg.content = response;
                        }
                        Err(err) => {
                            msg.content = format!(
                                "❌ **Error running pi-agent**\n\n```\n{}\n```\n\n\
                                 Make sure `pi` is installed and your API key is set.",
                                err
                            );
                            msg.role = Role::System;
                        }
                    }
                }
                this.is_loading = false;
                cx.notify();
            });
        }).detach();
    }

    /// Called from the Send button click — reads input and sends.
    fn submit_from_button(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let value = self.input_state.read(cx).value().to_string().trim().to_string();
        if !value.is_empty() {
            self.send_message(value, window, cx);
        }
    }

    // -----------------------------------------------------------------------
    // Render sections
    // -----------------------------------------------------------------------

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        Sidebar::new("agamotto-sidebar")
            .collapsible(SidebarCollapsible::Icon)
            .collapsed(false)
            .w(px(220.))
            .header(
                SidebarHeader::new().child(
                    h_flex()
                        .items_center()
                        .gap_2()
                        .px_2()
                        .child(Icon::new(IconName::Bot))
                        .child(
                            v_flex()
                                .flex_1()
                                .overflow_hidden()
                                .child(div().font_bold().text_sm().child("Agamotto"))
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("pi-agent GUI"),
                                ),
                        ),
                ),
            )
            .child(
                SidebarGroup::new("Sessions").child(
                    SidebarMenu::new().children(self.sessions.iter().map(|session| {
                        let active = *session == self.active_session;
                        SidebarMenuItem::new(session.clone())
                            .icon(IconName::Bell)
                            .active(active)
                    })),
                ),
            )
            .footer(
                SidebarFooter::new().child(
                    h_flex()
                        .gap_2()
                        .px_2()
                        .text_sm()
                        .text_color(cx.theme().muted_foreground)
                        .child(Icon::new(IconName::Plus))
                        .child("New Session"),
                ),
            )
    }

    fn render_messages(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .flex_1()
            .min_h_0()
            .bg(cx.theme().background)
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .gap_4()
                    .p_4()
                    .children(self.messages.iter().map(|msg| render_message(msg, cx))),
            )
    }

    fn render_input(
        &self,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .px_4()
            .py_3()
            .gap_3()
            .items_center()
            .child(div().flex_1().child(Input::new(&self.input_state)))
            .child(
                Button::new("send-btn")
                    .primary()
                    .label(if self.is_loading { "⏳" } else { "Send" })
                    .disabled(self.is_loading)
                    .on_click(cx.listener(|this: &mut AgamottoApp, _, window, cx| {
                        this.submit_from_button(window, cx);
                    })),
            )
    }

    fn render_status(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .h_7()
            .items_center()
            .px_4()
            .gap_4()
            .bg(cx.theme().secondary)
            .border_t_1()
            .border_color(cx.theme().border)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(format!("📋 Session: {}", self.active_session))
            .child(format!("💬 Messages: {}", self.messages.len()))
            .child(format!(
                "🔌 pi-agent: {}",
                if self.pi_available {
                    if self.is_loading {
                        "⏳ running..."
                    } else {
                        "✅ ready"
                    }
                } else {
                    "❌ not found"
                }
            ))
    }
}

// ---------------------------------------------------------------------------
// Render impl
// ---------------------------------------------------------------------------

impl Render for AgamottoApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .bg(cx.theme().background)
            // ── Title Bar ──
            .child(
                h_flex()
                    .h_8()
                    .items_center()
                    .px_4()
                    .gap_2()
                    .bg(cx.theme().title_bar)
                    .border_b_1()
                    .border_color(cx.theme().title_bar_border)
                    .child(Icon::new(IconName::Bot))
                    .child(div().font_bold().text_sm().child("Agamotto"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("pi-agent GUI"),
                    ),
            )
            // ── Main Content ──
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.render_sidebar(cx))
                    .child(div().w(px(1.)).bg(cx.theme().border).h_full())
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(self.render_messages(window, cx))
                            .child(self.render_input(cx)),
                    ),
            )
            // ── Status Bar ──
            .child(self.render_status(cx))
    }
}

// ---------------------------------------------------------------------------
// Message bubble rendering
// ---------------------------------------------------------------------------

fn render_message(msg: &Message, cx: &mut Context<AgamottoApp>) -> AnyElement {
    match msg.role {
        Role::User => render_user_bubble(msg, cx),
        Role::Assistant => render_assistant_bubble(msg, cx),
        Role::System => render_system_bubble(msg, cx),
    }
}

fn render_user_bubble(msg: &Message, cx: &mut Context<AgamottoApp>) -> AnyElement {
    h_flex()
        .justify_end()
        .child(
            v_flex()
                .max_w(px(600.))
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("You"),
                )
                .child(
                    div()
                        .rounded_lg()
                        .bg(cx.theme().primary)
                        .text_color(rgb(0xffffff))
                        .px_4()
                        .py_2()
                        .text_sm()
                        .child(msg.content.clone()),
                ),
        )
        .into_any_element()
}

fn render_assistant_bubble(msg: &Message, cx: &mut Context<AgamottoApp>) -> AnyElement {
    h_flex()
        .justify_start()
        .child(
            v_flex()
                .max_w(px(600.))
                .gap_1()
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child("🤖 pi-agent"),
                )
                .child(
                    div()
                        .rounded_lg()
                        .bg(cx.theme().secondary)
                        .text_color(cx.theme().foreground)
                        .px_4()
                        .py_3()
                        .text_sm()
                        .w_full()
                        .child(text::markdown(msg.content.clone())),
                ),
        )
        .into_any_element()
}

fn render_system_bubble(msg: &Message, cx: &mut Context<AgamottoApp>) -> AnyElement {
    h_flex()
        .justify_center()
        .w_full()
        .child(
            div()
                .max_w(px(500.))
                .rounded_lg()
                .bg(cx.theme().secondary)
                .border_1()
                .border_color(cx.theme().border)
                .px_4()
                .py_3()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(text::markdown(msg.content.clone())),
        )
        .into_any_element()
}

// ---------------------------------------------------------------------------
// pi-agent subprocess
// ---------------------------------------------------------------------------

async fn ask_pi(prompt: String) -> Result<String, String> {
    let output = Command::new("pi")
        .arg("-p")
        .arg(prompt)
        .output()
        .await
        .map_err(|e| format!("Failed to spawn pi: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let mut result = String::new();
        if !stderr.is_empty() {
            result.push_str(&format!("_Stderr:_\n```\n{}\n```\n\n", stderr.trim()));
        }
        result.push_str(stdout.trim());

        if result.is_empty() {
            Ok("_(No output from pi-agent)_".into())
        } else {
            Ok(result)
        }
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(if stderr.is_empty() {
            format!("pi exited with status: {}", output.status)
        } else {
            stderr.trim().to_string()
        })
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let app = gpui_platform::application().with_assets(Assets);

    app.run(move |cx| {
        gpui_component::init(cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(
                size(px(1000.), px(700.)),
                cx,
            )),
            ..Default::default()
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let view = cx.new(|cx| AgamottoApp::new(window, cx));
                cx.new(|cx| Root::new(view, window, cx))
            })
            .expect("Failed to open window");
        })
        .detach();
    });
}
