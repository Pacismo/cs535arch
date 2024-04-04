use super::*;

impl<'a> Widget for &mut Runtime<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let app_area = self.draw_app_frame(area, buf);

        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(app_area);

        self.draw_tabs(chunks[0], buf);

        self.draw_view(chunks, buf);

        if self.running.enabled {
            self.draw_running_window(area, buf);
        }
    }
}

impl<'a> Runtime<'a> {
    fn draw_running_window(&mut self, area: Rect, buf: &mut Buffer) {
        const HEIGHT: u16 = 5;
        const WIDTH: u16 = 64;
        let center = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(WIDTH),
            Constraint::Fill(1),
        ])
        .split(
            Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(HEIGHT),
                Constraint::Fill(1),
            ])
            .split(area)[1],
        )[1];

        Clear.render(center, buf);

        let block = Block::bordered()
            .title(Title::from(Line::from(
                " Running Program... ".rapid_blink().on_blue(),
            )))
            .title_bottom(Line::from(vec![
                " Press ".into(),
                "[ESC]".blue().bold(),
                " to stop ".into(),
            ]))
            .border_type(BorderType::Rounded);

        Canvas::default()
            .block(block)
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 2.0])
            .marker(symbols::Marker::Block)
            .paint(|c| {
                use ratatui::widgets::canvas::*;

                c.draw(&Rectangle {
                    x: (85 - (self.running.progress_bar - 85).abs()) as f64,
                    y: 1.0,
                    width: 15.0,
                    height: 0.0,
                    color: Color::Cyan,
                })
            })
            .render(center, buf);

        self.running.progress_bar = (self.running.progress_bar + 5) % 170;
    }

    fn draw_tabs(&self, area: Rect, buf: &mut Buffer) {
        Tabs::new(["Registers", "Memory", "Cache", "Pipeline"])
            .block(Block::new().borders(Borders::BOTTOM))
            .style(Style::default().white())
            .highlight_style(Style::default().white().on_blue().bold())
            .select(self.view.index())
            .render(area, buf);
    }

    fn draw_app_frame(&self, area: Rect, buf: &mut Buffer) -> Rect {
        let title = Title::from(" SEIS Simulation Terminal Fronend ".bold().on_blue())
            .alignment(Alignment::Center);
        let instructions = [
            [" q".blue().bold(), " to quit ".into()],
            [" c".blue().bold(), " to clock ".into()],
            [" r".blue().bold(), " to run ".into()],
            [" s".blue().bold(), " for slow run ".into()],
        ];

        let module = self.pipeline.memory_module();

        let application_block = if self.running.enabled {
            instructions
                .into_iter()
                .fold(Block::new(), |b, i| b.title_bottom(i.to_vec()))
                .title(title)
                .title_bottom(
                    Line::from(vec![
                        " Clocks: ".into(),
                        self.clocks.to_string().red().bold(),
                        " | Hits: ".into(),
                        module.cache_hits().to_string().red().bold(),
                        " | Misses: ".into(),
                        module.total_misses().to_string().red().bold(),
                        " | Accesses: ".into(),
                        module.accesses().to_string().red().bold(),
                        " | Evictions: ".into(),
                        module.evictions().to_string().red().bold(),
                        " ".into(),
                    ])
                    .alignment(Alignment::Right),
                )
                .borders(Borders::all())
                .border_set(border::DOUBLE)
                .dim()
        } else {
            instructions
                .into_iter()
                .fold(Block::new(), |b, i| b.title_bottom(i.to_vec()))
                .title(title)
                .title_bottom(
                    Line::from(vec![
                        " Clocks: ".into(),
                        self.clocks.to_string().red().bold(),
                        " | Hits: ".into(),
                        module.cache_hits().to_string().red().bold(),
                        " | Misses: ".into(),
                        module.total_misses().to_string().red().bold(),
                        " | Accesses: ".into(),
                        module.accesses().to_string().red().bold(),
                        " | Evictions: ".into(),
                        module.evictions().to_string().red().bold(),
                        " ".into(),
                    ])
                    .alignment(Alignment::Right),
                )
                .borders(Borders::all())
                .border_set(border::DOUBLE)
        };

        let application_area = application_block.inner(area);

        application_block.render(area, buf);

        application_area
    }

    fn draw_view(&mut self, chunks: Rc<[Rect]>, buf: &mut Buffer) {
        match &self.view {
            View::Registers => {
                self.draw_registers_view(&chunks, buf);
            }
            View::Memory => {
                self.draw_memory_view(&chunks, buf);
            }
            View::Cache => {
                self.draw_cache_view(&chunks, buf);
            }
            View::PipelineStages => {
                self.draw_pipeline_view(chunks, buf);
            }
        }
    }
}
