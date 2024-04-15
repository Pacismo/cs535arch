use super::*;

impl<'a> Runtime<'a> {
    pub fn draw_pipeline_view(&mut self, chunks: Rc<[Rect]>, buf: &mut Buffer) {
        let stages = self.pipeline.stages();

        [
            (
                "Fetch",
                Self::draw_pipe_fetch_view as fn(&Self, &PipelineStages, Rect, &mut Buffer),
            ),
            (
                "Decode",
                Self::draw_pipe_decode_view as fn(&Self, &PipelineStages, Rect, &mut Buffer),
            ),
            (
                "Execute",
                Self::draw_pipe_execute_view as fn(&Self, &PipelineStages, Rect, &mut Buffer),
            ),
            (
                "Memory",
                Self::draw_pipe_memory_view as fn(&Self, &PipelineStages, Rect, &mut Buffer),
            ),
            (
                "Writeback",
                Self::draw_pipe_writeback_view as fn(&Self, &PipelineStages, Rect, &mut Buffer),
            ),
        ]
        .into_iter()
        .skip(self.pipeline_view.stage)
        .take(PIPELINE_STAGE_VIEW_COUNT)
        .zip(
            Layout::horizontal([Constraint::Fill(1); PIPELINE_STAGE_VIEW_COUNT])
                .split(chunks[1])
                .into_iter(),
        )
        .for_each(|((name, pfn), &blk)| {
            let block = Block::new()
                .border_type(BorderType::Rounded)
                .borders(Borders::ALL)
                .title(Line::from(vec!["Stage: ".into(), name.red().bold()]));

            let area = block.inner(blk);
            block.render(blk, buf);

            pfn(self, &stages, area, buf);
        });

        Paragraph::new(Line::from(vec!["↔".blue().bold(), " to scroll ".into()]))
            .block(
                Block::new()
                    .padding(Padding::new(1, 1, 0, 0))
                    .borders(Borders::TOP),
            )
            .render(chunks[2], buf);
    }

