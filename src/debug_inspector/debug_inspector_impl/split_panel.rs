use std::iter;

use bevy::utils::default;
use bevy_egui::egui::{
    Align, Color32, Context, CursorIcon, Frame, Id, Layout, Margin, Rect, ScrollArea, Sense,
    SidePanel, Stroke, Ui, UiBuilder, panel::Side, pos2, vec2,
};

struct SplitPanelStyle {
    padding: Option<Margin>,
    border_stroke: Stroke,
    pane_min_size: f32,
    separator_width: f32,
    separator_interact_expand: f32,
    separator_color_default: Color32,
    separator_color_hover: Color32,
    separator_color_drag: Color32,
}

impl Default for SplitPanelStyle {
    fn default() -> Self {
        Self {
            padding: Some(Margin::same(8)),
            border_stroke: Stroke::new(0.0, Color32::BLACK),
            pane_min_size: 64.0,
            separator_width: 2.0,
            separator_interact_expand: 4.0,
            separator_color_default: Color32::from_gray(60),
            separator_color_hover: Color32::from_gray(240),
            separator_color_drag: Color32::WHITE,
        }
    }
}

pub struct SplitPanelState<Pane> {
    panes: Vec<PaneState<Pane>>,
}

impl<Pane> SplitPanelState<Pane> {
    pub fn equally_sized(panes: impl IntoIterator<Item = Pane>) -> Self {
        let mut panes = panes.into_iter().map(|p| PaneState::new(p, 0.0)).collect::<Vec<_>>();
        let frac = 1.0 / panes.len() as f32;
        panes.iter_mut().for_each(move |p| p.fraction = frac);

        let sum: f32 = panes.iter().map(|p| p.fraction).sum();
        assert!(
            (sum - 1.0).abs() <= f32::EPSILON,
            "The sum of the pane fractions in a SplitPanel must be 1.0"
        );

        Self { panes }
    }
}

struct PaneState<Pane> {
    rect: Rect,
    content_rect: Rect,
    fraction: f32,
    pane: Pane,
}

impl<Pane> PaneState<Pane> {
    fn new(pane: Pane, fraction: f32) -> Self {
        Self { pane, fraction, rect: Rect::ZERO, content_rect: Rect::ZERO }
    }
}

pub struct SplitPanel<'state, Pane> {
    id: Id,
    side: Side,
    default_width: f32,
    style: SplitPanelStyle,
    state: &'state mut SplitPanelState<Pane>,
    separator_rects: Vec<Rect>,
}

#[allow(dead_code)]
impl<'state, Pane> SplitPanel<'state, Pane> {
    fn new(id: impl Into<Id>, side: Side, state: &'state mut SplitPanelState<Pane>) -> Self {
        let n_panes = state.panes.len();
        let n_separators = n_panes - 1;

        Self {
            id: id.into(),
            side,
            default_width: 200.0,
            style: SplitPanelStyle::default(),
            state,
            separator_rects: iter::repeat_n(Rect::ZERO, n_separators).collect(),
        }
    }

    pub fn left(id: impl Into<Id>, state: &'state mut SplitPanelState<Pane>) -> Self {
        Self::new(id, Side::Left, state)
    }

    pub fn right(id: impl Into<Id>, state: &'state mut SplitPanelState<Pane>) -> Self {
        Self::new(id, Side::Right, state)
    }
}

impl<'state, Pane> SplitPanel<'state, Pane> {
    pub fn default_width(mut self, default_width: f32) -> Self {
        self.default_width = default_width;
        self
    }
}

impl<'state, Pane> SplitPanel<'state, Pane> {
    pub fn show(mut self, ctx: &Context, pane_viewer: &mut impl PaneViewer<Pane = Pane>) {
        let frame =
            Frame { inner_margin: Margin::ZERO, fill: ctx.style().visuals.panel_fill, ..default() };
        SidePanel::new(self.side, self.id).frame(frame).default_width(self.default_width).show(
            ctx,
            |ui| {
                let mut panel_rect = ui.available_rect_before_wrap();

                ui.expand_to_include_rect(panel_rect);

                let panel_border = ui.visuals().widgets.active.fg_stroke.width;
                panel_rect.max.x -= panel_border;
                panel_rect.min.x += panel_border;
                ui.set_clip_rect(panel_rect);

                let content_rect = panel_rect.expand(-self.style.border_stroke.width / 2.0);

                ui.allocate_rect(content_rect, Sense::hover());

                let layout = Layout::top_down_justified(Align::Min);
                let ui = &mut ui.new_child(UiBuilder::new().max_rect(content_rect).layout(layout));

                self.compute_rects(content_rect);

                self.show_panes(ui, pane_viewer);

                self.show_separators(ui, content_rect);
            },
        );
    }

