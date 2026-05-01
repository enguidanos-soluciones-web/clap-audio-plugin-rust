use parley::{Alignment, AlignmentOptions, FontContext, Layout, LayoutContext, PositionedLayoutItem, StyleProperty};
use vello::{
    Glyph, Scene,
    kurbo::Affine,
    peniko::{Brush, Color, Fill},
};

#[allow(dead_code)]
pub enum TextSize {
    Xs, // 12px
    Sm, // 14px
    Md, // 16px
    Lg, // 18px
    Xl, // 20px
}

impl TextSize {
    pub fn px(&self) -> f32 {
        match self {
            TextSize::Xs => 12.0,
            TextSize::Sm => 14.0,
            TextSize::Md => 16.0,
            TextSize::Lg => 18.0,
            TextSize::Xl => 20.0,
        }
    }
}

pub struct TextRenderer {
    pub font_cx: FontContext,
    pub layout_cx: LayoutContext<Brush>,
}

impl TextRenderer {
    pub fn new() -> Self {
        let mut font_cx = FontContext::new();
        font_cx.collection.load_system_fonts();
        Self {
            font_cx,
            layout_cx: LayoutContext::new(),
        }
    }

    fn build_layout(&mut self, text: &str, size: f32, color: Color) -> Layout<Brush> {
        let brush = Brush::Solid(color);

        let mut builder = self.layout_cx.ranged_builder(&mut self.font_cx, text, 1.0, true);
        builder.push_default(StyleProperty::FontSize(size));
        builder.push_default(StyleProperty::Brush(brush));

        let mut layout: Layout<Brush> = builder.build(text);
        layout.break_all_lines(None);
        layout.align(Alignment::Start, AlignmentOptions::default());
        layout
    }

    fn draw_layout(&self, scene: &mut Scene, layout: &Layout<Brush>, x: f64, y: f64) {
        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };

                let run = glyph_run.run();
                let transform = Affine::translate((x, y));

                scene
                    .draw_glyphs(run.font())
                    .font_size(run.font_size())
                    .transform(transform)
                    .brush(&glyph_run.style().brush)
                    .draw(
                        Fill::NonZero,
                        glyph_run.positioned_glyphs().map(|g| Glyph {
                            id: g.id as u32,
                            x: g.x,
                            y: g.y,
                        }),
                    );
            }
        }
    }

    pub fn draw_centered(
        &mut self,
        scene: &mut Scene,
        text: &str,
        size: TextSize,
        color: Color,
        coordinates: (f64, f64),
        dimensions: (f64, f64),
    ) {
        let layout = self.build_layout(text, size.px(), color);

        let ascent = layout
            .lines()
            .next()
            .map(|l| l.metrics().ascent as f64)
            .unwrap_or(size.px() as f64 * 0.8);

        let tx = coordinates.0 + (dimensions.0 - layout.width() as f64) / 2.0;
        let ty = coordinates.1 + dimensions.1 / 2.0 - ascent * 0.65;
        self.draw_layout(scene, &layout, tx, ty);
    }
}
