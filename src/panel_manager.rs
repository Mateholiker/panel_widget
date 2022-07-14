use std::{
    cmp::max,
    ops::{Deref, DerefMut},
};

use eframe::{
    egui::{style::Margin, Frame, Ui},
    epaint::Color32,
};

use crate::{
    error::{PanelError, PanelResult},
    panel::{Alignment, Panel, Share},
};

///An Element in a Panel list. The Panels get added in order of the list
struct PanelEntry {
    ///the panel itself
    panel: Panel,
    ///how much width_share is taken by all the Panels after this one
    total_width_share: Share,
    ///how much hight_share is taken by all the Panels after this one
    total_height_share: Share,

    ///how much width_share is taken by all the Panels after this one after everyone, that can be drawn, has their min_width
    total_remaining_width_share: Share,
    ///how much height_share is taken by all the Panels after this one after everyone, that can be drawn, has their min_height
    total_remaining_height_share: Share,

    ///how much width_share we still want after we got our min_width
    remaining_width_share: Share,
    ///how much height_share we still want after we got our min_height
    remaining_height_share: Share,
}

impl Deref for PanelEntry {
    type Target = Panel;

    fn deref(&self) -> &Self::Target {
        &self.panel
    }
}

impl DerefMut for PanelEntry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.panel
    }
}

pub struct PanelManager {
    panels: Vec<PanelEntry>,
    frame: Frame,
}

impl PanelManager {
    pub fn show_inside(&mut self, ui: &mut Ui) {}

    /// calculate spacing for all panels that fit on the screen and returns the index
    /// to the last panel that could fit
    fn calculate_spacing(&mut self, ui: &mut Ui) -> PanelResult<usize> {
        let info_0 = self.calculate_info_0(ui);
        let info_1 = self.calculate_info_1(info_0, ui)?;
        self.calcultae_shares(info_1);
        self.calculate_min_distribution_and_remaining_shares(info_0, info_1);

        let Info1 {
            last_panel_index,
            remaining_width,
            remaining_height,
        } = info_1;

        for PanelEntry {
            ref mut panel,
            ref total_remaining_width_share,
            ref total_remaining_height_share,
            ref remaining_width_share,
            ref remaining_height_share,
            ..
        } in self
            .panels
            .iter_mut()
            .filter(|p| p.active())
            .rev()
            .take(last_panel_index + 1)
        {
            let additional_width = (*remaining_width_share as f32
                / *total_remaining_width_share as f32)
                * remaining_width;

            panel.width = panel.min_width(self.frame) + additional_width;

            let additional_height = (*remaining_height_share as f32
                / *total_remaining_height_share as f32)
                * remaining_height;

            panel.height = panel.min_height(self.frame) + additional_height;
        }

        Ok(())
    }

    fn calculate_min_distribution_and_remaining_shares(&mut self, info_0: Info0, info_1: Info1) {
        let Info0 {
            available_width,
            available_height,
        } = info_0;

        let Info1 {
            last_panel_index,
            remaining_width,
            remaining_height,
        } = info_1;

        let used_width = available_width - remaining_width;
        let used_height = available_height - remaining_height;

        //let mut remaining_height = available_height;
        //let mut remaining_width = available_width;

        for PanelEntry {
            ref panel,
            ref total_width_share,
            ref total_height_share,
            ref mut remaining_width_share,
            ref mut remaining_height_share,
            ..
        } in self
            .panels
            .iter_mut()
            .take(last_panel_index + 1)
            .filter(|p| p.active())
        {
            let width_by_share = ((panel.width_share() as u64 * remaining_width as u64)
                / *total_width_share as u64) as u32;
            let height_by_share = ((panel.height_share() as u64 * remaining_height as u64)
                / *total_height_share as u64) as u32;

            *remaining_width_share = if width_by_share < panel.min_width(self.frame) {
                0
            } else {
                let remaining_wanted_width = width_by_share - panel.min_width(self.frame);
                ((remaining_wanted_width as u64 * *total_width_share as u64)
                    / available_width as u64) as u32
            };

            *remaining_height_share = if height_by_share < panel.min_height(self.frame) {
                0
            } else {
                let remaining_wanted_height = height_by_share - panel.min_height(self.frame);
                ((remaining_wanted_height as u64 * *total_height_share as u64)
                    / available_height as u64) as u32
            };
        }

        let mut total_remaining_width_share_counter = 0;
        let mut total_remaining_height_share_counter = 0;

        let mut i = last_panel_index + 1;
        while let Some(PanelEntry {
            ref panel,
            ref remaining_height_share,
            ref remaining_width_share,
            ref mut total_remaining_width_share,
            ref mut total_remaining_height_share,
            ..
        }) = self.panels.get(i)
        {
            i -= 1;
            if !panel.active() {
                continue;
            }

            match panel.alignment() {
                Alignment::Top | Alignment::Bottom => {
                    total_remaining_height_share_counter += *remaining_height_share;
                    total_remaining_width_share_counter =
                        max(total_remaining_width_share_counter, *remaining_width_share);
                }
                Alignment::Left | Alignment::Right => {
                    total_remaining_height_share_counter = max(
                        total_remaining_height_share_counter,
                        *remaining_height_share,
                    );
                    total_remaining_width_share_counter += *remaining_width_share;
                }
            }

            *total_remaining_width_share = total_remaining_width_share_counter;
            *total_remaining_height_share = total_remaining_height_share_counter;
        }
    }

