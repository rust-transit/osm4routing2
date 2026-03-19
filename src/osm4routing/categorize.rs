//! Accessibility categorization for transportation modes.
//!
//! This module defines enums and logic for determining whether an edge
//! is accessible by foot, car, bike, or train based on OpenStreetMap tags.

use serde::Serialize;

/// Accessibility level for pedestrians.
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub enum FootAccessibility {
    /// Not yet determined from tags.
    Unknown,
    /// Explicitly forbidden for pedestrians.
    Forbidden,
    /// Allowed for pedestrians.
    Allowed,
}

/// Accessibility level for cars, including road classification.
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub enum CarAccessibility {
    /// Not yet determined from tags.
    Unknown,
    /// Forbidden for cars.
    Forbidden,
    /// http://wiki.openstreetmap.org/wiki/Tag:highway%3Dresidential
    Residential,
    /// http://wiki.openstreetmap.org/wiki/Tag:highway%3Dtertiary
    Tertiary,
    /// http://wiki.openstreetmap.org/wiki/Tag:highway%3Dsecondary
    Secondary,
    /// http://wiki.http://wiki.openstreetmap.org/wiki/Tag:highway%3Dprimary
    Primary,
    /// http://wiki.openstreetmap.org/wiki/Tag:highway%3Dtrunk
    Trunk,
    /// http://wiki.openstreetmap.org/wiki/Tag:highway%3Dmotorway
    Motorway,
}

/// Accessibility level for bicycles.
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub enum BikeAccessibility {
    /// Not yet determined from tags.
    Unknown,
    /// Forbidden for bicycles.
    Forbidden,
    /// Allowed, but traffic is shared with cars.
    Allowed,
    /// Narrow lane dedicated for bikes without physical separation.
    Lane,
    /// Bikes are allowed on the bus lane.
    Busway,
    /// Physically separated from other traffic.
    Track,
}

/// Accessibility level for trains.
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
pub enum TrainAccessibility {
    /// Not yet determined from tags.
    Unknown,
    /// Forbidden for trains.
    Forbidden,
    /// Railway/track is present.
    Allowed,
}

/// Properties of an edge describing accessibility for all transportation modes.
///
/// Tracks accessibility separately for forward and backward directions
/// where applicable (cars and bikes).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EdgeProperties {
    /// Pedestrian accessibility (direction-independent).
    pub foot: FootAccessibility,
    /// Car accessibility in the forward direction.
    pub car_forward: CarAccessibility,
    /// Car accessibility in the backward direction.
    pub car_backward: CarAccessibility,
    /// Bike accessibility in the forward direction.
    pub bike_forward: BikeAccessibility,
    /// Bike accessibility in the backward direction.
    pub bike_backward: BikeAccessibility,
    /// Train accessibility (direction-independent).
    pub train: TrainAccessibility,
}

impl Default for EdgeProperties {
    fn default() -> EdgeProperties {
        EdgeProperties {
            foot: FootAccessibility::Unknown,
            car_forward: CarAccessibility::Unknown,
            car_backward: CarAccessibility::Unknown,
            bike_forward: BikeAccessibility::Unknown,
            bike_backward: BikeAccessibility::Unknown,
            train: TrainAccessibility::Unknown,
        }
    }
}

impl EdgeProperties {
    /// Normalize properties by converting Unknown values to concrete values.
    ///
    /// Applies the following rules:
    /// - If `car_backward` is Unknown, copy from `car_forward`
    /// - If `bike_backward` is Unknown, copy from `bike_forward`
    /// - Any remaining Unknown values become Forbidden
    pub fn normalize(&mut self) {
        if self.car_backward == CarAccessibility::Unknown {
            self.car_backward = self.car_forward;
        }
        if self.bike_backward == BikeAccessibility::Unknown {
            self.bike_backward = self.bike_forward;
        }
        if self.car_forward == CarAccessibility::Unknown {
            self.car_forward = CarAccessibility::Forbidden;
        }
        if self.bike_forward == BikeAccessibility::Unknown {
            self.bike_forward = BikeAccessibility::Forbidden;
        }
        if self.car_backward == CarAccessibility::Unknown {
            self.car_backward = CarAccessibility::Forbidden;
        }
        if self.bike_backward == BikeAccessibility::Unknown {
            self.bike_backward = BikeAccessibility::Forbidden;
        }
        if self.foot == FootAccessibility::Unknown {
            self.foot = FootAccessibility::Forbidden;
        }
        if self.train == TrainAccessibility::Unknown {
            self.train = TrainAccessibility::Forbidden;
        }
    }

