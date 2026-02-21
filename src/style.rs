use iced::{
    Background, Border, Color, Shadow, Theme, border, color,
    widget::{container, scrollable},
};

pub fn main_scrollbar(theme: &Theme, status: scrollable::Status) -> scrollable::Style {
    let bg = match status {
        scrollable::Status::Active {
            is_horizontal_scrollbar_disabled: false,
            is_vertical_scrollbar_disabled: true,
        } => color!(0xff0000),
        scrollable::Status::Hovered {
            is_horizontal_scrollbar_hovered: false,
            is_vertical_scrollbar_hovered: true,
            ..
        } => theme.extended_palette().background.neutral.color,
        scrollable::Status::Dragged { .. } => theme.extended_palette().background.neutral.color,
        _ => theme.extended_palette().background.weaker.color,
    };
    let rail = scrollable::Rail {
        background: Some(Background::Color(Color::TRANSPARENT)),
        border: Border::default(),
        scroller: scrollable::Scroller {
            background: Background::Color(bg),
            border: Border {
                color: bg,
                width: 1.,
                radius: border::Radius::new(3.),
            },
        },
    };

    scrollable::Style {
        container: container::Style::default(),
        vertical_rail: rail,
        horizontal_rail: rail,
        gap: None,
        auto_scroll: scrollable::AutoScroll {
            background: Background::Color(color!(0xff0000)),
            border: Border::default(),
            shadow: Shadow::default(),
            icon: color!(0xff0000),
        },
    }
}
