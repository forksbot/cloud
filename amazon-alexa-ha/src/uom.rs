//! # Alexa units of measure

use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "derive_debug", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnitOfMeasure {
    #[serde(rename = "Alexa.Unit.Angle.Degrees")]
    AlexaUnitAngleDegrees,
    #[serde(rename = "Alexa.Unit.Angle.Radians")]
    AlexaUnitAngleRadians,
    #[serde(rename = "Alexa.Unit.Distance.Feet")]
    AlexaUnitDistanceFeet,
    #[serde(rename = "Alexa.Unit.Distance.Inches")]
    AlexaUnitDistanceInches,
    #[serde(rename = "Alexa.Unit.Distance.Kilometers")]
    AlexaUnitDistanceKilometers,
    #[serde(rename = "Alexa.Unit.Distance.Meters")]
    AlexaUnitDistanceMeters,
    #[serde(rename = "Alexa.Unit.Distance.Miles")]
    AlexaUnitDistanceMiles,
    #[serde(rename = "Alexa.Unit.Distance.Yards")]
    AlexaUnitDistanceYards,
    #[serde(rename = "Alexa.Unit.Mass.Grams")]
    AlexaUnitMassGrams,
    #[serde(rename = "Alexa.Unit.Mass.Kilograms")]
    AlexaUnitMassKilograms,
    #[serde(rename = "Alexa.Unit.Percent")]
    AlexaUnitPercent,
    #[serde(rename = "Alexa.Unit.Temperature.Celsius")]
    AlexaUnitTemperatureCelsius,
    #[serde(rename = "Alexa.Unit.Temperature.Degrees")]
    AlexaUnitTemperatureDegrees,
    #[serde(rename = "Alexa.Unit.Temperature.Fahrenheit")]
    AlexaUnitTemperatureFahrenheit,
    #[serde(rename = "Alexa.Unit.Temperature.Kelvin")]
    AlexaUnitTemperatureKelvin,
    #[serde(rename = "Alexa.Unit.Volume.CubicFeet")]
    AlexaUnitVolumeCubicFeet,
    #[serde(rename = "Alexa.Unit.Volume.CubicMeters")]
    AlexaUnitVolumeCubicMeters,
    #[serde(rename = "Alexa.Unit.Volume.Gallons")]
    AlexaUnitVolumeGallons,
    #[serde(rename = "Alexa.Unit.Volume.Liters")]
    AlexaUnitVolumeLiters,
    #[serde(rename = "Alexa.Unit.Volume.Pints")]
    AlexaUnitVolumePints,
    #[serde(rename = "Alexa.Unit.Volume.Quarts")]
    AlexaUnitVolumeQuarts,
    #[serde(rename = "Alexa.Unit.Weight.Ounces")]
    AlexaUnitWeightOunces,
    #[serde(rename = "Alexa.Unit.Weight.Pounds")]
    AlexaUnitWeightPounds,
}
