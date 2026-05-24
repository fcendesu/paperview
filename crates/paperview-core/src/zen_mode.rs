#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ZenModeState {
    enabled: bool,
}

impl ZenModeState {
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

#[cfg(test)]
mod tests {
    use super::ZenModeState;

    #[test]
    fn toggles_zen_mode() {
        let mut zen = ZenModeState::default();

        assert!(!zen.is_enabled());

        zen.toggle();

        assert!(zen.is_enabled());
    }

    #[test]
    fn can_start_enabled_and_be_set() {
        let mut zen = ZenModeState::new(true);

        assert!(zen.is_enabled());

        zen.set_enabled(false);

        assert!(!zen.is_enabled());
    }
}