    fn calcultae_shares(&mut self, info_1: Info1) {
        let last_panel_index = info_1.last_panel_index;

        let mut total_width_share_counter = 0;
        let mut total_height_share_counter = 0;

        let mut i = last_panel_index + 1;
        while let Some(PanelEntry {
            ref panel,
            ref mut total_width_share,
            ref mut total_height_share,
            ..
        }) = self.panels.get(i)
        {
            i -= 1;
            if !panel.active() {
                continue;
            }
            match panel.alignment() {
                Alignment::Top | Alignment::Bottom => {
                    total_height_share_counter += panel.height_share();
                    total_width_share_counter = max(total_width_share_counter, panel.width_share());
                }
                Alignment::Left | Alignment::Right => {
                    total_height_share_counter =
                        max(total_height_share_counter, panel.height_share());
                    total_width_share_counter += panel.width_share();
                }
            }
            *total_width_share = total_width_share_counter;
            *total_height_share = total_height_share_counter;
        }
    }

    /// calculates the index of the last panel that can be drawn if every panel takes its min_width and min_height
    /// it returns the remaining width and height after we draw the panels that can be drawn
    fn calculate_info_1(&mut self, info_0: Info0, ui: &mut Ui) -> PanelResult<Info1> {
        let mut remaining_height = info_0.available_height;
        let mut remaining_width = info_0.available_width;

        let mut first_top_bottom = true;
        let mut first_left_right = true;
        let mut last_panel_index = None;
        // we iterate through all panels and check if they can fit. If they can we reduce the remaining size
        // and go to the next panel
        'panel_loop: for (i, panel) in self.panels.iter().enumerate().filter(|(i, p)| p.active()) {
            match panel.alignment() {
                Alignment::Top | Alignment::Bottom => {
                    // check if the panel fits
                    // if it is not the first panel in this dimension we need to include spacing between the panels
                    if remaining_width < panel.min_width(self.frame)
                        || remaining_height < panel.min_height(self.frame)
                        || (!first_top_bottom
                            && remaining_height
                                < panel.min_height(self.frame)
                                    + ui.style().spacing.item_spacing.y as u32)
                    {
                        // if it dose not fit we end the loop early
                        break 'panel_loop;
                    } else {
                        // if it fit we reduce the remaining space
                        remaining_height -= panel.min_height(self.frame);
                        if !first_top_bottom {
                            remaining_height -= ui.style().spacing.item_spacing.y as u32;
                        }
                        first_top_bottom = false
                    }
                }
                Alignment::Left | Alignment::Right => {
                    // check if the panel fits
                    // if it is not the first panel in this dimension we need to include spacing between the panels
                    if remaining_height < panel.min_height(self.frame)
                        || remaining_width < panel.min_width(self.frame)
                        || (!first_left_right
                            && remaining_width
                                < panel.min_width(self.frame)
                                    + ui.style().spacing.item_spacing.x as u32)
                    {
                        // if it dose not fit we end the loop early
                        break 'panel_loop;
                    } else {
                        // if it fit we reduce the remaining space
                        remaining_width -= panel.min_width(self.frame);
                        if !first_left_right {
                            remaining_width -= ui.style().spacing.item_spacing.x as u32;
                        }
                        first_left_right = false
                    }
                }
            }
            // we need to renember the last panel that we could fit
            last_panel_index = Some(i);
        }

        let last_panel_index = last_panel_index.ok_or(PanelError::NoPanelCanBeDrawn)?;
        Ok(Info1 {
            last_panel_index,
            remaining_width,
            remaining_height,
        })
    }

    fn calculate_info_0(&self, ui: &Ui) -> Info0 {
        Info0 {
            available_width: ui.available_width() as u32,
            available_height: ui.available_height() as u32,
        }
    }

    fn default_frame(ui: &mut Ui) -> Frame {
        Frame::window(ui.style())
            .rounding(10.0)
            .stroke((4.0, Color32::DARK_BLUE).into())
            .outer_margin(Margin::same(4.0))
    }
}

#[derive(Clone, Copy)]
struct Info0 {
    available_width: u32,
    available_height: u32,
}

#[derive(Clone, Copy)]
struct Info1 {
    ///the index of the last panel that fits in the gui after we filtered the non active ones
    last_panel_index: usize,
    ///the width that remains after every panel that can be drawn acording to the last_panel_index has taken its min_width
    remaining_width: u32,
    ///the height that remains after every panel that can be drawn acording to the last_panel_index has taken its min_height
    remaining_height: u32,
}
