use crate::keyboard::KeyboardModel;

pub struct ModelSpec {
    pub commit: Option<&'static [u8]>,
    pub group_addresses: &'static [(u8, &'static [u8])],
    pub effect_params: Option<(u8, u8)>,
    pub mr_header: Option<&'static [u8]>,
    pub mn_header: Option<&'static [u8]>,
    pub mn_map: Option<&'static [(u8, u8)]>,
    pub gkeys_header: Option<&'static [u8]>,
    pub startup_header: Option<&'static [u8]>,
    pub onboard_header: Option<&'static [u8]>,
    pub keys_header: Option<&'static [u8]>,
    pub region_header: Option<&'static [u8]>,
}

impl ModelSpec {
    const fn empty() -> Self {
        Self {
            commit: None,
            group_addresses: &[],
            effect_params: None,
            mr_header: None,
            mn_header: None,
            mn_map: None,
            gkeys_header: None,
            startup_header: None,
            onboard_header: None,
            keys_header: None,
            region_header: None,
        }
    }

    /// Entry point for our builder pattern.
    ///
    /// All setters are `const fn`, so the whole chain is still valid
    /// in a `const` context.
    pub const fn builder() -> Self {
        Self::empty()
    }

    #[must_use]
    pub const fn commit(mut self, commit_bytes: &'static [u8]) -> Self {
        self.commit = Some(commit_bytes);
        self
    }

    #[must_use]
    pub const fn group_addresses(
        mut self,
        group_addr_tuples: &'static [(u8, &'static [u8])],
    ) -> Self {
        self.group_addresses = group_addr_tuples;
        self
    }

    #[must_use]
    pub const fn effect_params(mut self, bank_number: u8, param_address: u8) -> Self {
        self.effect_params = Some((bank_number, param_address));
        self
    }

    #[must_use]
    pub const fn mr_header(mut self, mr_header_bytes: &'static [u8]) -> Self {
        self.mr_header = Some(mr_header_bytes);
        self
    }

    #[must_use]
    pub const fn mn_header(mut self, mn_header_bytes: &'static [u8]) -> Self {
        self.mn_header = Some(mn_header_bytes);
        self
    }

    #[must_use]
    pub const fn mn_map(mut self, map_entries: &'static [(u8, u8)]) -> Self {
        self.mn_map = Some(map_entries);
        self
    }

    #[must_use]
    pub const fn gkeys_header(mut self, gkeys_header_bytes: &'static [u8]) -> Self {
        self.gkeys_header = Some(gkeys_header_bytes);
        self
    }

    #[must_use]
    pub const fn startup_header(mut self, startup_header_bytes: &'static [u8]) -> Self {
        self.startup_header = Some(startup_header_bytes);
        self
    }

    #[must_use]
    pub const fn onboard_header(mut self, onboard_header_bytes: &'static [u8]) -> Self {
        self.onboard_header = Some(onboard_header_bytes);
        self
    }

    #[must_use]
    pub const fn keys_header(mut self, keys_header_bytes: &'static [u8]) -> Self {
        self.keys_header = Some(keys_header_bytes);
        self
    }

    #[must_use]
    pub const fn region_header(mut self, region_header_bytes: &'static [u8]) -> Self {
        self.region_header = Some(region_header_bytes);
        self
    }

    /// Applies the standard lighting effect parameters and startup header used by most GX-series models.
    ///
    /// This is a convenience helper for models like G410, G512, G610, G810, and G Pro,
    /// which share the same `(0x0d or 0x0c, 0x3c)` effect register pair and common startup
    /// initialization packet.
    pub const fn with_gx_defaults(mut self, bank: u8) -> Self {
        self.effect_params = Some((bank, 0x3c));
        self.startup_header = Some(&[0x11, 0xff, 0x0d, 0x5a, 0x00, 0x01]);
        self
    }
}

const ADDR_GX: &[(u8, &[u8])] = &[
    (0, &[0x11, 0xff, 0x0c, 0x3a, 0x00, 0x10, 0x00, 0x01]),
    (1, &[0x12, 0xff, 0x0c, 0x3a, 0x00, 0x40, 0x00, 0x05]),
    (4, &[0x12, 0xff, 0x0f, 0x3d, 0x00, 0x01, 0x00, 0x0e]),
];