    fn compute_rects(&mut self, content_rect: Rect) {
        let n_panes = self.state.panes.len();
        let n_separators = n_panes - 1;

        let padding = self.style.padding.unwrap_or_default();
        let contiguous_pane_height =
            content_rect.height() - n_separators as f32 * self.style.separator_width;

        self.state.panes.iter_mut().enumerate().fold(0.0, |last_y, (i, pane)| {
            pane.rect = Rect {
                min: pos2(content_rect.min.x, last_y),
                max: pos2(content_rect.max.x, last_y + pane.fraction * contiguous_pane_height),
            };

            pane.content_rect = Rect {
                min: pane.rect.min + padding.left_top(),
                max: pane.rect.max - padding.right_bottom(),
            };

            if i < n_panes - 1 {
                self.separator_rects[i] = Rect {
                    min: pos2(content_rect.min.x, pane.rect.max.y),
                    max: pos2(content_rect.max.x, pane.rect.max.y + self.style.separator_width),
                };
                self.separator_rects[i].max.y
            } else {
                pane.rect.max.y
            }
        });
    }

    fn show_panes(&mut self, ui: &mut Ui, pane_viewer: &mut impl PaneViewer<Pane = Pane>) {
        for (i, pane) in self.state.panes.iter_mut().enumerate() {
            let layout = Layout::top_down(Align::Min);
            let ui = &mut ui.new_child(UiBuilder::new().max_rect(pane.content_rect).layout(layout));
            ui.set_clip_rect(pane.rect);
            ScrollArea::both().id_salt(("split_panel", "pane", i)).show(ui, |ui| {
                ui.expand_to_include_rect(ui.available_rect_before_wrap());
                pane_viewer.ui(ui, &mut pane.pane);
            });
        }
    }

    fn show_separators(&mut self, ui: &mut Ui, content_rect: Rect) {
        let n_separators = self.state.panes.len() - 1;
        let content_range = (content_rect.max.y - content_rect.min.y)
            - n_separators as f32 * self.style.separator_width;
        let min_fraction = (self.style.pane_min_size / content_range).min(1.0);

        for (i, rect) in self.separator_rects.iter().copied().enumerate() {
            let interact_expand = vec2(0.0, self.style.separator_interact_expand);
            let interact_rect = rect.expand2(interact_expand);

            let response = ui
                .allocate_rect(interact_rect, Sense::click_and_drag())
                .on_hover_and_drag_cursor(CursorIcon::ResizeVertical);

            let color = if response.dragged() {
                self.style.separator_color_drag
            } else if response.hovered() {
                self.style.separator_color_hover
            } else {
                self.style.separator_color_default
            };
            ui.painter().rect_filled(rect, 0.0, color);

            if response.double_clicked() {
                let fraction_1 = self.state.panes[i].fraction;
                let fraction_2 = self.state.panes[i + 1].fraction;
                let avg_fraction = (fraction_1 + fraction_2) / 2.0;
                self.state.panes[i].fraction = avg_fraction;
                self.state.panes[i + 1].fraction = avg_fraction;
            } else if response.interact_pointer_pos().is_some() {
                let delta = response.drag_delta().y;
                let orig_pane_above_frac = self.state.panes[i].fraction;
                let orig_pane_below_frac = self.state.panes[i + 1].fraction;
                let max_fraction = orig_pane_above_frac + orig_pane_below_frac - min_fraction;

                let new_pane_above_frac = (orig_pane_above_frac + delta / content_range)
                    .max(min_fraction)
                    .min(max_fraction);

                self.state.panes[i].fraction = new_pane_above_frac;
                self.state.panes[i + 1].fraction -= new_pane_above_frac - orig_pane_above_frac;
            }
        }
    }
}

pub trait PaneViewer {
    type Pane;

    fn ui(&mut self, ui: &mut Ui, pane: &mut Self::Pane);
}
