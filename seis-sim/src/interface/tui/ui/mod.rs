mod cache;
mod events;
mod memory;
mod pipeline_stage;
mod registers;
mod render;

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
