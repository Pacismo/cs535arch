use crate::{config::SimulationConfiguration, PAGES};
use crossterm::event;
use libpipe::{Pipeline, PipelineStage};
use libseis::{
    instruction_set::{decode, Instruction},
    registers::{get_name, PC},
    types::{Register, Word},
};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::{Buffer, CrosstermBackend, Stylize, Terminal},
    style::Style,
    symbols::border,
    text::Line,
    widgets::{
        block::Title, Block, BorderType, Borders, List, ListItem, Padding, Paragraph, Row, Table,
        Tabs, Widget,
    },
    Frame,
};
use std::{error::Error, io::Stdout};

pub type Term = Terminal<CrosstermBackend<Stdout>>;

#[derive(Default, Debug, Clone, Copy)]
enum View {
    #[default]
    Registers,
    Memory,
    Cache,
    PipelineStages,
}

impl View {
    fn index(&self) -> usize {
        match self {
            View::Registers => 0,
            View::Memory => 1,
            View::Cache => 2,
            View::PipelineStages => 3,
        }
    }

    fn next(self) -> Self {
        match self {
            Self::Registers => Self::Memory,
            Self::Memory => Self::Cache,
            Self::Cache => Self::PipelineStages,
            Self::PipelineStages => Self::Registers,
        }
    }

    fn previous(self) -> Self {
        match self {
            Self::Registers => Self::PipelineStages,
            Self::Memory => Self::Registers,
            Self::Cache => Self::Memory,
            Self::PipelineStages => Self::Cache,
        }
    }
}

#[derive(Debug)]
pub struct Runtime<'a> {
    config: SimulationConfiguration,

    view: View,
    pipeline: &'a mut dyn Pipeline,
    clocks: usize,
    clocks_required: usize,

    page: usize,
    page_offset: usize,
    disassembly: bool,
    binary: bool,

    cache_index: usize,
    cache_count: usize,
    caches: Vec<String>,
}

impl<'a> Runtime<'a> {
    pub fn new(pipeline: &'a mut dyn Pipeline, config: SimulationConfiguration) -> Self {
        let cache_count = pipeline.memory_module().caches().len();
        let caches = config.cache.keys().map(|s| s.to_owned()).collect();

        Self {
            config,

            view: View::default(),
            pipeline,
            clocks: 0,
            clocks_required: 1,

            page: 0,
            page_offset: 0,
            disassembly: false,
            binary: false,

            cache_index: 0,
            cache_count,
            caches,
        }
    }

    pub fn run(&mut self, terminal: &mut Term) -> Result<(), Box<dyn Error>> {
        loop {
            terminal.draw(|frame| self.draw_frame(frame))?;

            if !self.process_input()? {
                break;
            }
        }

        Ok(())
    }

