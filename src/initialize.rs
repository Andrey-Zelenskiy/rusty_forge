use config::Config;
// Copyright Andrey Zelenskiy, 2024-2026
use serde::{Deserialize, Serialize};

/* ------------------------------------ */
/* Methods for structure initialization */
/* ------------------------------------ */

// Trait for argument structure with required initialization function
pub trait BuilderMethods:
    Default + for<'de> Deserialize<'de> + Serialize
{
    type Target;

    // Initialize target structure from the parameters
    fn build(&mut self) -> Self::Target;

    // Create a builder from the existing target
    fn from_target(target: &Self::Target) -> Self;
}

// Trait for initializing a structure from an argument structure
pub trait TargetFromBuilder {
    type Builder: BuilderMethods<Target = Self>;
    // Initialize new Target from input parameters
    fn builder() -> Self::Builder {
        Self::Builder::default()
    }

    // Initialize Target from a config file
    fn from_config(config: &Config, config_name: &str) -> Self
    where
        Self: Sized,
    {
        //Populate the parameters from the config
        match config.get::<Self::Builder>(config_name) {
            Ok(mut builder) => builder.build(),
            Err(reason) => panic!("Missing config for {config_name}: {reason}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    pub struct TargetStruct {
        x2: u32,
        xy: u32,
        y2: u32,
    }

    #[derive(Deserialize, Default, Serialize)]
    pub struct Builder {
        x: u32,
        y: u32,
    }

    // Add methods for setting values
    impl Builder {
        pub fn set_x(&mut self, x: u32) -> &mut Self {
            self.x = x;
            self
        }

        pub fn set_y(&mut self, y: u32) -> &mut Self {
            self.y = y;
            self
        }
    }

    impl BuilderMethods for Builder {
        type Target = TargetStruct;

        fn build(&mut self) -> Self::Target {
            Self::Target {
                x2: self.x * self.x,
                xy: self.x * self.y,
                y2: self.y * self.y,
            }
        }

        fn from_target(target: &TargetStruct) -> Self {
            Self {
                x: target.x2.isqrt(),
                y: target.y2.isqrt(),
            }
        }
    }

    impl TargetFromBuilder for TargetStruct {
        type Builder = Builder;
    }

    #[test]
    fn build() {
        let target = TargetStruct::builder().set_x(1).set_y(2).build();

        assert_eq!(1, target.x2);
        assert_eq!(2, target.xy);
        assert_eq!(4, target.y2);
    }

    #[test]
    fn builder_from_target() {
        let target = TargetStruct {
            x2: 1,
            xy: 2,
            y2: 4,
        };
        let builder = Builder::from_target(&target);

        assert_eq!(builder.x, 1);
        assert_eq!(builder.y, 2);
    }
}