    /// Check if at least one transportation mode can use this edge in any direction.
    ///
    /// Returns `false` if all modes are Forbidden.
    pub fn accessible(self) -> bool {
        self.bike_forward != BikeAccessibility::Forbidden
            || self.bike_backward != BikeAccessibility::Forbidden
            || self.car_forward != CarAccessibility::Forbidden
            || self.car_backward != CarAccessibility::Forbidden
            || self.foot != FootAccessibility::Forbidden
            || self.train != TrainAccessibility::Forbidden
    }

    /// Update properties based on an OSM tag key-value pair (owned strings).
    ///
    /// # Arguments
    /// * `key_string` - The OSM tag key.
    /// * `val_string` - The OSM tag value.
    pub fn update(&mut self, key_string: String, val_string: String) {
        let key = key_string.as_str();
        let val = val_string.as_str();
        self.update_with_str(key, val);
    }

    /// Update properties based on an OSM tag key-value pair (string slices).
    ///
    /// Recognizes the following tags:
    /// - `highway`: road classification (cycleway, path, primary, secondary, etc.)
    /// - `pedestrian`/`foot`: pedestrian access
    /// - `cycleway`: bike lane/track configuration
    /// - `bicycle`: bike access
    /// - `busway`: bus lane access for bikes
    /// - `oneway`: one-way restriction
    /// - `junction=roundabout`: roundabout handling
    /// - `railway`: train access
    ///
    /// # Arguments
    /// * `key` - The OSM tag key.
    /// * `val` - The OSM tag value.
    pub fn update_with_str(&mut self, key: &str, val: &str) {
        match key {
            "highway" => match val {
                "cycleway" => {
                    self.bike_forward = BikeAccessibility::Track;
                    self.foot = FootAccessibility::Allowed;
                }
                "path" | "footway" | "steps" | "pedestrian" => {
                    self.bike_forward = BikeAccessibility::Allowed;
                    self.foot = FootAccessibility::Allowed;
                }
                "primary" | "primary_link" => {
                    self.car_forward = CarAccessibility::Primary;
                    self.foot = FootAccessibility::Allowed;
                    self.bike_forward = BikeAccessibility::Allowed;
                }
                "secondary" | "secondary_link" => {
                    self.car_forward = CarAccessibility::Secondary;
                    self.foot = FootAccessibility::Allowed;
                    self.bike_forward = BikeAccessibility::Allowed;
                }
                "tertiary" | "tertiary_link" => {
                    self.car_forward = CarAccessibility::Tertiary;
                    self.foot = FootAccessibility::Allowed;
                    self.bike_forward = BikeAccessibility::Allowed;
                }
                "unclassified" | "residential" | "living_street" | "road" | "service" | "track" => {
                    self.car_forward = CarAccessibility::Residential;
                    self.foot = FootAccessibility::Allowed;
                    self.bike_forward = BikeAccessibility::Allowed;
                }
                "motorway" | "motorway_link" | "motorway_junction" => {
                    self.car_forward = CarAccessibility::Motorway;
                    self.foot = FootAccessibility::Forbidden;
                    self.bike_forward = BikeAccessibility::Forbidden;
                }
                "trunk" | "trunk_link" => {
                    self.car_forward = CarAccessibility::Trunk;
                    self.foot = FootAccessibility::Forbidden;
                    self.bike_forward = BikeAccessibility::Forbidden;
                }
                _ => {}
            },
            "pedestrian" | "foot" => match val {
                "no" => self.foot = FootAccessibility::Forbidden,
                _ => self.foot = FootAccessibility::Allowed,
            },

            "cycleway" => match val {
                "track" => self.bike_forward = BikeAccessibility::Track,
                "opposite_track" => self.bike_backward = BikeAccessibility::Track,
                "opposite" => self.bike_backward = BikeAccessibility::Allowed,
                "share_busway" => self.bike_forward = BikeAccessibility::Busway,
                "lane_left" | "opposite_lane" => self.bike_backward = BikeAccessibility::Lane,
                _ => self.bike_forward = BikeAccessibility::Lane,
            },

            "bicycle" => match val {
                "no" | "false" => self.bike_forward = BikeAccessibility::Forbidden,
                _ => self.bike_forward = BikeAccessibility::Allowed,
            },
            "busway" => match val {
                "opposite_lane" | "opposite_track" => {
                    self.bike_backward = BikeAccessibility::Busway
                }
                _ => self.bike_forward = BikeAccessibility::Busway,
            },
            "oneway" => match val {
                "yes" | "true" | "1" => {
                    self.car_backward = CarAccessibility::Forbidden;
                    if self.bike_backward == BikeAccessibility::Unknown {
                        self.bike_backward = BikeAccessibility::Forbidden;
                    }
                }
                _ => {}
            },
            "junction" => {
                if val == "roundabout" {
                    self.car_backward = CarAccessibility::Forbidden;
                    if self.bike_backward == BikeAccessibility::Unknown {
                        self.bike_backward = BikeAccessibility::Forbidden;
                    }
                }
            }
            "railway" => {
                self.train = TrainAccessibility::Allowed;
            }
            _ => {}
        }
    }
}

