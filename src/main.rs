#![no_std]
#![no_main]

use bsp::entry;
use embedded_hal::{
    delay::DelayNs,
    digital::{OutputPin, PinState},
    spi::MODE_0,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_halt as _;

use pimoroni_badger2040::{
    self as bsp,
    hal::{fugit::RateExtU32, gpio::FunctionSpi, usb::UsbBus, Spi, Timer},
};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use uc8151::{blocking::Uc8151, LUT};
use usb_device::{
    bus::UsbBusAllocator,
    device::{UsbDeviceBuilder, UsbVidPid},
    UsbError,
};
use usbd_serial::{embedded_io::Write, SerialPort, USB_CLASS_CDC};

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    // let core = pac::CorePeripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);
    let sio = Sio::new(pac.SIO);

    let clocks = init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .unwrap();

    let mut timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut serial = SerialPort::new(&usb_bus);
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0xdead, 0xbeef))
        .device_class(USB_CLASS_CDC)
        .build();

    let p3v3_en = pins.p3v3_en.into_push_pull_output_in_state(PinState::High);
    let sw_a = pins.sw_a.into_pull_down_input();
    let sw_b = pins.sw_b.into_pull_down_input();
    let sw_c = pins.sw_c.into_pull_down_input();
    let sw_up = pins.sw_up.into_pull_down_input();
    let sw_down = pins.sw_down.into_pull_down_input();
    let user_sw = pins.user_sw.into_pull_up_input();
    let vbus_detect = pins.vbus_detect.into_pull_up_input();
    let mut led = pins.led.into_push_pull_output_in_state(PinState::Low);

    let spi_bus = Spi::<_, _, _, 8>::new(
        pac.SPI0,
        (
            pins.mosi.into_function::<FunctionSpi>(),
            pins.miso.into_function::<FunctionSpi>(),
            pins.sclk.into_function::<FunctionSpi>(),
        ),
    );

    let spi_bus = spi_bus.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        12_000_000u32.Hz(),
        MODE_0,
    );

    let mut uc8151 = Uc8151::new(
        ExclusiveDevice::new(
            spi_bus,
            pins.inky_cs_gpio
                .into_push_pull_output_in_state(PinState::High),
            timer,
        )
        .unwrap(),
        pins.inky_dc.into_push_pull_output(),
        pins.inky_busy.into_pull_up_input(),
        pins.inky_res.into_push_pull_output_in_state(PinState::High),
        timer,
    );

    uc8151.setup(LUT::Internal).unwrap();
    for x in 0..296 {
        for y in 0..128 {
            uc8151.pixel(x, y, false);
        }
    }
    uc8151.update().unwrap();

    loop {
        led.set_high().unwrap();
        timer.delay_ms(500);
        led.set_low().unwrap();
        timer.delay_ms(500);
    }

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    //
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead.
    // One way to do that is by using [embassy](https://github.com/embassy-rs/embassy/blob/main/examples/rp/src/bin/wifi_blinky.rs)
    //
    // If you have a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here. Don't forget adding an appropriate resistor
    // in series with the LED.
    // let mut led_pin = pins.led.into_push_pull_output();

    // timer.delay_ms(100);

    // loop {
    //     led.set_high().unwrap();
    //     if !usb_dev.poll(&mut [&mut serial]) {
    //         continue;
    //     }
    //     let mut buf = [0u8; 64];

    //     match serial.read(&mut buf[..]) {
    //         Ok(count) => {
    //             // count bytes were read to &buf[..count]
    //         }
    //         Err(UsbError::WouldBlock) => {}
    //         Err(err) => {}
    //     };

    //     if (err) {
    //         match serial.write(&[0x3a, 0x29]) {
    //             Ok(count) => {
    //                 // count bytes were written
    //             }
    //             Err(UsbError::WouldBlock) => {}
    //             Err(err) => {}
    //         };
    //     }
    // }
    // loop {
    //     info!("on!");
    //     led_pin.set_high().unwrap();
    //     delay.delay_ms(500);
    //     info!("off!");
    //     led_pin.set_low().unwrap();
    //     delay.delay_ms(500);
    // }
}

// End of file
