use embedded_graphics::{
    pixelcolor::Rgb565, prelude::*, primitives::{Triangle, PrimitiveStyle, Rectangle, Styled, PrimitiveStyleBuilder},
};

const STYLE: PrimitiveStyle<Rgb565> = PrimitiveStyleBuilder::new()
                .stroke_color(Rgb565::WHITE)
                .stroke_width(1)
                .fill_color(Rgb565::CSS_CADET_BLUE)
                .build();
const CURSOR_WIDTH: i32 = 4;


pub fn cursor_right(rightmost_point: Point) -> Styled<Triangle, PrimitiveStyle<Rgb565>> {
    Triangle::new(Point::new(0, 0), Point::new(-CURSOR_WIDTH, -CURSOR_WIDTH), Point::new(-CURSOR_WIDTH, CURSOR_WIDTH))
            .translate(rightmost_point)
            .into_styled(STYLE)
}

pub fn cursor_left(leftmost_point: Point) -> Styled<Triangle, PrimitiveStyle<Rgb565>> {
    Triangle::new(Point::new(0, 0), Point::new(CURSOR_WIDTH, -CURSOR_WIDTH), Point::new(CURSOR_WIDTH, CURSOR_WIDTH))
            .translate(leftmost_point)
            .into_styled(STYLE)
}

pub fn static_draw_two_cursors<S: DrawTarget<Color = Rgb565>>(center: Point, width: i32, s: &mut S) -> Result<(Rectangle, Rectangle), <S as DrawTarget>::Error> {
    let left_cursor = cursor_right(center - Point::new(width, 0));
    let right_cursor = cursor_left(center + Point::new(width, 0));
    left_cursor.draw(s)?;
    right_cursor.draw(s)?;
    Ok((left_cursor.bounding_box(), right_cursor.bounding_box()))
}