    fn draw_frame<'f>(&mut self, frame: &mut Frame<'f>) {
        frame.render_widget(self, frame.size());
    }

    fn process_input(&mut self) -> Result<bool, Box<dyn Error>> {
        use event::*;
        match event::read()? {
            Event::Key(key) => match key.kind {
                KeyEventKind::Press => match key.code {
                    KeyCode::Char('q') => Ok(false),

                    KeyCode::Char('1') => {
                        self.view = View::Registers;
                        Ok(true)
                    }
                    KeyCode::Char('2') => {
                        self.view = View::Memory;
                        Ok(true)
                    }
                    KeyCode::Char('3') => {
                        self.view = View::Cache;
                        Ok(true)
                    }
                    KeyCode::Char('4') => {
                        self.view = View::PipelineStages;
                        Ok(true)
                    }

                    KeyCode::Tab => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            self.view = self.view.previous()
                        } else {
                            self.view = self.view.next();
                        }

                        Ok(true)
                    }

                    KeyCode::Up => {
                        if matches!(self.view, View::Memory) {
                            self.page_offset = self.page_offset.saturating_sub(1);
                        }

                        Ok(true)
                    }
                    KeyCode::Down => {
                        if matches!(self.view, View::Memory) {
                            self.page_offset = self.page_offset.saturating_add(1);
                        }

                        Ok(true)
                    }
                    KeyCode::Left => {
                        if matches!(self.view, View::Memory) {
                            self.page_offset = 0;
                            self.page = self.page.saturating_sub(1).clamp(0, PAGES - 1);
                        } else if matches!(self.view, View::Cache) {
                            self.cache_index = self.cache_index.saturating_sub(1);
                        }

                        Ok(true)
                    }
                    KeyCode::Right => {
                        if matches!(self.view, View::Memory) {
                            self.page_offset = 0;
                            self.page = self.page.saturating_add(1).clamp(0, PAGES - 1);
                        } else if matches!(self.view, View::Cache) {
                            self.cache_index = self.cache_index.saturating_add(1);

                            if self.cache_index >= self.cache_count {
                                self.cache_index = self.cache_count - 1;
                            }
                        }

                        Ok(true)
                    }

                    KeyCode::Char('d') => {
                        if matches!(self.view, View::Memory) {
                            self.disassembly = !self.disassembly;
                        }

                        Ok(true)
                    }

                    KeyCode::Char('b') => {
                        if matches!(self.view, View::Memory) {
                            self.binary = !self.binary;
                        }

                        Ok(true)
                    }

                    KeyCode::Char('c') => {
                        if self.clocks_required != 0 {
                            self.clocks_required = match self.pipeline.clock(1) {
                                libpipe::ClockResult::Stall(clocks) => clocks,
                                libpipe::ClockResult::Flow => 1,
                                libpipe::ClockResult::Dry => 0,
                            };
                            self.clocks += 1;
                        }
                        Ok(true)
                    }

                    KeyCode::Char('f') => {
                        if self.clocks_required != 0 {
                            self.clocks += self.clocks_required;
                            self.clocks_required = match self.pipeline.clock(self.clocks_required) {
                                libpipe::ClockResult::Stall(clocks) => clocks,
                                libpipe::ClockResult::Flow => 1,
                                libpipe::ClockResult::Dry => 0,
                            };
                        }

                        Ok(true)
                    }

                    _ => Ok(true),
                },

                _ => Ok(true),
            },

            _ => Ok(true),
        }
    }
}

