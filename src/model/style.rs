use crate::model::Tagged;

use std::default::Default;

pub trait Style {
    fn common_styles_apply(&mut self, _styles: &mut CommonStyles) {}

    fn tagged_styles_apply(&mut self, _value: &mut dyn Tagged) {}
}

impl<'a, 'b> Style for &'a mut [&'b mut dyn Style] {
    fn common_styles_apply(&mut self, styles: &mut CommonStyles) {
        for style in self.into_iter() {
            style.common_styles_apply(styles);
        }
    }

    fn tagged_styles_apply(&mut self, value: &mut dyn Tagged) {
        for style in self.into_iter() {
            style.tagged_styles_apply(value);
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct CommonStyles {
    indent: u8,
    threshold: u8,
    params: u16,
}

impl CommonStyles {
    fn _on(&mut self, val: u16) {
        self.params |= val;
    }

    fn _off(&mut self, val: u16) {
        self.params &= !val;
    }

    fn _set(&mut self, val: u16, onoff: bool) {
        if onoff {
            self._on(val)
        } else {
            self._off(val)
        }
    }

    fn _is(&self, val: u16) -> bool {
        self.params & val == val
    }

    pub fn flow(&self) -> bool {
        self._is(1)
    }

    pub fn set_flow(&mut self, val: bool) {
        self._set(1, val);
    }

    pub fn compact(&self) -> bool {
        self._is(2)
    }

    pub fn set_compact(&mut self, val: bool) {
        self._set(2, val)
    }

    pub fn respect_threshold(&self) -> bool {
        self._is(4)
    }

    pub fn set_respect_threshold(&mut self, val: bool) {
        self._set(4, val)
    }

    pub fn multiline(&self) -> bool {
        self._is(8)
    }

    pub fn set_multiline(&mut self, val: bool) {
        self._set(8, val)
    }

    pub fn issue_tag(&self) -> bool {
        self._is(16)
    }

    pub fn set_issue_tag(&mut self, val: bool) {
        self._set(16, val)
    }

    pub fn indent(&self) -> u8 {
        self.indent
    }

    pub fn set_indent(&mut self, value: u8) {
        if value > 0 {
            self.indent = value;
        }
    }

    pub fn threshold(&self) -> u8 {
        self.threshold
    }

    pub fn set_threshold(&mut self, value: u8) {
        if value > 0 {
            self.threshold = value;
        }
    }
}

impl Default for CommonStyles {
    fn default() -> CommonStyles {
        CommonStyles {
            params: 0,
            indent: 2,
            threshold: 120,
        }
    }
}

pub struct Indent(pub u8);

impl Style for Indent {
    fn common_styles_apply(&mut self, style: &mut CommonStyles) {
        style.set_indent(self.0)
    }
}

pub struct Threshold(pub u8);

impl Style for Threshold {
    fn common_styles_apply(&mut self, style: &mut CommonStyles) {
        style.set_threshold(self.0)
    }
}

pub const FLOW: Flow = Flow(true);
pub const NO_FLOW: Flow = Flow(false);

pub struct Flow(pub bool);

impl Style for Flow {
    fn common_styles_apply(&mut self, style: &mut CommonStyles) {
        style.set_flow(self.0)
    }
}

pub const COMPACT: Compact = Compact(true);
pub const NO_COMPACT: Compact = Compact(false);

pub struct Compact(pub bool);

impl Style for Compact {
    fn common_styles_apply(&mut self, style: &mut CommonStyles) {
        style.set_compact(self.0)
    }
}

pub const RESPECT_THRESHOLD: RespectThreshold = RespectThreshold(true);
pub const NO_RESPECT_THRESHOLD: RespectThreshold = RespectThreshold(false);

pub struct RespectThreshold(pub bool);

impl Style for RespectThreshold {
    fn common_styles_apply(&mut self, style: &mut CommonStyles) {
        style.set_respect_threshold(self.0)
    }
}

pub const MULTILINE: Multiline = Multiline(true);
pub const NO_MULTILINE: Multiline = Multiline(false);

pub struct Multiline(pub bool);

impl Style for Multiline {
    fn common_styles_apply(&mut self, style: &mut CommonStyles) {
        style.set_multiline(self.0)
    }
}

pub const ISSUE_TAG: IssueTag = IssueTag(true);
pub const NO_ISSUE_TAG: IssueTag = IssueTag(false);

pub struct IssueTag(pub bool);

impl Style for IssueTag {
    fn common_styles_apply(&mut self, style: &mut CommonStyles) {
        style.set_issue_tag(self.0)
    }
}
