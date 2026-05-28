use super::AuroraRunner;
use super::ime;
use egui_rotate::{transform_clipped_primitives, transform_raw_input};
use glutin::context::PossiblyCurrentGlContext;

impl AuroraRunner {
    pub fn redraw_main(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let app_gl = self.app_gl.as_ref().unwrap();
        let main_window = self.main_state.aurora_window.as_mut().unwrap();

        // Apply visibility changes if the user modified them
        if self.platform.frame.statusbar_visible != self.platform.last_applied_statusbar {
            self.platform.last_applied_statusbar = self.platform.frame.statusbar_visible;
            main_window.set_statusbar_visible(self.platform.frame.statusbar_visible);
        }

        let window = main_window.window();

        // Make main context current
        app_gl
            .gl_context
            .make_current(&self.main_state.surface.as_ref().unwrap().gl_surface)
            .expect("failed to make main context current");

        // -- Custom egui run with optional rotation --
        let mut raw_input = self
            .main_state
            .egui_glow
            .as_mut()
            .unwrap()
            .egui_winit
            .take_egui_input(window);

        // Inject pending maliit events before taking disjoint field borrows.
        ime::process_maliit_events(
            &mut self.main_state.pending_ime_events,
            &mut self.main_state.keyboard_area,
            &self.main_state.egui_glow,
            &mut raw_input,
        );

        let app = self.app_state.app.as_mut().unwrap();

        // Reserve space for status bar and keyboard via safe_area_insets
        // instead of Panels, which can cause text-input focus loss.
        let is_landscape = matches!(
            self.main_state.rotation,
            egui_rotate::Rotation::CW90 | egui_rotate::Rotation::CW270
        );
        // statusbar_height comes from aurora_services in physical pixels,
        // so convert to points before using as an inset.
        let top_px = if self.platform.frame.statusbar_visible && !is_landscape {
            self.platform.statusbar_height
        } else {
            0.0
        };

        let bottom_px = self.main_state.keyboard_area.map_or(0.0, |area| {
            if !is_landscape {
                area.height()
            } else {
                area.width()
            }
        });

        let logical_insets = egui::epaint::MarginF32 {
            left: 0.0,
            right: 0.0,
            top: top_px / self.main_state.pixels_per_point,
            bottom: bottom_px / self.main_state.pixels_per_point,
        };

        // Always set safe_area_insets (even when zero) so egui clears
        // any previous insets when the keyboard or statusbar hides.
        raw_input.safe_area_insets = Some(egui::SafeAreaInsets(logical_insets));

        transform_raw_input(&mut raw_input, self.main_state.rotation);

        let full_output =
            self.main_state
                .egui_glow
                .as_ref()
                .unwrap()
                .egui_ctx
                .run_ui(raw_input, |ui| {
                    app.update(ui, &mut self.platform.frame);
                });

        // Process viewport commands
        if full_output.viewport_output.len() > 1 {
            log::warn!("Multiple viewports not yet supported by aurora_egui");
        }
        for (_, egui::ViewportOutput { commands, .. }) in full_output.viewport_output {
            let mut actions_requested = Default::default();
            egui_winit::process_viewport_commands(
                &self.main_state.egui_glow.as_ref().unwrap().egui_ctx,
                &mut self.main_state.viewport_info,
                commands,
                window,
                &mut actions_requested,
            );
            for action in actions_requested {
                log::warn!("{:?} not yet supported by aurora_egui", action);
            }
        }

        self.main_state
            .egui_glow
            .as_mut()
            .unwrap()
            .egui_winit
            .handle_platform_output(window, full_output.platform_output);

        self.main_state.shapes = full_output.shapes;
        self.main_state.pixels_per_point = full_output.pixels_per_point;
        self.main_state.textures_delta = full_output.textures_delta;

        event_loop.set_control_flow(
            if self.platform.repaint_delay.map_or(false, |d| d.is_zero()) {
                window.request_redraw();
                winit::event_loop::ControlFlow::Poll
            } else if let Some(repaint_after_instant) = self
                .platform
                .repaint_delay
                .and_then(|d| std::time::Instant::now().checked_add(d))
            {
                winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
            } else {
                winit::event_loop::ControlFlow::Wait
            },
        );

        unsafe {
            use glow::HasContext;
            // Use the app's panel fill color for clearing so safe-area
            // inset bands match the application style instead of showing
            // as transparent (which often looks black on Aurora OS).
            let panel_fill = self
                .main_state
                .egui_glow
                .as_ref()
                .unwrap()
                .egui_ctx
                .global_style()
                .visuals
                .panel_fill;
            let clear_color = panel_fill.to_normalized_gamma_f32();
            app_gl.glow.clear_color(
                clear_color[0],
                clear_color[1],
                clear_color[2],
                clear_color[3],
            );
            app_gl.glow.clear(glow::COLOR_BUFFER_BIT);
        }

        // -- Custom paint with optional rotation --
        let shapes = std::mem::take(&mut self.main_state.shapes);
        let mut textures_delta = std::mem::take(&mut self.main_state.textures_delta);

        {
            let main_egui_glow = self.main_state.egui_glow.as_mut().unwrap();
            for (id, image_delta) in textures_delta.set {
                main_egui_glow.painter.set_texture(id, &image_delta);
            }

            let pixels_per_point = self.main_state.pixels_per_point;
            let mut clipped_primitives =
                main_egui_glow.egui_ctx.tessellate(shapes, pixels_per_point);

            let logical_size = main_egui_glow.egui_ctx.viewport_rect().size();
            transform_clipped_primitives(
                &mut clipped_primitives,
                self.main_state.rotation,
                logical_size,
            );

            let dimensions: [u32; 2] = window.inner_size().into();
            main_egui_glow.painter.paint_primitives(
                dimensions,
                pixels_per_point,
                &clipped_primitives,
            );

            for id in textures_delta.free.drain(..) {
                main_egui_glow.painter.free_texture(id);
            }
        }

        self.main_state
            .surface
            .as_ref()
            .unwrap()
            .swap_buffers(&app_gl.gl_context)
            .unwrap();
        window.set_visible(true);
    }

    pub fn redraw_cover(&mut self) {
        let app = self.app_state.app.as_mut().unwrap();
        let cover = self.cover.as_mut().unwrap();
        let app_gl = self.app_gl.as_ref().unwrap();
        cover.render(app_gl, app.as_mut());
    }

    pub fn redraw(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
    ) {
        if self.is_main_window(window_id) {
            self.redraw_main(event_loop);
        } else if self.is_cover_window(window_id) {
            self.redraw_cover();
        } else {
            log::warn!("Redraw requested for unknown window: {:?}", window_id);
        }
    }
}
