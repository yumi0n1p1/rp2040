#![no_std]
#![no_main]

use bsp::entry;
use embedded_hal::{
    digital::{OutputPin, PinState},
    spi::MODE_0,
};
use embedded_hal_bus::spi::ExclusiveDevice;
use panic_halt as _;

use pimoroni_badger2040::{
    self as bsp,
    hal::{fugit::RateExtU32, gpio::FunctionSpi, Spi, Timer},
};

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use uc8151::{blocking::Uc8151, LUT};

mod draw;

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

    let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // let usb_bus = UsbBusAllocator::new(UsbBus::new(
    //     pac.USBCTRL_REGS,
    //     pac.USBCTRL_DPRAM,
    //     clocks.usb_clock,
    //     true,
    //     &mut pac.RESETS,
    // ));

    // let mut serial = SerialPort::new(&usb_bus);
    // let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0xdead, 0xbeef))
    //     .device_class(USB_CLASS_CDC)
    //     .build();

    // let p3v3_en = pins.p3v3_en.into_push_pull_output_in_state(PinState::High);
    // let sw_a = pins.sw_a.into_pull_down_input();
    // let sw_b = pins.sw_b.into_pull_down_input();
    // let sw_c = pins.sw_c.into_pull_down_input();
    // let sw_up = pins.sw_up.into_pull_down_input();
    // let sw_down = pins.sw_down.into_pull_down_input();
    // let user_sw = pins.user_sw.into_pull_up_input();
    // let vbus_detect = pins.vbus_detect.into_pull_up_input();

    let mut led = pins.led.into_push_pull_output_in_state(PinState::Low);
    led.set_high().unwrap();

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
    draw::draw(&mut uc8151);
    uc8151.update().unwrap();

    loop {}
}

// End of file
