#![no_std]
#![no_main]

use defmt::info;
use embassy_stm32::gpio::{Level, Output, Speed as PinSpeed};
use embassy_time::{Duration, Timer};
use stepper::stepper::{
    Stepper, StepperAttachment, StepperOptions, SteppingMode,
};
use embassy_executor::Spawner;
use math::{common::RotationDirection, measurements::AngularVelocity};
use common::{OutputPinBase, TimerBase};

use {defmt_rtt as _, panic_probe as _, };

#[cfg(feature="defmt-log")]
use defmt::info;

struct StepperTimer {}

impl TimerBase for StepperTimer {
    async fn after(duration: core::time::Duration) {
        let duration = embassy_time::Duration::from_micros(duration.as_micros() as u64);
        Timer::after(duration).await
    }
}

struct StepperPin<'a> {
    pin: Output<'a>,
}

impl OutputPinBase for StepperPin<'_> {
    fn set_high(&mut self) {
        self.pin.set_high();
    }

    fn set_low(&mut self) {
        self.pin.set_low();
    }

    fn is_high(&self) -> bool {
        self.pin.is_set_high()
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(embassy_stm32::Config::default());

    // let step = SimplePwm::new(
    //     p.TIM5,
    //     Some(PwmPin::new_ch1(p.PA0, OutputType::PushPull)),
    //     None,
    //     None,
    //     None,
    //     hz(1),
    //     CountingMode::EdgeAlignedUp,
    // );

    let step = StepperPin {
        pin: Output::new(p.PC9, Level::Low, PinSpeed::Low),
    };

    let dir = StepperPin {
        pin: Output::new(p.PC8, Level::Low, PinSpeed::Low),
    };

    let mut stepper = Stepper::new_with_attachment(
        step,
        dir,
        StepperOptions::default(),
        StepperAttachment::default(),
    );

    stepper.set_stepping_mode(SteppingMode::FullStep);

    stepper.set_speed(AngularVelocity::from_rpm(360.0));

    // let mut d = Distance::from_mm(80.0);

    loop {
        stepper.set_direction(RotationDirection::CounterClockwise);
        if let Err(_) = stepper.move_for_steps::<StepperTimer>(400).await {
            #[cfg(feature="defmt-log")]
            info!("Cannot move");
        };
        #[cfg(feature="defmt-log")]
        info!("Position: {}", stepper.get_position().as_millimeters());

        Timer::after(Duration::from_millis(100)).await;

        stepper.set_direction(RotationDirection::Clockwise);

        if let Err(_) = stepper.move_for_steps::<StepperTimer>(400).await {
            #[cfg(feature="defmt-log")]
            info!("Cannot move");
        };

        Timer::after(Duration::from_millis(100)).await;

        // #[cfg(feature="defmt-log")]
        // info!("Position: {}", stepper.get_position().as_millimeters());

        // info!("Moving to {}mm", d.to_mm());
        // if let Err(e) = stepper.move_to_destination(d).await {
        //     match e {
        //         stepper::a4988::StepperError::MoveTooShort => info!("Move too short"),
        //         stepper::a4988::StepperError::MoveOutOfBounds => info!("Move out of bounds"),
        //         stepper::a4988::StepperError::MoveNotValid => info!("Move not valid"),
        //         stepper::a4988::StepperError::MissingAttachment => info!("Missing attachment"),
        //     }
        // };

        // d = Distance::from_mm(-d.to_mm());

        // Timer::after(Duration::from_millis(100)).await;
    }
}