const ADDR_G610_G810: &[(u8, &[u8])] = &[
    (0, &[0x11, 0xff, 0x0c, 0x3a, 0x00, 0x10, 0x00, 0x01]),
    (1, &[0x12, 0xff, 0x0c, 0x3a, 0x00, 0x40, 0x00, 0x05]),
    (4, &[0x12, 0xff, 0x0f, 0x3d, 0x00, 0x01, 0x00, 0x0e]),
    (2, &[0x12, 0xff, 0x0c, 0x3a, 0x00, 0x02, 0x00, 0x05]),
];

const ADDR_G815: &[(u8, &[u8])] = &[(0, &[0x11, 0xff, 0x10, 0x1c])];

const ADDR_G910: &[(u8, &[u8])] = &[
    (0, &[0x11, 0xff, 0x0f, 0x3a, 0x00, 0x10, 0x00, 0x02]),
    (1, &[0x12, 0xff, 0x0c, 0x3a, 0x00, 0x40, 0x00, 0x05]),
    (3, &[0x12, 0xff, 0x0f, 0x3e, 0x00, 0x04, 0x00, 0x09]),
    (4, &[0x12, 0xff, 0x0f, 0x3d, 0x00, 0x01, 0x00, 0x0e]),
];

const MN_MAP_G815: &[(u8, u8)] = &[(0x01, 0x01), (0x02, 0x02), (0x03, 0x04)];

pub const MODEL_SPECS: [ModelSpec; 11] = [
    // Unknown
    ModelSpec::builder(),
    // G213
    ModelSpec::builder()
        .group_addresses(ADDR_GX)
        .with_gx_defaults(0x0c)
        .region_header(&[0x11, 0xff, 0x0c, 0x3a]),
    // G410
    ModelSpec::builder()
        .commit(&[0x11, 0xff, 0x0c, 0x5a])
        .group_addresses(ADDR_GX)
        .with_gx_defaults(0x0d),
    // G413
    ModelSpec::builder()
        .group_addresses(ADDR_GX)
        .with_gx_defaults(0x0c),
    // G512
    ModelSpec::builder()
        .commit(&[0x11, 0xff, 0x0c, 0x5a])
        .group_addresses(ADDR_GX)
        .with_gx_defaults(0x0d),
    // G513
    ModelSpec::builder()
        .commit(&[0x11, 0xff, 0x0c, 0x5a])
        .group_addresses(ADDR_GX)
        .with_gx_defaults(0x0d),
    // G610
    ModelSpec::builder()
        .commit(&[0x11, 0xff, 0x0c, 0x5a])
        .group_addresses(ADDR_G610_G810)
        .with_gx_defaults(0x0d),
    // G810
    ModelSpec::builder()
        .commit(&[0x11, 0xff, 0x0c, 0x5a])
        .group_addresses(ADDR_G610_G810)
        .with_gx_defaults(0x0d),
    // G815
    ModelSpec::builder()
        .commit(&[0x11, 0xff, 0x10, 0x7f])
        .group_addresses(ADDR_G815)
        .effect_params(0x0f, 0x1c)
        .mr_header(&[0x11, 0xff, 0x0c, 0x0c])
        .mn_header(&[0x11, 0xff, 0x0b, 0x1c])
        .mn_map(MN_MAP_G815)
        .gkeys_header(&[0x11, 0xff, 0x0a, 0x2b])
        .onboard_header(&[0x11, 0xff, 0x11, 0x1a])
        .keys_header(&[0x11, 0xff, 0x10, 0x6c]),
    // G910
    ModelSpec::builder()
        .commit(&[0x11, 0xff, 0x0f, 0x5d])
        .group_addresses(ADDR_G910)
        .effect_params(0x10, 0x3c)
        .mr_header(&[0x11, 0xff, 0x0a, 0x0e])
        .mn_header(&[0x11, 0xff, 0x09, 0x1e])
        .gkeys_header(&[0x11, 0xff, 0x08, 0x2e])
        .startup_header(&[0x11, 0xff, 0x10, 0x5e, 0x00, 0x01]),
    // GPro
    ModelSpec::builder()
        .commit(&[0x11, 0xff, 0x0c, 0x5a])
        .group_addresses(ADDR_GX)
        .with_gx_defaults(0x0d),
];

impl KeyboardModel {
    pub fn spec(self) -> &'static ModelSpec {
        &MODEL_SPECS[self as usize]
    }
}
