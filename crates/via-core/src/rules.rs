use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct BoardRules {
    grid_mm: f64,
    default_track_width_mm: f64,
    net_class_track_width_mm: BTreeMap<String, f64>,
    clearance_mm: f64,
    via_drill_mm: f64,
    via_diameter_mm: f64,
}

impl Default for BoardRules {
    fn default() -> Self {
        Self {
            grid_mm: 2.0,
            default_track_width_mm: 0.3,
            net_class_track_width_mm: BTreeMap::from([
                ("ground".to_owned(), 0.6),
                ("power:12V".to_owned(), 0.8),
                ("power:5V".to_owned(), 0.6),
                ("power:3V3".to_owned(), 0.5),
                ("logic:3V3".to_owned(), 0.25),
                ("motor-phase".to_owned(), 0.5),
            ]),
            clearance_mm: 0.2,
            via_drill_mm: 0.4,
            via_diameter_mm: 0.8,
        }
    }
}

impl BoardRules {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grid_mm(&self) -> f64 {
        self.grid_mm
    }

    pub fn default_track_width_mm(&self) -> f64 {
        self.default_track_width_mm
    }

    pub fn net_class_track_widths_mm(&self) -> impl Iterator<Item = (&String, &f64)> {
        self.net_class_track_width_mm.iter()
    }

    pub fn track_width_for_class_mm(&self, class: &str) -> Option<f64> {
        self.net_class_track_width_mm.get(class).copied()
    }

    pub fn clearance_mm(&self) -> f64 {
        self.clearance_mm
    }

    pub fn via_drill_mm(&self) -> f64 {
        self.via_drill_mm
    }

    pub fn via_diameter_mm(&self) -> f64 {
        self.via_diameter_mm
    }

    pub fn set_grid_mm(&mut self, value: f64) -> &mut Self {
        self.grid_mm = value;
        self
    }

    pub fn set_default_track_width_mm(&mut self, value: f64) -> &mut Self {
        self.default_track_width_mm = value;
        self
    }

    pub fn set_net_class_track_width_mm(
        &mut self,
        class: impl Into<String>,
        value: f64,
    ) -> &mut Self {
        self.net_class_track_width_mm.insert(class.into(), value);
        self
    }

    pub fn set_clearance_mm(&mut self, value: f64) -> &mut Self {
        self.clearance_mm = value;
        self
    }

    pub fn set_via(&mut self, diameter_mm: f64, drill_mm: f64) -> &mut Self {
        self.via_diameter_mm = diameter_mm;
        self.via_drill_mm = drill_mm;
        self
    }
}
