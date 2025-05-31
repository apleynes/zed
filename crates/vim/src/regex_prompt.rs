use editor::{Editor, EditorEvent};
use futures::channel::oneshot;
use gpui::{
    actions, Context, DismissEvent, Entity, EventEmitter, FocusHandle, Focusable, Render,
    Window, prelude::*,
};
use ui::{prelude::*, Button, ButtonStyle, Label, v_flex};
use workspace::{ModalView, Workspace};

actions!(regex_prompt, [Confirm, Dismiss]);

pub struct RegexPrompt {
    editor: Entity<Editor>,
    prompt_message: String,
    focus_handle: FocusHandle,
    tx: Option<oneshot::Sender<Option<String>>>,
}

impl RegexPrompt {
    pub fn new(
        prompt_message: String,
        tx: oneshot::Sender<Option<String>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let editor = cx.new(|cx| {
            let mut editor = Editor::single_line(window, cx);
            editor.set_placeholder_text("Enter regex pattern...", cx);
            editor
        });

        cx.subscribe(&editor, |_, _editor, _event: &EditorEvent, _cx| {
            // Handle editor events if needed
        })
        .detach();

        Self {
            editor,
            prompt_message,
            focus_handle: cx.focus_handle(),
            tx: Some(tx),
        }
    }

    pub fn prompt_for_regex(
        workspace: &mut Workspace,
        prompt_message: String,
        window: &mut Window,
        cx: &mut Context<Workspace>,
    ) -> oneshot::Receiver<Option<String>> {
        let (tx, rx) = oneshot::channel();
        
        workspace.toggle_modal(window, cx, |window, cx| {
            RegexPrompt::new(prompt_message, tx, window, cx)
        });
        
        rx
    }

    fn confirm(&mut self, _action: &Confirm, _window: &mut Window, cx: &mut Context<Self>) {
        let text = self.editor.read(cx).text(cx);
        let regex_pattern = if text.trim().is_empty() {
            None
        } else {
            Some(text)
        };

        if let Some(tx) = self.tx.take() {
            tx.send(regex_pattern).ok();
        }
        cx.emit(DismissEvent);
    }

    fn dismiss(&mut self, _action: &Dismiss, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tx) = self.tx.take() {
            tx.send(None).ok();
        }
        cx.emit(DismissEvent);
    }
}

impl EventEmitter<DismissEvent> for RegexPrompt {}

impl Focusable for RegexPrompt {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl ModalView for RegexPrompt {}

impl Render for RegexPrompt {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();

        v_flex()
            .key_context("RegexPrompt")
            .on_action(cx.listener(Self::confirm))
            .on_action(cx.listener(Self::dismiss))
            .track_focus(&focus_handle)
            .elevation_3(cx)
            .w(rems(34.))
            .child(
                v_flex()
                    .gap_2()
                    .p_4()
                    .child(
                        Label::new(self.prompt_message.clone())
                            .size(LabelSize::Default)
                    )
                    .child(
                        self.editor.clone()
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_end()
                            .child(
                                Button::new("cancel", "Cancel")
                                    .style(ButtonStyle::Filled)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.dismiss(&Dismiss, window, cx);
                                    }))
                            )
                            .child(
                                Button::new("confirm", "Confirm")
                                    .style(ButtonStyle::Filled)
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.confirm(&Confirm, window, cx);
                                    }))
                            )
                    )
            )
    }
}