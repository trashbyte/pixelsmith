use std::path::Path;
use imgui::{Condition, WindowFlags};

pub fn draw_recent_window(ui: &imgui::Ui, size: [f32; 2]) -> Option<(String, String)> {
    let recent_text = match std::env::current_dir() {
        Ok(cd) => match std::fs::read_to_string(cd.join("recent.yaml")) {
            Ok(text) => Some(text),
            Err(e) => { println!("{:?}", e); None }
        }
        Err(e) => { println!("{:?}", e); None }
    };
    let recent_projects: Vec<(String, String)> =
        if let Some(text) = recent_text {
            let mut entries = Vec::new();
            for (idx, line) in text.lines().enumerate() {
                let result = match line.find(":") {
                    Some(idx) => {
                        let (name, path) = line.split_at(idx);
                        let name = name.trim();
                        if name.len() == 0 || path.len() == 0 { None }
                        else {
                            let path = (&path[1..]).trim().trim_matches('"').to_string();
                            if path.len() == 0 { None }
                            else {
                                if Path::new(&path).exists() {
                                    Some((name, path))
                                }
                                else { None }
                            }
                        }
                    }
                    None => None
                };

                if let Some((name, path)) = result {
                    entries.push((name.to_string(), path.to_string()));
                }
                else {
                    println!("Invalid entry in recent projects list on line {}: '{}'\n    Expected format PROJECTNAME: \"PATH\"", idx+1, line);
                }
            }
            entries
        }
        else { Vec::new() };
    let items = recent_projects.iter().map(|(n, p)| format!("{}: \"{}\"", n, p)).collect::<Vec<_>>();
    //println!("parsed:\n  {}", items.join("\n"));
    let mut selected = -1;
    match ui.window("##startup-window")
        .flags(WindowFlags::NO_MOVE   | WindowFlags::NO_RESIZE   | WindowFlags::NO_TITLE_BAR
            | WindowFlags::NO_DOCKING | WindowFlags::NO_COLLAPSE | WindowFlags::NO_SAVED_SETTINGS)
        .position([0.0, 0.0], Condition::Always)
        .size(size, Condition::Always)
        .build(|| {
            ui.text("Open Project");
            ui.text("Recent Projects");
            if ui.list_box("##recent-proj-list", &mut selected, &items.iter().collect::<Vec<_>>()[..], items.len() as i32) {
                if selected >= 0 && selected < recent_projects.len() as i32 {
                    return Some(recent_projects[selected as usize].clone());
                }
            }
            None
        })
    {
        Some(res) => res,
        None => None
    }
}