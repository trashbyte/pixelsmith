use imgui::{StyleColor, StyleVar, TableColumnFlags, TableColumnSetup, TableFlags, TableRowFlags};
use toolbelt::{Color, ColorSpace};

pub struct PaletteEditor {
    pub colors: Vec<Color>,
    pub selected_idx: usize,
}

impl PaletteEditor {
    pub fn new() -> Self {
        PaletteEditor {
            colors: vec! {
                Color::from_hsv(0.0, 0.0, 0.0),
                Color::from_hsv(0.0, 0.0, 0.2),
                Color::from_hsv(0.0, 0.0, 0.4),
                Color::from_hsv(0.0, 0.0, 0.6),
                Color::from_hsv(0.0, 0.0, 0.8),
                Color::from_hsv(0.0, 0.0, 1.0),

                Color::from_hsv(0.0/12.0, 0.8, 0.7),
                Color::from_hsv(1.0/12.0, 0.8, 0.7),
                Color::from_hsv(2.0/12.0, 0.8, 0.7),
                Color::from_hsv(3.0/12.0, 0.8, 0.7),
                Color::from_hsv(4.0/12.0, 0.8, 0.7),
                Color::from_hsv(5.0/12.0, 0.8, 0.7),
                Color::from_hsv(6.0/12.0, 0.8, 0.7),
                Color::from_hsv(7.0/12.0, 0.8, 0.7),
                Color::from_hsv(8.0/12.0, 0.8, 0.7),
                Color::from_hsv(9.0/12.0, 0.8, 0.7),
                Color::from_hsv(10.0/12.0, 0.8, 0.7),
                Color::from_hsv(11.0/12.0, 0.8, 0.7),
            },
            selected_idx: 0
        }
    }

    pub fn draw(&mut self, ui: &imgui::Ui) {
        ui.window("Palette").build(|| {
            if let Some(_token) = ui.begin_table_with_flags("##palette-table-top", 3,
                                                            TableFlags::NO_PAD_INNER_X) {
                ui.table_setup_column_with(TableColumnSetup {
                    name: "##one",
                    init_width_or_weight: 1000.0,
                    flags: TableColumnFlags::WIDTH_STRETCH,
                    ..Default::default()
                });
                ui.table_setup_column_with(TableColumnSetup {
                    name: "##two",
                    init_width_or_weight: 20.0,
                    flags: TableColumnFlags::WIDTH_FIXED,
                    ..Default::default()
                });
                ui.table_setup_column_with(TableColumnSetup {
                    name: "##three",
                    init_width_or_weight: 20.0,
                    flags: TableColumnFlags::WIDTH_FIXED,
                    ..Default::default()
                });
                ui.table_next_row();
                ui.table_next_column();
                ui.text("Palette");
                ui.table_next_column();
                if ui.button("+") {}
                ui.table_next_column();
                if ui.button("-") {}
            }

            let palette_width = ui.current_column_width() - 4.0;
            let swatch_size = 22.0;
            let swatches_per_row = ((palette_width / (swatch_size + 3.0)).floor() as usize).max(1);

            if let Some(_token) = ui.begin_table_with_sizing("##palette-table",
                                                             50,
                                                             TableFlags::SIZING_FIXED_SAME | TableFlags::NO_PAD_INNER_X,
                                                             [0.0, 0.0],
                                                             swatch_size) {
                for i in 0..self.colors.len() {
                    let _id = ui.push_id(i.to_string());
                    let _s1 = ui.push_style_var(StyleVar::FrameBorderSize(0.5));
                    let _s2 = ui.push_style_color(StyleColor::Border, [1.0, 1.0, 1.0, 0.1]);

                    if i % swatches_per_row == 0 {
                        ui.table_next_row_with_height(TableRowFlags::empty(), swatch_size + 5.0);
                    }

                    ui.table_next_column();

                    let [x, y] = ui.cursor_pos();
                    if i == self.selected_idx {
                        ui.set_cursor_pos([x - 3.0, y - 5.0]);
                    } else {
                        ui.set_cursor_pos([x + 1.0, y - 2.0]);
                    }

                    if ui.color_button_config("##palette", *self.colors[i].to_hsv().components_4())
                        .flags(imgui::ColorEditFlags::INPUT_HSV | imgui::ColorEditFlags::NO_PICKER | imgui::ColorEditFlags::NO_ALPHA | imgui::ColorEditFlags::NO_TOOLTIP)
                        .size(if i == self.selected_idx { [swatch_size + 6.0, swatch_size + 6.0] } else { [swatch_size - 2.0, swatch_size - 2.0] })
                        .build()
                    {
                        self.selected_idx = i;
                    }

                    if let Some(target) = ui.drag_drop_target() {
                        if let Some(payload) = unsafe { target.accept_payload_unchecked("_COL3F", imgui::DragDropFlags::SOURCE_NO_PREVIEW_TOOLTIP) } {
                            assert_eq!(payload.size, std::mem::size_of::<f32>() * 3);
                            let [r, g, b] = unsafe { *payload.data.cast::<[f32; 3]>() };
                            self.colors[i] = Color::from_rgb(r, g, b).to_hsv();
                        }
                    }
                }
            }
        });

        ui.window("Color Picker").build(|| {
            // make sure source is in HSV...
            self.colors[self.selected_idx].convert(ColorSpace::HSV);
            // ...then get a mutable reference to the source color
            let color = &mut self.colors[self.selected_idx];

            ui.color_button_config("##current-color", *color.components_4())
                .flags(imgui::ColorEditFlags::INPUT_HSV | imgui::ColorEditFlags::NO_PICKER | imgui::ColorEditFlags::NO_ALPHA | imgui::ColorEditFlags::NO_TOOLTIP)
                .size([ui.current_column_width(), 24.0])
                .build();

            ui.set_next_item_width(ui.window_content_region_max()[0]);
            ui.color_picker3_config("##picker", color.components_3_mut())
                .flags(imgui::ColorEditFlags::INPUT_HSV | imgui::ColorEditFlags::NO_SMALL_PREVIEW | imgui::ColorEditFlags::NO_SIDE_PREVIEW | imgui::ColorEditFlags::FLOAT)
                .build();
        });
    }
}