impl<'a> Widget for &mut Runtime<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        // Draw the outer box

        let title =
            Title::from(" SEIS Simulation Terminal Fronend ".bold()).alignment(Alignment::Center);
        let instructions = Line::from(vec![
            " q".blue().bold(),
            " to quit ".into(),
            " c".blue().bold(),
            " to clock ".into(),
            " f".blue().bold(),
            " to clock ".into(),
            self.clocks_required.to_string().red(),
            if self.clocks_required == 1 {
                " time ".into()
            } else {
                " times ".into()
            },
        ]);

        let module = self.pipeline.memory_module();

        let application_block = Block::new()
            .title(title)
            .title_bottom(instructions)
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
                    " ".into(),
                ])
                .alignment(Alignment::Right),
            )
            .borders(Borders::all())
            .border_set(border::DOUBLE);

        let chunks = Layout::vertical([
            Constraint::Length(2),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(application_block.inner(area));
        application_block.render(area, buf);

        Tabs::new(["Registers", "Memory", "Cache", "Pipeline"])
            .block(Block::new().borders(Borders::BOTTOM))
            .style(Style::default().white())
            .highlight_style(Style::default().white().on_blue().bold())
            .select(self.view.index())
            .render(chunks[0], buf);

        let help_block = Block::new()
            .padding(Padding::new(1, 1, 0, 0))
            .borders(Borders::TOP);

        match &self.view {
            View::Registers => {
                let lines = chunks[1].height as usize;
                let splits = Layout::horizontal(vec![Constraint::Fill(1); lines]).split(chunks[1]);

                self.pipeline
                    .registers()
                    .iter()
                    .enumerate()
                    .map(|(i, val)| {
                        Line::from(vec![
                            format!("{}: ", get_name(i as Register).unwrap()).into(),
                            if i as Register == PC {
                                format!("{val:#010X}").blue().bold()
                            } else {
                                format!("{val}").blue().bold()
                            },
                        ])
                    })
                    .collect::<Vec<_>>()
                    .chunks(lines)
                    .enumerate()
                    .for_each(|(i, x)| {
                        List::new(x.into_iter().map(|x| x.clone())).render(splits[i], buf)
                    });
            }
            View::Memory => {
                self.page_offset = self.page_offset.clamp(0, (chunks[1].height - 1) as usize);

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
                    help_block.title(
                        Title::from(format!(" {} | {} ", self.page, self.page_offset))
                            .alignment(Alignment::Center),
                    ),
                )
                .render(chunks[2], buf);

                if let Some(page) = self.pipeline.memory_module().memory().get_page(self.page) {
                    if self.disassembly {
                        const BYTES: usize = 4;
                        let pc = self.pipeline.registers().pc.wrapping_sub(4);

                        let mut headers = vec!["Address".to_string()];
                        headers.extend((0..BYTES).into_iter().map(|i| format!("{i:02X}")));
                        headers.push("Instruction".to_string());
                        let mut columns = vec![Constraint::Max(10)];
                        columns.extend(
                            (0..BYTES).map(|_| Constraint::Max(if self.binary { 8 } else { 2 })),
                        );
                        columns.push(Constraint::Fill(1));

                        let table = Table::new(
                            page.chunks(BYTES)
                                .enumerate()
                                .skip(self.page_offset)
                                .take((chunks[1].height - 1) as usize)
                                .enumerate()
                                .map(|(rid, (i, row))| {
                                    let mut result = vec![format!("{:#010X}", i * BYTES)];
                                    result.extend(row.into_iter().map(|v| {
                                        if self.binary {
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

                                    if i * BYTES == pc as usize {
                                        Row::new(result).on_red()
                                    } else if rid % 2 == 0 {
                                        Row::new(result).on_light_blue()
                                    } else {
                                        Row::new(result).on_blue()
                                    }
                                }),
                            columns,
                        )
                        .header(Row::new(headers).on_blue().bold());

                        table.render(chunks[1], buf);
                    } else {
                        const BYTES: usize = 16;
                        let mut headers = vec!["Address".to_string()];
                        headers.extend((0..BYTES).into_iter().map(|i| format!("{i:02X}")));
                        let mut columns = vec![Constraint::Max(10)];
                        columns.extend(
                            (0..BYTES).map(|_| Constraint::Max(if self.binary { 8 } else { 2 })),
                        );

                        let table = Table::new(
                            page.chunks(BYTES)
                                .enumerate()
                                .skip(self.page_offset)
                                .take((chunks[1].height - 1) as usize)
                                .enumerate()
                                .map(|(rid, (i, row))| {
                                    let mut result = vec![format!("{:#010X}", i * BYTES)];
                                    result.extend(row.into_iter().map(|v| {
                                        if self.binary {
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
                        .header(Row::new(headers).on_blue().bold());

                        table.render(chunks[1], buf);
                    }
                } else {
                    let split = Layout::vertical([
                        Constraint::Fill(1),
                        Constraint::Min(1),
                        Constraint::Fill(1),
                    ])
                    .split(chunks[1]);
                    Block::new().on_blue().render(split[0], buf);
                    Block::new().on_blue().render(split[2], buf);

                    Paragraph::new("Not allocated")
                        .centered()
                        .block(Block::new().on_blue())
                        .render(split[1], buf);
                }
            }
            View::Cache => {
                let cache_name = &self.caches[self.cache_index];

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

                let split =
                    Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).split(chunks[1]);

                Paragraph::new(Line::from(
                    format!(
                        "{} ({}/{})",
                        cache_name,
                        self.cache_index + 1,
                        self.cache_count
                    )
                    .blue()
                    .bold()
                    .underlined(),
                ))
                .render(split[0], buf);

                let data = cache_lines
                    .into_iter()
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
                .render(split[1], buf);

                Paragraph::new(Line::from(vec![
                    "↔".blue().bold(),
                    " to switch caches".into(),
                ]))
                .block(
                    help_block.title(
                        Title::from(format!(" {} | {} ", self.page, self.page_offset))
                            .alignment(Alignment::Center),
                    ),
                )
                .render(chunks[2], buf);
            }
            View::PipelineStages => {
                let stages = self.pipeline.stages();

                let splits: Vec<_> = Layout::horizontal([Constraint::Fill(1); 5])
                    .split(chunks[1])
                    .into_iter()
                    .zip(["Fetch", "Decode", "Execute", "Memory", "Writeback"])
                    .map(|(split, stage)| {
                        let block = Block::new()
                            .border_type(BorderType::Rounded)
                            .borders(Borders::ALL)
                            .title(Line::from(vec!["Stage: ".into(), stage.red().bold()]));

                        let area = block.inner(*split);
                        block.render(*split, buf);

                        area
                    })
                    .collect();

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
                .render(splits[0], buf);

                match stages.decode.get_state() {
                    libpipe::decode::State::Decoding { word } => List::new(
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
                    libpipe::decode::State::Ready { word } => List::new(
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
                .render(splits[1], buf);

                match stages.execute.get_state() {
                    libpipe::execute::State::Idle => List::new(
                        [Line::from(vec!["State: ".into(), "Idle".red().bold()])]
                            .into_iter()
                            .map(ListItem::new),
                    ),
                    libpipe::execute::State::Waiting {
                        instruction,
                        wregs,
                        rvals,
                        clocks,
                    } => List::new(
                        [
                            Line::from(vec!["State: ".into(), "Waiting".red().bold()]),
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
                                            vec![
                                                get_name(r).unwrap_or("<?>").red().bold(),
                                                ", ".into(),
                                            ]
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
                                            vec![
                                                get_name(r).unwrap_or("<?>").red().bold(),
                                                ", ".into(),
                                            ]
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
                                            vec![
                                                get_name(r).unwrap_or("<?>").red().bold(),
                                                ", ".into(),
                                            ]
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
                .render(splits[2], buf);

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
                        lp,
                        state,
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
                            Line::from(vec!["State: ".into(), state.to_string().red().bold()]),
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
                            Line::from(vec![
                                "LP".red().bold(),
                                " = ".into(),
                                lp.to_string().red().bold(),
                            ]),
                        ]
                        .into_iter()
                        .map(ListItem::new),
                    ),
                    libpipe::memory::State::RetPrep {
                        link,
                        bp,
                        state,
                        clocks,
                    } => List::new(
                        [
                            Line::from(vec!["State: ".into(), "Preparing for JSR".red().bold()]),
                            Line::from(vec![
                                "Expected wait time: ".into(),
                                clocks.to_string().red().bold(),
                            ]),
                            Line::from(vec!["Link: ".into(), link.to_string().red().bold()]),
                            Line::from(vec!["State: ".into(), state.to_string().red().bold()]),
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
                    libpipe::memory::State::Halted => List::new(
                        [Line::from(vec!["State: ".into(), "Halted".red().bold()])]
                            .into_iter()
                            .map(ListItem::new),
                    ),
                }
                .render(splits[3], buf);

                match stages.writeback.get_state() {
                    Some(_) => List::new(
                        [Line::from(vec!["State: ".into(), "Busy".red().bold()])]
                            .into_iter()
                            .map(ListItem::new),
                    ),
                    None => List::new(
                        [Line::from(vec!["State: ".into(), "Idle".red().bold()])]
                            .into_iter()
                            .map(ListItem::new),
                    ),
                }
                .render(splits[4], buf);
            }
        }
    }
}
