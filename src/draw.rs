use core::convert::Infallible;

use embedded_graphics::{
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
};
use qrcodegen::{QrCode, QrCodeEcc};
use qrcodegen_no_heap as qrcodegen;
use u8g2_fonts::{
    fonts,
    types::{FontColor, HorizontalAlignment, VerticalPosition},
    FontRenderer,
};

fn to_color(x: bool) -> BinaryColor {
    if x {
        BinaryColor::Off
    } else {
        BinaryColor::On
    }
}

fn draw_qr<T: DrawTarget<Color = BinaryColor, Error = Infallible>>(
    canvas: &mut T,
    qr: &QrCode,
    size: u32,
    top_left: Point,
) {
    let qr_size = qr.size();

    for x in 0..qr_size {
        for y in 0..qr_size {
            Rectangle::new(
                top_left + Point::new(x * size as i32, y * size as i32),
                Size::new(size, size),
            )
            .into_styled(
                PrimitiveStyleBuilder::new()
                    .fill_color(to_color(qr.get_module(x, y)))
                    .build(),
            )
            .draw(canvas)
            .unwrap();
        }
    }
}

fn qr_component<T: DrawTarget<Color = BinaryColor, Error = Infallible>>(canvas: &mut T) {
    let mut outbuffer = [0u8; qrcodegen::Version::MAX.buffer_len()];
    let mut tempbuffer = [0u8; qrcodegen::Version::MAX.buffer_len()];

    let qr = QrCode::encode_text(
        "https://yumi0n1p1.dev",
        &mut tempbuffer,
        &mut outbuffer,
        QrCodeEcc::High,
        qrcodegen::Version::MIN,
        qrcodegen::Version::MAX,
        None,
        true,
    )
    .unwrap();

    draw_qr(canvas, &qr, 4, Point::new(6, 6));
}

struct StackContext<'a, T: DrawTarget<Color = BinaryColor, Error = Infallible>> {
    canvas: &'a mut T,
    x: i32,
    y: i32,
    padding: u32,
}

impl<T: DrawTarget<Color = BinaryColor, Error = Infallible>> StackContext<'_, T> {
    fn new(canvas: &mut T, x: i32, padding: u32) -> StackContext<'_, T> {
        StackContext {
            canvas,
            x,
            y: 0,
            padding,
        }
    }

    fn draw_text(&mut self, content: &str, font: &FontRenderer, height: u32) {
        self.y += self.padding as i32;
        font.render_aligned(
            content,
            Point::new(self.x, self.y),
            VerticalPosition::Top,
            HorizontalAlignment::Left,
            FontColor::Transparent(BinaryColor::Off),
            self.canvas,
        )
        .unwrap();
        self.y += height as i32;
    }
}

const HEADING_HEIGHT: u32 = 24;
const BODY_HEIGHT: u32 = 12;

fn text_component<T: DrawTarget<Color = BinaryColor, Error = Infallible>>(canvas: &mut T) {
    let heading_font = FontRenderer::new::<fonts::u8g2_font_logisoso24_tf>();
    let body_font = FontRenderer::new::<fonts::u8g2_font_bytesize_tf>();
    let mut ctx = StackContext::new(canvas, 128, 6);

    ctx.draw_text("Kagurazaka", &heading_font, HEADING_HEIGHT);
    ctx.draw_text("Yumi", &heading_font, HEADING_HEIGHT);
    ctx.draw_text("Id @yumi0n1p1", &body_font, BODY_HEIGHT);
}

pub fn draw<T: DrawTarget<Color = BinaryColor, Error = Infallible>>(canvas: &mut T) {
    assert_eq!(
        canvas.bounding_box(),
        Rectangle::new(Point::new(0, 0), Size::new(296, 128))
    );

    qr_component(canvas);
    text_component(canvas);
}
