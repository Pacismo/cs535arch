mod cache;
mod events;
mod memory;
mod pipeline_stage;
mod registers;

use crate::{config::SimulationConfiguration, PAGES};
use crossterm::event;
use libpipe::{Pipeline, PipelineStage, PipelineStages};
use libseis::{
    instruction_set::{decode, Instruction},
    pages::PAGE_SIZE,
    registers::{get_name, PC},
    types::{Register, Word},
};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    prelude::{Buffer, CrosstermBackend, Stylize, Terminal},
    style::{Color, Style},
    symbols::{self, border},
    text::Line,
    widgets::{
        block::Title, canvas::Canvas, Block, BorderType, Borders, Clear, List, ListItem, Padding,
        Paragraph, Row, Table, Tabs, Widget,
    },
    Frame,
};
use std::{
    error::Error,
    io::Stdout,
    rc::Rc,
    time::{Duration, Instant},
};

pub type Term = Terminal<CrosstermBackend<Stdout>>;

const BYTES_PER_ROW_HEXDEC: usize = 32;
const BYTES_PER_ROW_BINARY: usize = 8;

const PIPELINE_STAGE_VIEW_COUNT: usize = 3;

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
struct MemoryView {
    page: usize,
    page_offset: usize,
    disassembly: bool,
    binary: bool,
}

#[derive(Debug)]
struct CacheView {
    index: usize,
    count: usize,
    names: Vec<String>,
    scroll: usize,
}

#[derive(Debug)]
struct PipelineView {
    stage: usize,
}

#[derive(Debug)]
struct RunningView {
    enabled: bool,
    progress_bar: isize,
    slow_mode: bool,
}

#[derive(Debug)]
pub struct Runtime<'a> {
    config: SimulationConfiguration,

    view: View,
    pipeline: &'a mut dyn Pipeline,
    clocks: usize,
    clocks_required: usize,

    memory_view: MemoryView,
    cache_view: CacheView,
    pipeline_view: PipelineView,

    running: RunningView,

    last_update: Instant,
}

impl<'a> Runtime<'a> {
    pub fn new(pipeline: &'a mut dyn Pipeline, config: SimulationConfiguration) -> Self {
        let mut caches: Vec<_> = config.cache.keys().map(|s| s.to_owned()).collect();
        caches.sort();
        let cache_count = caches.len();

        Self {
            config,

            view: View::default(),
            pipeline,
            clocks: 0,
            clocks_required: 1,

            memory_view: MemoryView {
                page: 0,
                page_offset: 0,
                disassembly: false,
                binary: false,
            },
            cache_view: CacheView {
                index: 0,
                count: cache_count,
                names: caches,
                scroll: 0,
            },
            pipeline_view: PipelineView { stage: 0 },

            running: RunningView {
                progress_bar: 0,
                enabled: false,
                slow_mode: false,
            },

            last_update: Instant::now(),
        }
    }

    pub fn run(&mut self, terminal: &mut Term) -> Result<(), Box<dyn Error>> {
        loop {
            const UPDATE_DELAY: Duration = Duration::from_millis(100);

            if self.running.enabled {
                if Instant::now().duration_since(self.last_update) >= UPDATE_DELAY {
                    terminal.draw(|frame| self.draw(frame))?;
                    self.last_update = Instant::now();
                }
            } else {
                terminal.draw(|frame| self.draw(frame))?;
                self.last_update = Instant::now();
            }

            if !self.process_input()? {
                break;
            }
        }

        Ok(())
    }

    fn draw<'f>(&mut self, frame: &mut Frame<'f>) {
        frame.render_widget(self, frame.size());
    }
}

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
