use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::{ComponentHandle, Model, ModelRc, Rgb8Pixel, VecModel};
use std::rc::Rc;

slint::slint! {
    import { ModulesPage } from "ui/pages/modules.slint";
    import { DisplayCore, RemoteReceiverView } from "ui/components/ModuleView/module-display.slint";
    import { LedView } from "ui/module_definitions/led.slint";
    import { Theme } from "ui/shared/theme.slint";
    import { ModuleView, UiModuleType } from "ui/shared/types.slint";
    export { ModuleView, UiModuleType } from "ui/shared/types.slint";

    export component LayoutTest inherits Window {
        in property <[ModuleView]> modules;
        in property <UiModuleType> probe-type: UiModuleType.Led;
        out property <length> probe-width: probe.preferred-width;
        out property <length> probe-height: probe.preferred-height;
        out property <length> led-width: led.width;
        out property <length> led-height: led.height;
        out property <length> remote-width: remote.width;
        out property <length> remote-height: remote.height;
        out property <color> surface: Theme.surface;
        ModulesPage { module-list: root.modules; }
        // Offscreen probes compare the wrapper with the actual view geometry.
        probe := DisplayCore {
            x: -10000px;
            modData: { id: "probe", module-type: root.probe-type };
        }
        led := LedView { x: -10000px; module-id: "led"; }
        remote := RemoteReceiverView { x: -10000px; module-id: "remote"; }
    }
}

struct TestPlatform(Rc<MinimalSoftwareWindow>);

impl slint::platform::Platform for TestPlatform {
    fn create_window_adapter(
        &self,
    ) -> Result<Rc<dyn slint::platform::WindowAdapter>, slint::PlatformError> {
        Ok(self.0.clone())
    }
}

fn render(window: &MinimalSoftwareWindow, width: u32, height: u32) -> Vec<Rgb8Pixel> {
    window.set_size(slint::PhysicalSize::new(width, height));
    window.request_redraw();
    let mut pixels = vec![Rgb8Pixel::default(); (width * height) as usize];
    assert!(window.draw_if_needed(|renderer| {
        renderer.render(&mut pixels, width as usize);
    }));
    pixels
}

// Find connected card surfaces in the real rendered page. Overlapping cards
// merge into one region, so this catches painted overflow as well as bad sizing.
fn card_bounds(pixels: &[Rgb8Pixel], width: usize, surface: Rgb8Pixel) -> Vec<[usize; 4]> {
    let mut visited = vec![false; pixels.len()];
    let mut cards = Vec::new();
    for start in 0..pixels.len() {
        if visited[start] || pixels[start] != surface {
            continue;
        }
        let mut stack = vec![start];
        visited[start] = true;
        let mut bounds = [start % width, start / width, start % width, start / width];
        let mut area = 0;
        while let Some(i) = stack.pop() {
            let (x, y) = (i % width, i / width);
            bounds = [
                bounds[0].min(x),
                bounds[1].min(y),
                bounds[2].max(x),
                bounds[3].max(y),
            ];
            area += 1;
            for next in [
                (x > 0).then(|| i - 1),
                (x + 1 < width).then(|| i + 1),
                (i >= width).then(|| i - width),
                (i + width < pixels.len()).then(|| i + width),
            ]
            .into_iter()
            .flatten()
            {
                if !visited[next] && pixels[next] == surface {
                    visited[next] = true;
                    stack.push(next);
                }
            }
        }
        // Ignore isolated pixels inside text and controls.
        if area > 1000 {
            cards.push(bounds);
        }
    }
    cards
}

#[test]
fn modules_wrap_without_painted_overflow_and_scroll() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(TestPlatform(window.clone()))).unwrap();
    let ui = LayoutTest::new().unwrap();
    let modules = Rc::new(VecModel::from(vec![
        ModuleView {
            id: "led-a".into(),
            module_type: UiModuleType::Led,
        },
        ModuleView {
            id: "remote".into(),
            module_type: UiModuleType::RemoteReceiver,
        },
        ModuleView {
            id: "led-b".into(),
            module_type: UiModuleType::Led,
        },
    ]));
    ui.set_modules(ModelRc::from(modules.clone()));
    ui.show().unwrap();

    assert_eq!(ui.get_probe_width(), ui.get_led_width());
    assert_eq!(ui.get_probe_height(), ui.get_led_height());
    ui.set_probe_type(UiModuleType::RemoteReceiver);
    render(&window, 1100, 1000);
    assert_eq!(ui.get_probe_width(), ui.get_remote_width());
    assert_eq!(ui.get_probe_height(), ui.get_remote_height());

    let surface = ui.get_surface();
    let surface = Rgb8Pixel {
        r: surface.red(),
        g: surface.green(),
        b: surface.blue(),
    };
    let wide = card_bounds(&render(&window, 1100, 1000), 1100, surface);
    assert_eq!(wide.len(), 3, "wide card bounds: {wide:?}");
    assert!(wide.iter().all(|b| b[1] == wide[0][1]));
    assert!(
        wide[1][3] < wide[0][3],
        "different heights must be preserved"
    );
    for pair in wide.windows(2) {
        assert!(pair[1][0] > pair[0][2] + 12);
    }

    let narrow = card_bounds(&render(&window, 560, 1000), 560, surface);
    assert_eq!(narrow.len(), 3, "narrow card bounds: {narrow:?}");
    assert!(narrow.iter().all(|b| b[0] == narrow[0][0]));
    for pair in narrow.windows(2) {
        assert!(pair[1][1] > pair[0][3] + 12, "overlap: {narrow:?}");
    }

    // Mutate the live model: the next row must move when the active view grows.
    modules.set_row_data(
        1,
        ModuleView {
            id: "replacement-led".into(),
            module_type: UiModuleType::Led,
        },
    );
    let changed = card_bounds(&render(&window, 560, 1000), 560, surface);
    assert_eq!(changed.len(), 3);
    assert!(changed[2][1] > narrow[2][1]);
    assert_eq!(
        card_bounds(&render(&window, 1100, 1000), 1100, surface).len(),
        3
    );

    let before_scroll = render(&window, 560, 400);
    window.dispatch_event(slint::platform::WindowEvent::PointerScrolled {
        position: slint::LogicalPosition::new(100., 100.),
        delta_x: 0.,
        delta_y: -10000.,
    });
    let after_scroll = render(&window, 560, 400);
    assert_ne!(before_scroll, after_scroll, "overflow must scroll");
    let bottom = card_bounds(&after_scroll, 560, surface);
    assert!(
        bottom.last().unwrap()[3] < 399,
        "last card must be reachable in full"
    );

    modules.set_vec(Vec::new());
    render(&window, 560, 400);
    modules.push(ModuleView {
        id: "new-remote".into(),
        module_type: UiModuleType::RemoteReceiver,
    });
    assert_eq!(
        card_bounds(&render(&window, 560, 400), 560, surface).len(),
        1
    );
}
