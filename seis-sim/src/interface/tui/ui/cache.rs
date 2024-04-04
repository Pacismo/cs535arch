use super::*;

impl<'a> Runtime<'a> {
    pub fn draw_cache_view(&mut self, chunks: &Rc<[Rect]>, buf: &mut Buffer) {
        let splits =
            Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).split(chunks[1]);

        Tabs::new(self.cache_view.names.clone())
            .block(Block::new().borders(Borders::BOTTOM))
            .select(self.cache_view.index)
            .highlight_style(Style::new().bold().blue())
            .render(splits[0], buf);

        let cache_name = &self.cache_view.names[self.cache_view.index];

        let cache_lines = self
            .pipeline
            .memory_module()
            .caches()
            .get(cache_name.as_str())
            .unwrap()
            .get_lines();

        let cache_ways = self
            .config
            .cache
            .get(cache_name)
            .map(|c| match c {
                crate::config::CacheConfiguration::Disabled => 0,
                crate::config::CacheConfiguration::Associative { ways, .. } => *ways,
            })
            .unwrap_or(0);

        // TODO: handle scroll
        self.cache_view.scroll = self.cache_view.scroll.clamp(
            0,
            cache_lines
                .len()
                .saturating_sub((splits[1].height) as usize),
        );

        let data = cache_lines
            .into_iter()
            .skip(self.cache_view.scroll)
            .take((splits[1].height) as usize)
            .enumerate()
            .map(|(i, line)| {
                let row = if let Some(line) = line {
                    Row::new(vec![
                        format!("{}", i / cache_ways),
                        format!("{}", i % cache_ways),
                        format!("{}", line.dirty),
                        format!("{:#010X}", line.base_address),
                        format!("{:?}", line.data),
                    ])
                } else {
                    Row::new(vec![
                        format!("{}", i / cache_ways),
                        format!("{}", i % cache_ways),
                        format!(""),
                        format!(""),
                        format!("Invalid"),
                    ])
                };

                if i % 2 == 0 {
                    row.on_light_blue()
                } else {
                    row.on_blue()
                }
            })
            .collect::<Vec<_>>();

        Table::new(
            data,
            vec![
                Constraint::Max(3),
                Constraint::Max(3),
                Constraint::Max(5),
                Constraint::Max(10),
                Constraint::Fill(1),
            ],
        )
        .header(
            Row::new(vec!["Set", "Way", "Dirty", "Address", "Data"])
                .on_blue()
                .bold(),
        )
        .render(splits[1], buf);

        Paragraph::new(Line::from(vec![
            "↕".blue().bold(),
            " to scroll | ".into(),
            "↔".blue().bold(),
            " to switch caches".into(),
        ]))
        .block(
            Block::new()
                .padding(Padding::new(1, 1, 0, 0))
                .borders(Borders::TOP),
        )
        .render(chunks[2], buf);
    }
}
