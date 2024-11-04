https://www.martyncurrey.com/arduino-with-hc-05-bluetooth-module-at-mode/
https://www.robotstore.it/Modulo-convertitore-Seriale-Bluetooth-HC-05
https://components101.com/sites/default/files/component_datasheet/HC-05%20Datasheet.pdf

## Notes
i need something like a macro that takes a string as input, like that

my_macro {
    ($name: ident, $p: ident) => {
        use embassy_stm32::peripherals::$name
    }
}