    fn draw_pipe_fetch_view(&self, stages: &PipelineStages, chunk: Rect, buf: &mut Buffer) {
        match stages.fetch.get_state() {
            libpipe::fetch::State::Idle => List::new(
                [Line::from(vec!["State: ".into(), "Idle".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
            libpipe::fetch::State::Waiting { clocks } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Waiting".red().bold()]),
                    Line::from(vec!["clocks: ".into(), clocks.to_string().red().bold()]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::fetch::State::Ready { instruction } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Ready".red().bold()]),
                    Line::from(vec!["Word: ".into(), instruction.to_string().red().bold()]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::fetch::State::Squashed { clocks } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Squashed".red().bold()]),
                    Line::from(vec![
                        "But waiting for ".into(),
                        clocks.to_string().red().bold(),
                        " clocks".into(),
                    ]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::fetch::State::Halted => List::new(
                [Line::from(vec!["State: ".into(), "Halted".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
        }
        .render(chunk, buf);
    }

    fn draw_pipe_decode_view(&self, stages: &PipelineStages, chunk: Rect, buf: &mut Buffer) {
        match stages.decode.get_state() {
            libpipe::decode::State::Decoding { word, .. } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Decoding".red().bold()]),
                    Line::from(vec!["Word: ".into(), word.to_string().red().bold()]),
                    Line::from(vec![
                        "As Instruction: ".into(),
                        decode::<Instruction>(*word)
                            .map(|i| i.to_string())
                            .unwrap_or("<UNKNOWN>".to_string())
                            .red()
                            .bold(),
                    ]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::decode::State::Ready { word, .. } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Ready".red().bold()]),
                    Line::from(vec!["Word: ".into(), word.to_string().red().bold()]),
                    Line::from(vec![
                        "As Instruction: ".into(),
                        decode::<Instruction>(*word)
                            .map(|i| i.to_string())
                            .unwrap_or("<UNKNOWN>".to_string())
                            .red()
                            .bold(),
                    ]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::decode::State::Idle => List::new(
                [Line::from(vec!["State: ".into(), "Idle".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
            libpipe::decode::State::Squashed => List::new(
                [Line::from(vec!["State: ".into(), "Squashed".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
            libpipe::decode::State::PrevSquash => List::new(
                [Line::from(vec!["State: ".into(), "Squashed".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
            libpipe::decode::State::Halted => List::new(
                [Line::from(vec!["State: ".into(), "Halted".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
        }
        .render(chunk, buf);
    }

    fn draw_pipe_execute_view(&self, stages: &PipelineStages, chunk: Rect, buf: &mut Buffer) {
        match stages.execute.get_state() {
            libpipe::execute::State::Idle => List::new(
                [Line::from(vec!["State: ".into(), "Idle".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
            libpipe::execute::State::Executing {
                instruction,
                wregs,
                rvals,
                clocks,
            } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Executing".red().bold()]),
                    Line::from(vec![
                        "Expected wait time: ".into(),
                        clocks.to_string().red().bold(),
                    ]),
                    Line::from(vec![
                        "Instruction: ".into(),
                        instruction.to_string().red().bold(),
                    ]),
                    Line::from(
                        ["Write: ".into()]
                            .into_iter()
                            .chain(wregs.registers().enumerate().flat_map(|(i, r)| {
                                if i == wregs.count() {
                                    vec![get_name(r).unwrap_or("<?>").red().bold()]
                                } else {
                                    vec![get_name(r).unwrap_or("<?>").red().bold(), ", ".into()]
                                }
                            }))
                            .collect::<Vec<_>>(),
                    ),
                ]
                .into_iter()
                .chain(
                    rvals
                        .iter()
                        .enumerate()
                        .map(|(i, r)| {
                            if i == wregs.count().saturating_sub(1) {
                                vec![
                                    get_name(r.register).unwrap_or("<?>").red().bold(),
                                    " = ".into(),
                                    r.value.to_string().red().bold(),
                                ]
                            } else {
                                vec![
                                    get_name(r.register).unwrap_or("<?>").red().bold(),
                                    " = ".into(),
                                    r.value.to_string().red().bold(),
                                    ", ".into(),
                                ]
                            }
                        })
                        .map(Line::from),
                )
                .map(ListItem::new),
            ),
            libpipe::execute::State::Ready { wregs, .. } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Ready".red().bold()]),
                    Line::from(
                        ["Write: ".into()]
                            .into_iter()
                            .chain(wregs.registers().enumerate().flat_map(|(i, r)| {
                                if i == wregs.count().saturating_sub(1) {
                                    vec![get_name(r).unwrap_or("<?>").red().bold()]
                                } else {
                                    vec![get_name(r).unwrap_or("<?>").red().bold(), ", ".into()]
                                }
                            }))
                            .collect::<Vec<_>>(),
                    ),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::execute::State::Squashed { wregs } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Squashed".red().bold()]),
                    Line::from(
                        ["Write: ".into()]
                            .into_iter()
                            .chain(wregs.registers().enumerate().flat_map(|(i, r)| {
                                if i == wregs.count().saturating_sub(1) {
                                    vec![get_name(r).unwrap_or("<?>").red().bold()]
                                } else {
                                    vec![get_name(r).unwrap_or("<?>").red().bold(), ", ".into()]
                                }
                            }))
                            .collect::<Vec<_>>(),
                    ),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::execute::State::Halted => List::new(
                [Line::from(vec!["State: ".into(), "Halted".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
        }
        .render(chunk, buf);
    }

    fn draw_pipe_memory_view(&self, stages: &PipelineStages, chunk: Rect, buf: &mut Buffer) {
        match stages.memory.get_state() {
            libpipe::memory::State::Idle => List::new(
                [Line::from(vec!["State: ".into(), "Idle".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
            libpipe::memory::State::Reading {
                mode,
                destination,
                clocks,
            } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Reading".red().bold()]),
                    Line::from(vec![
                        "Destination: ".into(),
                        get_name(*destination).unwrap_or("<?>").red().bold(),
                    ]),
                    Line::from(vec![
                        "Expected wait time: ".into(),
                        clocks.to_string().red().bold(),
                    ]),
                    Line::from(match mode {
                        libpipe::memory::ReadMode::ReadByte { volatile, .. } => {
                            vec![
                                "Mode: ".into(),
                                "byte".red().bold(),
                                if *volatile { " volatile" } else { "" }.blue().bold(),
                            ]
                        }
                        libpipe::memory::ReadMode::ReadShort { volatile, .. } => {
                            vec![
                                "Mode: ".into(),
                                "short".red().bold(),
                                if *volatile { " volatile" } else { "" }.blue().bold(),
                            ]
                        }
                        libpipe::memory::ReadMode::ReadWord { volatile, .. } => {
                            vec![
                                "Mode: ".into(),
                                "word".red().bold(),
                                if *volatile { " volatile" } else { "" }.blue().bold(),
                            ]
                        }
                    }),
                    Line::from(match mode {
                        libpipe::memory::ReadMode::ReadByte { address, .. }
                        | libpipe::memory::ReadMode::ReadShort { address, .. }
                        | libpipe::memory::ReadMode::ReadWord { address, .. } => {
                            vec!["Address: ".into(), address.to_string().red().bold()]
                        }
                    }),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::memory::State::Writing { mode, clocks } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Writing".red().bold()]),
                    Line::from(vec![
                        "Expected wait time: ".into(),
                        clocks.to_string().red().bold(),
                    ]),
                    Line::from(match mode {
                        libpipe::memory::WriteMode::WriteByte { volatile, .. } => {
                            vec![
                                "Mode: ".into(),
                                "byte".red().bold(),
                                if *volatile { " volatile" } else { "" }.blue().bold(),
                            ]
                        }
                        libpipe::memory::WriteMode::WriteShort { volatile, .. } => {
                            vec![
                                "Mode: ".into(),
                                "short".red().bold(),
                                if *volatile { " volatile" } else { "" }.blue().bold(),
                            ]
                        }
                        libpipe::memory::WriteMode::WriteWord { volatile, .. } => {
                            vec![
                                "Mode: ".into(),
                                "word".red().bold(),
                                if *volatile { " volatile" } else { "" }.blue().bold(),
                            ]
                        }
                    }),
                    Line::from(match mode {
                        libpipe::memory::WriteMode::WriteByte { address, .. }
                        | libpipe::memory::WriteMode::WriteShort { address, .. }
                        | libpipe::memory::WriteMode::WriteWord { address, .. } => {
                            vec!["Address: ".into(), address.to_string().red().bold()]
                        }
                    }),
                    Line::from(match mode {
                        libpipe::memory::WriteMode::WriteByte { value, .. } => {
                            vec!["Value: ".into(), value.to_string().red().bold()]
                        }
                        libpipe::memory::WriteMode::WriteShort { value, .. } => {
                            vec!["Value: ".into(), value.to_string().red().bold()]
                        }
                        libpipe::memory::WriteMode::WriteWord { value, .. } => {
                            vec!["Value: ".into(), value.to_string().red().bold()]
                        }
                    }),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::memory::State::Pushing { value, sp, clocks } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Pushing".red().bold()]),
                    Line::from(vec![
                        "Expected wait time: ".into(),
                        clocks.to_string().red().bold(),
                    ]),
                    Line::from(vec!["Value: ".into(), value.to_string().red().bold()]),
                    Line::from(vec![
                        "SP".red().bold(),
                        " = ".into(),
                        sp.to_string().red().bold(),
                    ]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::memory::State::Popping {
                destination,
                sp,
                clocks,
            } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Popping".red().bold()]),
                    Line::from(vec![
                        "Expected wait time: ".into(),
                        clocks.to_string().red().bold(),
                    ]),
                    Line::from(vec![
                        "Destination: ".into(),
                        get_name(*destination).unwrap_or("<?>").red().bold(),
                    ]),
                    Line::from(vec![
                        "SP".red().bold(),
                        " = ".into(),
                        sp.to_string().red().bold(),
                    ]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::memory::State::DummyPop { sp, clocks } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Popping".red().bold()]),
                    Line::from(vec![
                        "Expected wait time: ".into(),
                        clocks.to_string().red().bold(),
                    ]),
                    Line::from(vec!["Destination: ".into(), "invalid".red().bold()]),
                    Line::from(vec![
                        "SP".red().bold(),
                        " = ".into(),
                        sp.to_string().red().bold(),
                    ]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::memory::State::JsrPrep {
                address,
                link,
                sp,
                bp,
                clocks,
            } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Preparing for JSR".red().bold()]),
                    Line::from(vec![
                        "Expected wait time: ".into(),
                        clocks.to_string().red().bold(),
                    ]),
                    Line::from(vec!["Address: ".into(), address.to_string().red().bold()]),
                    Line::from(vec!["Link: ".into(), link.to_string().red().bold()]),
                    Line::from(vec![
                        "SP".red().bold(),
                        " = ".into(),
                        sp.to_string().red().bold(),
                    ]),
                    Line::from(vec![
                        "BP".red().bold(),
                        " = ".into(),
                        bp.to_string().red().bold(),
                    ]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::memory::State::RetPrep { link, bp, clocks } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Preparing for JSR".red().bold()]),
                    Line::from(vec![
                        "Expected wait time: ".into(),
                        clocks.to_string().red().bold(),
                    ]),
                    Line::from(vec!["Link: ".into(), link.to_string().red().bold()]),
                    Line::from(vec![
                        "BP".red().bold(),
                        " = ".into(),
                        bp.to_string().red().bold(),
                    ]),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::memory::State::Ready { .. } => List::new(
                [Line::from(vec!["State: ".into(), "Ready".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
            libpipe::memory::State::Squashed { wregs } => List::new(
                [
                    Line::from(vec!["State: ".into(), "Squashed".red().bold()]),
                    ["Locks: ".into()]
                        .into_iter()
                        .chain(wregs.registers().enumerate().flat_map(|(i, r)| {
                            if i == wregs.count().saturating_sub(1) {
                                vec![get_name(r).unwrap_or("<?>").red().bold()]
                            } else {
                                vec![get_name(r).unwrap_or("<?>").red().bold(), ", ".into()]
                            }
                        }))
                        .collect::<Vec<_>>()
                        .into(),
                ]
                .into_iter()
                .map(ListItem::new),
            ),
            libpipe::memory::State::Halting(..) => List::new(
                [Line::from(vec!["State: ".into(), "Halting".red().bold()])]
                    .into_iter()
                    .map(ListItem::from),
            ),
            libpipe::memory::State::Halted => List::new(
                [Line::from(vec!["State: ".into(), "Halted".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            ),
        }
        .render(chunk, buf);
    }

    fn draw_pipe_writeback_view(&self, stages: &PipelineStages, chunk: Rect, buf: &mut Buffer) {
        match stages.writeback.get_state() {
            Some(op) => {
                let op_str = format!("{op:#?}");
                let op = op_str.lines().map(Line::from).collect::<Vec<_>>();
                List::new(
                    [Line::from(vec!["State: ".into(), "Busy".red().bold()])]
                        .into_iter()
                        .chain(op)
                        .map(ListItem::new),
                )
                .render(chunk, buf)
            }
            None => List::new(
                [Line::from(vec!["State: ".into(), "Idle".red().bold()])]
                    .into_iter()
                    .map(ListItem::new),
            )
            .render(chunk, buf),
        }
    }
}