#[test]
fn test_accessible() {
    let mut p = EdgeProperties::default();
    p.normalize();
    assert!(!p.accessible());

    p.foot = FootAccessibility::Allowed;
    assert!(p.accessible())
}

#[test]
fn test_normalize() {
    let mut p = EdgeProperties::default();
    p.bike_forward = BikeAccessibility::Lane;
    p.normalize();
    assert_eq!(BikeAccessibility::Lane, p.bike_backward);
    p.bike_forward = BikeAccessibility::Allowed;
    p.normalize();
    assert_eq!(BikeAccessibility::Lane, p.bike_backward);

    p.car_forward = CarAccessibility::Secondary;
    p.car_backward = CarAccessibility::Unknown;
    p.normalize();
    assert_eq!(CarAccessibility::Secondary, p.car_backward)
}

#[test]
fn test_update() {
    let mut p = EdgeProperties::default();
    p.update_with_str("highway", "secondary");
    assert_eq!(CarAccessibility::Secondary, p.car_forward);

    p.update_with_str("highway", "primary_link");
    assert_eq!(CarAccessibility::Primary, p.car_forward);

    p.update_with_str("highway", "motorway");
    assert_eq!(CarAccessibility::Motorway, p.car_forward);

    p.update_with_str("highway", "residential");
    assert_eq!(CarAccessibility::Residential, p.car_forward);

    p.update_with_str("highway", "tertiary");
    assert_eq!(CarAccessibility::Tertiary, p.car_forward);

    p.update_with_str("highway", "trunk");
    assert_eq!(CarAccessibility::Trunk, p.car_forward);

    p.update_with_str("highway", "cycleway");
    assert_eq!(BikeAccessibility::Track, p.bike_forward);
    assert_eq!(FootAccessibility::Allowed, p.foot);

    p.update_with_str("foot", "designated");
    assert_eq!(FootAccessibility::Allowed, p.foot);

    p.update_with_str("foot", "no");
    assert_eq!(FootAccessibility::Forbidden, p.foot);

    p.update_with_str("cycleway", "lane");
    assert_eq!(BikeAccessibility::Lane, p.bike_forward);

    p.update_with_str("cycleway", "track");
    assert_eq!(BikeAccessibility::Track, p.bike_forward);

    p.update_with_str("cycleway", "opposite_lane");
    assert_eq!(BikeAccessibility::Lane, p.bike_backward);

    p.update_with_str("cycleway", "opposite_track");
    assert_eq!(BikeAccessibility::Track, p.bike_backward);

    p.update_with_str("cycleway", "opposite");
    assert_eq!(BikeAccessibility::Allowed, p.bike_backward);

    p.update_with_str("cycleway", "share_busway");
    assert_eq!(BikeAccessibility::Busway, p.bike_forward);

    p.update_with_str("cycleway", "lane_left");
    assert_eq!(BikeAccessibility::Lane, p.bike_backward);

    p.update_with_str("bicycle", "yes");
    assert_eq!(BikeAccessibility::Allowed, p.bike_forward);

    p.update_with_str("bicycle", "no");
    assert_eq!(BikeAccessibility::Forbidden, p.bike_forward);

    p.update_with_str("busway", "yes");
    assert_eq!(BikeAccessibility::Busway, p.bike_forward);

    p.update_with_str("busway", "opposite_track");
    assert_eq!(BikeAccessibility::Busway, p.bike_backward);

    p.update_with_str("oneway", "yes");
    assert_eq!(CarAccessibility::Forbidden, p.car_backward);
    assert!(p.bike_backward != BikeAccessibility::Forbidden);

    p.bike_backward = BikeAccessibility::Unknown;
    p.update_with_str("oneway", "yes");
    assert_eq!(BikeAccessibility::Forbidden, p.bike_backward);

    p.update_with_str("junction", "roundabout");
    assert_eq!(CarAccessibility::Forbidden, p.car_backward);

    p.bike_backward = BikeAccessibility::Unknown;
    p.update_with_str("junction", "roundabout");
    assert_eq!(BikeAccessibility::Forbidden, p.bike_backward);
}
