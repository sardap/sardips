use bevy::prelude::*;

pub struct ButtonHoverPlugin;

impl Plugin for ButtonHoverPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update,
                add_missing_com,
                toggle_selected_colors,
                toggle_disabled_colors,
                on_remove,
            ),
        );
    }
}

#[derive(Copy, Clone)]
pub struct ButtonColorSet {
    pub normal: Color,
    pub hover: Color,
    pub pressed: Color,
    pub disabled: Color,
}

impl ButtonColorSet {
    pub const fn new(normal: Color, hover: Color, pressed: Color) -> Self {
        Self {
            normal,
            hover,
            pressed,
            disabled: normal,
        }
    }

    pub const fn with_disabled(mut self, disabled: Color) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Component, Default)]
pub struct ButtonHover {
    pub background: Option<ButtonColorSet>,
    pub border: Option<ButtonColorSet>,
}

impl ButtonHover {
    pub fn with_background(mut self, background: ButtonColorSet) -> Self {
        self.background = Some(background);
        self
    }

    pub fn with_border(mut self, border: ButtonColorSet) -> Self {
        self.border = Some(border);
        self
    }
}

fn add_missing_com(
    mut commands: Commands,
    mut new_buttons: Query<(Entity, &ButtonHover), Added<ButtonHover>>,
    existing_buttons_with_backgrounds: Query<Entity, With<Button>>,
    existing_buttons_with_borders: Query<Entity, With<Button>>,
) {
    for (entity, button_hover) in new_buttons.iter_mut() {
        if let Some(background) = &button_hover.background {
            if existing_buttons_with_backgrounds.get(entity).is_err() {
                commands
                    .entity(entity)
                    .insert(BackgroundColor(background.normal));
            }
        }

        if let Some(border) = &button_hover.border {
            if existing_buttons_with_borders.get(entity).is_err() {
                commands.entity(entity).insert(BorderColor(border.normal));
            }
        }
    }
}

fn update(
    mut interaction_query: Query<
        (
            &Interaction,
            Option<&mut BackgroundColor>,
            Option<&mut BorderColor>,
            &ButtonHover,
        ),
        (With<Button>, Without<Selected>),
    >,
) {
    for (interaction, background_color, border_color, button_hover) in interaction_query.iter_mut()
    {
        let (background, border) = match *interaction {
            Interaction::Hovered => (
                button_hover
                    .background
                    .as_ref()
                    .map(|background| background.hover),
                button_hover.border.as_ref().map(|border| border.hover),
            ),
            Interaction::None => (
                button_hover
                    .background
                    .as_ref()
                    .map(|background| background.normal),
                button_hover.border.as_ref().map(|border| border.normal),
            ),
            Interaction::Pressed => (
                button_hover
                    .background
                    .as_ref()
                    .map(|background| background.pressed),
                button_hover.border.as_ref().map(|border| border.pressed),
            ),
        };

        if let Some(mut background_color) = background_color {
            if let Some(background) = background {
                *background_color = BackgroundColor(background);
            }
        }

        if let Some(mut border_color) = border_color {
            if let Some(border) = border {
                *border_color = BorderColor(border);
            }
        }
    }
}

#[derive(Component)]
pub struct Selected;

fn toggle_selected_colors(
    mut buttons: Query<
        (
            &ButtonHover,
            Option<&mut BackgroundColor>,
            Option<&mut BorderColor>,
        ),
        With<Selected>,
    >,
) {
    for (hover, background, border) in buttons.iter_mut() {
        if let Some(hover_background) = &hover.background {
            if let Some(mut background) = background {
                *background = BackgroundColor(hover_background.pressed);
            }
        }

        if let Some(hover_border) = &hover.border {
            if let Some(mut border) = border {
                *border = BorderColor(hover_border.pressed);
            }
        }
    }
}

fn toggle_disabled_colors(
    mut buttons: Query<
        (Option<&mut BackgroundColor>, Option<&mut BorderColor>),
        (With<ButtonHover>, Without<Interaction>),
    >,
) {
    for (background, border) in buttons.iter_mut() {
        if let Some(mut background) = background {
            *background = BackgroundColor(Color::Srgba(bevy::color::palettes::css::DARK_GRAY));
        }

        if let Some(mut border) = border {
            *border = BorderColor(Color::Srgba(bevy::color::palettes::css::DARK_GRAY));
        }
    }
}

fn on_remove(
    mut removed: RemovedComponents<Interaction>,
    mut buttons: Query<(
        &ButtonHover,
        Option<&mut BackgroundColor>,
        Option<&mut BorderColor>,
    )>,
) {
    for entity in removed.read() {
        if let Ok((hover, background, border)) = buttons.get_mut(entity) {
            if let Some(mut background_color) = background {
                if let Some(background) = &hover.background {
                    *background_color = BackgroundColor(background.disabled);
                }
            }

            if let Some(mut border_color) = border {
                if let Some(border) = &hover.border {
                    *border_color = BorderColor(border.disabled);
                }
            }
        }
    }
}
