use eframe::egui::Frame;


pub type Share = u32;

#[derive(Clone, Copy)]
pub enum Alignment {
    Top,
    Bottom,
    Left,
    Right,
}

pub struct Panel {
    width: u32,
    height: u32,

    min_width: u32,
    min_height: u32,

    width_share: Share,
    height_share: Share,

    alignment: Alignment,
    active: bool,
}

impl Panel {
    /// Get the panel's min width.
    pub fn min_width(&self, frame: Frame) -> u32 {
        self.min_width
            + frame.inner_margin.left as u32
            + frame.outer_margin.left as u32
            + frame.inner_margin.right as u32
            + frame.outer_margin.right as u32
    }

    /// Get the panel's min height.
    pub fn min_height(&self, frame: Frame) -> u32 {
        self.min_height
            + frame.inner_margin.top as u32
            + frame.outer_margin.top as u32
            + frame.inner_margin.bottom as u32
            + frame.outer_margin.bottom as u32
    }

    /// Get if the panel is active.
    pub fn active(&self) -> bool {
        self.active
    }

    /// Get the panel's width share.
    pub fn width_share(&self) -> u32 {
        assert!(self.width_share > 0);
        self.width_share
    }

    /// Get the panel's height share.
    pub fn height_share(&self) -> u32 {
        assert!(self.height_share > 0);
        self.height_share
    }

    /// Get the panel's alignment.
    pub fn alignment(&self) -> Alignment {
        self.alignment
    }
}