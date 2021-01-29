use ggez::{Context, GameResult as GuiResult, input, GameError, event};
use ggez::event::{EventHandler, MouseButton};
use ggez::graphics::{self, Rect, Color, DrawMode, DrawParam, BLACK, Align, Text};
use std::collections::HashSet;

pub struct MapMenu {
    controls: Vec<Control>,
    choice: Option<usize>,
    label: String,
}

impl MapMenu {
    pub fn new() -> Self {
        Self {
            controls: vec![
                Control::new_button(
                    Rect::new(425.0, 150.0, 150.0, 50.0),
                    "The Good".to_string(),
                    Color::new(0.0, 0.0, 1.0, 1.0),
                    Color::new(0.7, 0.0, 1.0, 1.0),
                    |menu| {
                        menu.controls.remove(0);
                        menu.label = "Nope.\nWe never made that one.".to_string();
                    },
                ),
                Control::new_button(
                    Rect::new(425.0, 225.0, 150.0, 50.0),
                    "The Bad".to_string(),
                    Color::new(0.0, 0.0, 1.0, 1.0),
                    Color::new(0.7, 0.0, 1.0, 1.0),
                    |menu| { menu.choice = Some(1) },
                ),
                Control::new_button(
                    Rect::new(425.0, 300.0, 150.0, 50.0),
                    "The Ugly".to_string(),
                    Color::new(0.0, 0.0, 1.0, 1.0),
                    Color::new(0.7, 0.0, 1.0, 1.0),
                    |menu| { menu.choice = Some(2) },
                )
            ],
            choice: None,
            label: "Select a map".to_string(),
        }
    }

    pub fn choice(&self) -> Option<usize> {
        self.choice.clone()
    }
}

impl EventHandler for MapMenu {
    fn update(&mut self, ctx: &mut Context) -> GuiResult<()> {
        if self.choice.is_some() {
            event::quit(ctx);
        }
        Ok(())

    }

    fn draw(&mut self, ctx: &mut Context) -> GuiResult<()> {
        graphics::clear(ctx, BLACK);
        let label = Text::new(self.label.clone());
        let label_width = label.width(ctx) as f32;
        graphics::draw(ctx, &label, DrawParam::default().dest([500.0 - label_width * 0.5, 100.0]));
        for control in &mut self.controls {
            control.draw(ctx)?;
        }
        graphics::present(ctx)
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            for control in &mut self.controls {
                if control.region.contains([x, y]) {
                    control.on_mouse_down();
                }
            }
        }
    }

    fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            let mut scripts = Vec::new();
            for i in 0..self.controls.len() {
                if self.controls[i].region.contains([x, y]) {
                    self.controls[i].on_mouse_up();
                    let control = &mut self.controls[i];
                    scripts.push(control.on_action_script)
                }
            }
            for script in scripts {
                (script)(self);
            }
        }
    }
}

trait GuiHandler {
    fn draw(&mut self, ctx: &mut Context) -> GuiResult<()>;
    fn on_mouse_down(&mut self);
    fn on_mouse_up(&mut self);
}

struct Control {
    region: Rect,
    color: Color,
    control_kind: ControlKind,
    hovered: bool,
    on_action_script: fn(&mut MapMenu),
}

impl Control {
    fn new_button(region: Rect, text: String, primary_color: Color, hover_color: Color, on_click: fn(&mut MapMenu)) -> Self {
        Self {
            region,
            color: primary_color,
            control_kind: ControlKind::Button(text, hover_color),
            hovered: false,
            on_action_script: on_click,
        }
    }

    fn on_action(&self) -> fn(&mut MapMenu) {
        self.on_action_script
    }
}

enum ControlKind {
    Button(String, Color)
}

impl GuiHandler for Control {
    fn draw(&mut self, ctx: &mut Context) -> GuiResult<()> {
        let rect = graphics::Mesh::new_rectangle(ctx, DrawMode::fill(), self.region, self.color)?;
        graphics::draw(ctx, &rect, DrawParam::default());
        match &self.control_kind {
            ControlKind::Button(text, ..) => {
                let mut text = graphics::Text::new(text.clone());
                text.set_bounds([self.region.w, self.region.h], Align::Center);
                graphics::draw(ctx, &text, DrawParam::default().dest([
                    self.region.x,
                    self.region.y + self.region.h * 0.5
                ]))?;
            }
        }
        Ok(())
    }

    fn on_mouse_down(&mut self) {
        // They're holding the mouse down, swap the colors!
        if let ControlKind::Button(.., click_color) = &mut self.control_kind {
            std::mem::swap(&mut self.color, click_color);
        }
    }

    fn on_mouse_up(&mut self) {
        // They've released the mouse, swap those colors back!
        if let ControlKind::Button(_, click_color) = &mut self.control_kind {
            std::mem::swap(&mut self.color, click_color);
        }
    }
}