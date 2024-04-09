use super::*;

impl<'a> Runtime<'a> {
    pub fn draw_memory_view(&mut self, chunks: &Rc<[Rect]>, buf: &mut Buffer) {
        if self.memory_view.disassembly {
            self.memory_view.page_offset = self
                .memory_view
                .page_offset
                .clamp(0, (PAGE_SIZE / 4) - chunks[1].height as usize + 1);
        } else {
            if self.memory_view.binary {
                self.memory_view.page_offset = self.memory_view.page_offset.clamp(
                    0,
                    (PAGE_SIZE / BYTES_PER_ROW_BINARY) - chunks[1].height as usize + 1,
                );
            } else {
                self.memory_view.page_offset = self.memory_view.page_offset.clamp(
                    0,
                    (PAGE_SIZE / BYTES_PER_ROW_HEXDEC) - chunks[1].height as usize + 1,
                );
            }
        }

        Paragraph::new(Line::from(vec![
            "↕".blue().bold(),
            " to scroll | ".into(),
            "↔".blue().bold(),
            " to switch pages | ".into(),
            "d".blue().bold(),
            " to toggle disassembly view | ".into(),
            "b".blue().bold(),
            " to toggle binary view".into(),
        ]))
        .block(
            Block::new()
                .borders(Borders::TOP)
                .padding(Padding::new(1, 1, 0, 0))
                .title(
                    Title::from(format!(
                        " {} | {} ",
                        self.memory_view.page, self.memory_view.page_offset
                    ))
                    .alignment(Alignment::Center),
                ),
        )
        .render(chunks[2], buf);

        if let Some(page) = self
            .pipeline
            .memory_module()
            .memory()
            .get_page(self.memory_view.page)
        {
            if self.memory_view.disassembly {
                let bytes: usize = 4;
                let pc = self.pipeline.registers().pc;

                let mut headers = vec!["Address".to_string()];
                headers.extend((0..bytes).into_iter().map(|i| format!("{i:02X}")));
                headers.push("Instruction".to_string());
                let mut columns = vec![Constraint::Max(10)];
                columns.extend(
                    (0..bytes)
                        .map(|_| Constraint::Max(if self.memory_view.binary { 8 } else { 2 })),
                );
                columns.push(Constraint::Fill(1));

                let table = Table::new(
                    page.chunks(bytes)
                        .enumerate()
                        .skip(self.memory_view.page_offset)
                        .take(chunks[1].height as usize + 1)
                        .enumerate()
                        .map(|(rid, (i, row))| {
                            let mut result = vec![format!(
                                "{:#010X}",
                                i * bytes + (self.memory_view.page << 16)
                            )];
                            result.extend(row.into_iter().map(|v| {
                                if self.memory_view.binary {
                                    format!("{v:08b}")
                                } else {
                                    format!("{v:02X}")
                                }
                            }));
                            result.push(
                                decode::<libseis::instruction_set::Instruction>(
                                    Word::from_be_bytes([row[0], row[1], row[2], row[3]]),
                                )
                                .map(|i| i.to_string())
                                .unwrap_or_default(),
                            );

                            if i * bytes == pc as usize {
                                Row::new(result).on_red()
                            } else if rid % 2 == 0 {
                                Row::new(result).on_light_blue()
                            } else {
                                Row::new(result).on_blue()
                            }
                        }),
                    columns,
                )
                .header(
                    Row::new(headers.into_iter().map(|h| h.bold()))
                        .on_blue()
                        .bold(),
                );

                table.render(chunks[1], buf);
            } else {
                let bytes: usize = if self.memory_view.binary {
                    BYTES_PER_ROW_BINARY
                } else {
                    BYTES_PER_ROW_HEXDEC
                };

                let mut headers = vec!["Address".to_string()];
                headers.extend((0..bytes).into_iter().map(|i| format!("{i:02X}")));
                let mut columns = vec![Constraint::Max(10)];
                columns.extend(
                    (0..bytes)
                        .map(|_| Constraint::Max(if self.memory_view.binary { 8 } else { 2 })),
                );

                let table = Table::new(
                    page.chunks(bytes)
                        .enumerate()
                        .skip(self.memory_view.page_offset)
                        .take(chunks[1].height as usize + 1)
                        .enumerate()
                        .map(|(rid, (i, row))| {
                            let mut result = vec![format!(
                                "{:#010X}",
                                i * bytes + (self.memory_view.page << 16)
                            )];
                            result.extend(row.into_iter().map(|v| {
                                if self.memory_view.binary {
                                    format!("{v:08b}")
                                } else {
                                    format!("{v:02X}")
                                }
                            }));
                            if rid % 2 == 0 {
                                Row::new(result).on_light_blue()
                            } else {
                                Row::new(result).on_blue()
                            }
                        }),
                    columns,
                )
                .header(
                    Row::new(headers.into_iter().map(|h| h.bold()))
                        .on_blue()
                        .bold(),
                );

                table.render(chunks[1], buf);
            }
        } else {
            let split =
                Layout::vertical([Constraint::Fill(1), Constraint::Min(1), Constraint::Fill(1)])
                    .split(chunks[1]);
            Block::new().on_blue().render(split[0], buf);
            Block::new().on_blue().render(split[2], buf);

            Paragraph::new("Not allocated")
                .centered()
                .block(Block::new().on_blue())
                .render(split[1], buf);
        }
    }
}
