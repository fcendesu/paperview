#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitResize {
    GrowPrimary,
    ShrinkPrimary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SplitViewState {
    secondary_index: Option<usize>,
    primary_width: u16,
}

#[must_use]
pub fn synced_scroll_offset(primary_offset: u16, primary_max: u16, secondary_max: u16) -> u16 {
    if primary_max == 0 || secondary_max == 0 {
        return 0;
    }

    let progress = f32::from(primary_offset.min(primary_max)) / f32::from(primary_max);
    (progress * f32::from(secondary_max)).round() as u16
}

impl SplitViewState {
    pub const DEFAULT_PRIMARY_WIDTH: u16 = 50;
    pub const MIN_PRIMARY_WIDTH: u16 = 30;
    pub const MAX_PRIMARY_WIDTH: u16 = 70;
    pub const RESIZE_STEP: u16 = 10;

    #[must_use]
    pub fn new(primary_width: u16) -> Self {
        Self {
            secondary_index: None,
            primary_width: clamp_primary_width(primary_width),
        }
    }

    #[must_use]
    pub fn secondary_index(&self) -> Option<usize> {
        self.secondary_index
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.secondary_index.is_some()
    }

    #[must_use]
    pub fn primary_width(&self) -> u16 {
        self.primary_width
    }

    #[must_use]
    pub fn widths(&self) -> (u16, u16) {
        (self.primary_width, 100 - self.primary_width)
    }

    pub fn toggle(&mut self, active_index: Option<usize>, document_count: usize) {
        self.secondary_index = self
            .secondary_index
            .is_none()
            .then(|| first_secondary_index(active_index, document_count))
            .flatten();
    }

    pub fn disable(&mut self) {
        self.secondary_index = None;
    }

    pub fn select_secondary(&mut self, index: usize, active_index: Option<usize>, len: usize) {
        if self.is_enabled() && Some(index) != active_index && index < len {
            self.secondary_index = Some(index);
        }
    }

    pub fn retarget(&mut self, active_index: Option<usize>, document_count: usize) {
        if self
            .secondary_index
            .is_some_and(|index| Some(index) == active_index || index >= document_count)
        {
            self.secondary_index = first_secondary_index(active_index, document_count);
        }
    }

    pub fn resize(&mut self, direction: SplitResize) -> bool {
        if !self.is_enabled() {
            return false;
        }

        self.primary_width = resized_width(self.primary_width, direction);
        true
    }

    pub fn set_primary_width(&mut self, width: u16) {
        self.primary_width = clamp_primary_width(width);
    }

    #[must_use]
    pub fn cycle_secondary(
        &self,
        active_index: Option<usize>,
        document_count: usize,
        offset: isize,
    ) -> Option<usize> {
        if !self.is_enabled() {
            return self.secondary_index;
        }

        let active = active_index?;
        let candidates = (0..document_count)
            .filter(|index| *index != active)
            .collect::<Vec<_>>();
        if candidates.is_empty() {
            return None;
        }

        let current_position = self
            .secondary_index
            .and_then(|current| candidates.iter().position(|index| *index == current))
            .unwrap_or(0);
        let next_position =
            (current_position as isize + offset).rem_euclid(candidates.len() as isize) as usize;
        Some(candidates[next_position])
    }
}

impl Default for SplitViewState {
    fn default() -> Self {
        Self::new(Self::DEFAULT_PRIMARY_WIDTH)
    }
}

fn resized_width(width: u16, direction: SplitResize) -> u16 {
    match direction {
        SplitResize::GrowPrimary => width
            .saturating_add(SplitViewState::RESIZE_STEP)
            .min(SplitViewState::MAX_PRIMARY_WIDTH),
        SplitResize::ShrinkPrimary => width
            .saturating_sub(SplitViewState::RESIZE_STEP)
            .max(SplitViewState::MIN_PRIMARY_WIDTH),
    }
}

fn first_secondary_index(active_index: Option<usize>, document_count: usize) -> Option<usize> {
    let active = active_index?;
    (0..document_count).find(|index| *index != active)
}

fn clamp_primary_width(width: u16) -> u16 {
    width.clamp(
        SplitViewState::MIN_PRIMARY_WIDTH,
        SplitViewState::MAX_PRIMARY_WIDTH,
    )
}

#[cfg(test)]
mod tests {
    use super::{SplitResize, SplitViewState, synced_scroll_offset};

    #[test]
    fn toggles_to_first_non_active_document() {
        let mut split = SplitViewState::default();

        split.toggle(Some(0), 2);

        assert_eq!(split.secondary_index(), Some(1));

        split.toggle(Some(0), 2);

        assert_eq!(split.secondary_index(), None);
    }

    #[test]
    fn retargets_invalid_or_active_secondary() {
        let mut split = SplitViewState::default();
        split.toggle(Some(0), 3);

        split.retarget(Some(1), 3);

        assert_eq!(split.secondary_index(), Some(0));

        split.retarget(Some(0), 1);

        assert_eq!(split.secondary_index(), None);
    }

    #[test]
    fn resizes_with_bounds() {
        let mut split = SplitViewState::default();

        assert!(!split.resize(SplitResize::GrowPrimary));

        split.toggle(Some(0), 2);
        split.resize(SplitResize::GrowPrimary);
        assert_eq!(split.widths(), (60, 40));

        for _ in 0..8 {
            split.resize(SplitResize::ShrinkPrimary);
        }

        assert_eq!(split.widths(), (30, 70));
    }

    #[test]
    fn cycles_secondary_indices() {
        let mut split = SplitViewState::default();
        split.toggle(Some(0), 3);

        assert_eq!(split.cycle_secondary(Some(0), 3, 1), Some(2));
        assert_eq!(split.cycle_secondary(Some(0), 3, -1), Some(2));
    }

    #[test]
    fn maps_synced_scroll_offset_by_relative_progress() {
        assert_eq!(synced_scroll_offset(0, 100, 50), 0);
        assert_eq!(synced_scroll_offset(50, 100, 20), 10);
        assert_eq!(synced_scroll_offset(100, 100, 50), 50);
        assert_eq!(synced_scroll_offset(150, 100, 50), 50);
        assert_eq!(synced_scroll_offset(50, 0, 50), 0);
        assert_eq!(synced_scroll_offset(50, 100, 0), 0);
    }
}
